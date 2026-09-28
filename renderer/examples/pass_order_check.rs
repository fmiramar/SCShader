// Original GPU readback regression: graph -> feedback + presentation, across frames.
// SPDX-License-Identifier: GPL-3.0-or-later
use std::{error::Error, sync::mpsc, time::Duration};

const SHADER: &str = r#"
@group(0) @binding(0) var source: texture_2d<f32>;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
    let p = array<vec2f, 3>(vec2f(-1, -1), vec2f(3, -1), vec2f(-1, 3));
    return vec4f(p[i], 0, 1);
}
@fragment fn fs_main(@builtin(position) p: vec4f) -> @location(0) vec4f {
    return textureLoad(source, vec2i(p.xy), 0) + vec4f(1.0 / 255.0, 2.0 / 255.0, 0, 0);
}
"#;

fn main() -> Result<(), Box<dyn Error>> {
    pollster::block_on(run())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })
        .await?;
    let (device, queue) = adapter.request_device(&Default::default()).await?;
    println!("adapter={:?}", adapter.get_info());
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("pass-order regression"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            count: None,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    let extent = wgpu::Extent3d {
        width: 4,
        height: 4,
        depth_or_array_layers: 1,
    };
    // Previous/next feedback, graph intermediate, and presentation destination.
    let textures: Vec<_> = (0..4)
        .map(|_| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            })
        })
        .collect();
    let views: Vec<_> = textures
        .iter()
        .map(|t| t.create_view(&Default::default()))
        .collect();
    let bind_groups: Vec<_> = views
        .iter()
        .map(|view| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                }],
            })
        })
        .collect();
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 256 * 4,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    for split in [false, true] {
        for view in &views {
            let mut encoder = device.create_command_encoder(&Default::default());
            encode(&mut encoder, view, None);
            queue.submit(Some(encoder.finish()));
        }
        for frame in 0..40 {
            let mut encoder = device.create_command_encoder(&Default::default());
            let previous = frame % 2;
            let next = 1 - previous;
            // The terminal shader executes twice with the same input, exactly as
            // in SCShader. No CPU completion waits between frames or submissions.
            for (index, (source, destination)) in
                [(previous, 2), (2, next), (2, 3)].iter().enumerate()
            {
                encode(
                    &mut encoder,
                    &views[*destination],
                    Some((&pipeline, &bind_groups[*source])),
                );
                if split && index < 2 {
                    queue.submit(Some(encoder.finish()));
                    encoder = device.create_command_encoder(&Default::default());
                }
            }
            queue.submit(Some(encoder.finish()));
        }
        // After 40 frames, each full graph has incremented RG by (2, 4) bytes.
        // The current feedback texture and presentation must contain identical pixels.
        for target in [0, 3] {
            let mut encoder = device.create_command_encoder(&Default::default());
            encoder.copy_texture_to_buffer(
                textures[target].as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &readback,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(256),
                        rows_per_image: Some(4),
                    },
                },
                extent,
            );
            queue.submit(Some(encoder.finish()));
            let (send, receive) = mpsc::channel();
            readback.map_async(wgpu::MapMode::Read, .., move |result| {
                let _ = send.send(result);
            });
            device.poll(wgpu::PollType::Wait {
                submission_index: None,
                timeout: Some(Duration::from_secs(5)),
            })?;
            receive.recv_timeout(Duration::from_secs(1))??;
            {
                let bytes = readback.get_mapped_range(..)?;
                for row in 0..4 {
                    for column in 0..4 {
                        let start = row * 256 + column * 4;
                        assert_eq!(
                            &bytes[start..start + 4],
                            &[80, 160, 0, 255],
                            "split={split} target={target} row={row} column={column}"
                        );
                    }
                }
            }
            readback.unmap();
        }
        println!(
            "PASS split={split}: 40 feedback frames; graph, feedback, and presentation pixel ordering"
        );
    }
    Ok(())
}

fn encode(
    encoder: &mut wgpu::CommandEncoder,
    target: &wgpu::TextureView,
    draw: Option<(&wgpu::RenderPipeline, &wgpu::BindGroup)>,
) {
    let attachments = [Some(wgpu::RenderPassColorAttachment {
        view: target,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
            store: wgpu::StoreOp::Store,
        },
    })];
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &attachments,
        ..Default::default()
    });
    if let Some((pipeline, bind_group)) = draw {
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
}
