// Original offscreen comparison for tools/metal_memory_probe.m. No window or OSC.
// SPDX-License-Identifier: GPL-3.0-or-later
use std::{error::Error, process::Command, time::Duration};
use wgpu::util::DeviceExt;

const SHADER: &str = r#"
@group(0) @binding(0) var<uniform> color: vec4f;
@vertex fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
    let p = array<vec2f, 3>(vec2f(-1, -1), vec2f(3, -1), vec2f(-1, 3));
    return vec4f(p[i], 0, 1);
}
@fragment fn fs_main() -> @location(0) vec4f { return color; }
"#;

fn main() -> Result<(), Box<dyn Error>> {
    let mut single = false;
    let mut upload = false;
    let mut recreate = false;
    let mut separate = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "single-pass" => single = true,
            "upload" => upload = true,
            "recreate-bind-group" => recreate = true,
            "separate-submissions" => separate = true,
            _ => {
                return Err(
                    "usage: memory_probe [single-pass] [upload] [recreate-bind-group] [separate-submissions]".into(),
                );
            }
        }
    }
    pollster::block_on(run(single, upload, recreate, separate))
}

async fn run(
    single: bool,
    upload: bool,
    recreate: bool,
    separate: bool,
) -> Result<(), Box<dyn Error>> {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            ..Default::default()
        })
        .await?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("SCShader offscreen memory probe"),
            ..Default::default()
        })
        .await?;
    println!(
        "adapter={:?} single={single} upload={upload} recreate={recreate} separate={separate}",
        adapter.get_info()
    );
    let targets: Vec<_> = (0..2)
        .map(|_| {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("probe target"),
                size: wgpu::Extent3d {
                    width: 256,
                    height: 256,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Bgra8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
            let view = texture.create_view(&Default::default());
            (texture, view)
        })
        .collect();
    let color = [0.1_f32, 0.2, 0.3, 1.0];
    let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("probe color"),
        contents: bytemuck::cast_slice(&color),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("probe binding layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let bind = || {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("probe bindings"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        })
    };
    let mut bind_group = bind();
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("probe pipeline layout"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("probe shader"),
        source: wgpu::ShaderSource::Wgsl(SHADER.into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("probe pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::TextureFormat::Bgra8Unorm.into())],
        }),
        multiview_mask: None,
        cache: None,
    });
    for frame in 0..=20000_u32 {
        if upload {
            queue.write_buffer(&uniform, 0, bytemuck::cast_slice(&color));
        }
        if recreate {
            bind_group = bind();
        }
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("probe command encoder"),
        });
        for (index, (_, view)) in targets.iter().take(if single { 1 } else { 2 }).enumerate() {
            let attachments = [Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })];
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("probe pass"),
                color_attachments: &attachments,
                ..Default::default()
            });
            pass.set_pipeline(&pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.draw(0..3, 0..1);
            drop(pass);
            if separate && index == 0 && !single {
                queue.submit(Some(encoder.finish()));
                encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("probe next encoder"),
                });
            }
        }
        queue.submit(Some(encoder.finish()));
        device.poll(wgpu::PollType::Wait {
            submission_index: None,
            timeout: Some(Duration::from_secs(2)),
        })?;
        if frame.is_multiple_of(2000) {
            // Diagnostic tool for macOS/Linux. No platform dependency or unsafe FFI.
            let output = Command::new("ps")
                .args(["-o", "rss=", "-p", &std::process::id().to_string()])
                .output()?;
            if !output.status.success() {
                return Err("RSS sampling failed".into());
            }
            let rss: u64 = std::str::from_utf8(&output.stdout)?.trim().parse()?;
            println!("frames={frame} rss_kib={rss}");
            if rss == 0 || rss > 512 * 1024 {
                return Err("RSS bound exceeded".into());
            }
            #[cfg(feature = "gpu-counters")]
            println!("counters={:?}", device.get_internal_counters());
        }
    }
    Ok(())
}
