use super::gpu::Candidate;

// One history image plus its render and presentation bindings.
pub struct Output {
    pub candidate: Candidate,
    pub parameters: Vec<super::parameters::Parameter>,
    numbers: wgpu::Buffer,
    pub inputs: Option<wgpu::BindGroup>,
    pub texture: wgpu::Texture,
    frame: wgpu::Buffer,
    bindings: wgpu::BindGroup,
    presenter: wgpu::RenderPipeline,
    presentation: wgpu::BindGroup,
    pub width: u32,
    pub height: u32,
}
impl Output {
    pub fn new(
        device: &wgpu::Device,
        candidate: Candidate,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> Result<Self, String> {
        Self::with_parameters(device, candidate, width, height, format, vec![])
    }
    pub fn with_parameters(
        device: &wgpu::Device,
        candidate: Candidate,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        parameters: Vec<super::parameters::Parameter>,
    ) -> Result<Self, String> {
        if width == 0
            || height == 0
            || width > 4096
            || height > 4096
            || width > device.limits().max_texture_dimension_2d
            || height > device.limits().max_texture_dimension_2d
        {
            return Err("Hydra output dimensions exceed limits".into());
        }
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Hydra history image"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let frame = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hydra frame"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let numbers = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hydra numeric values"),
            size: 1024,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &candidate._layout,
            entries: &{
                let mut entries = vec![wgpu::BindGroupEntry {
                    binding: 0,
                    resource: frame.as_entire_binding(),
                }];
                if !parameters.is_empty() {
                    entries.push(wgpu::BindGroupEntry {
                        binding: 1,
                        resource: numbers.as_entire_binding(),
                    });
                }
                entries
            },
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            }],
        });
        let presentation = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(
                    &texture.create_view(&Default::default()),
                ),
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let source = format!(
            r#"
@group(0) @binding(0) var image: texture_2d<f32>;
struct Vertex {{ @builtin(position) position:vec4<f32>, @location(0) uv:vec2<f32> }};
@vertex fn vs_main(@builtin(vertex_index) i:u32)->Vertex {{
 let p=array<vec2<f32>,3>(vec2<f32>(-1.,-1.),vec2<f32>(3.,-1.),vec2<f32>(-1.,3.))[i];
 var v:Vertex;v.position=vec4<f32>(p,0.,1.);v.uv=p*0.5+vec2<f32>(0.5);return v;
}}
@fragment fn fs_main(v:Vertex)->@location(0) vec4<f32> {{
 let size=textureDimensions(image); let uv=vec2<f32>(v.uv.x,1.-v.uv.y);
 let point=clamp(vec2<i32>(uv*vec2<f32>(size)),vec2<i32>(0),vec2<i32>(size)-vec2<i32>(1));
 let c=textureLoad(image,point,0); {}
}}
"#,
            if format.is_srgb() {
                "let low=c.rgb/12.92;let high=pow((c.rgb+vec3<f32>(0.055))/1.055,vec3<f32>(2.4));return vec4<f32>(select(high,low,c.rgb<=vec3<f32>(0.04045)),c.a);"
            } else {
                "return c;"
            }
        );
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Hydra presentation"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        let presenter = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Hydra presentation"),
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
                targets: &[Some(format.into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        if let Some(error) = pollster::block_on(scope.pop()) {
            return Err(error.to_string());
        }
        Ok(Self {
            parameters,
            numbers,
            candidate,
            inputs: None,
            texture,
            frame,
            bindings,
            presenter,
            presentation,
            width,
            height,
        })
    }
    #[allow(dead_code)] // Compatibility wrapper for fixed-clock GPU fixtures.
    pub fn encode_output(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        time: f32,
        delta: f32,
        frame: u32,
    ) {
        self.encode_frame(
            queue,
            encoder,
            super::parameters::Frame::new(time, delta, frame),
        );
    }
    pub fn encode_frame(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        frame: super::parameters::Frame,
    ) {
        if !self.parameters.is_empty() {
            let mut data = [0f32; 256];
            for p in &self.parameters {
                data[(p.id - 1) as usize] = p.value_at(frame);
            }
            queue.write_buffer(&self.numbers, 0, bytemuck::cast_slice(&data));
        }
        let mut values = [0f32; 16];
        values[..8].copy_from_slice(&[
            frame.time,
            frame.speed,
            frame.bpm,
            frame.delta,
            self.width as f32,
            self.height as f32,
            1. / self.width as f32,
            1. / self.height as f32,
        ]);
        values[12] = f32::from_bits(frame.frame);
        queue.write_buffer(&self.frame, 0, bytemuck::cast_slice(&values));
        let view = self.texture.create_view(&Default::default());
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Hydra output render"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&self.candidate._pipeline);
        pass.set_bind_group(0, &self.bindings, &[]);
        if let Some(inputs) = &self.inputs {
            pass.set_bind_group(1, inputs, &[]);
        }
        pass.draw(0..3, 0..1);
    }
    pub fn encode_present(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        self.encode_present_region(encoder, view, None);
    }
    pub fn encode_present_region(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        region: Option<[f32; 4]>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Hydra presentation"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: if region.is_some() {
                        wgpu::LoadOp::Load
                    } else {
                        wgpu::LoadOp::Clear(wgpu::Color::BLACK)
                    },
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        if let Some([x, y, w, h]) = region {
            pass.set_viewport(x, y, w, h, 0., 1.);
        }
        pass.set_pipeline(&self.presenter);
        pass.set_bind_group(0, &self.presentation, &[]);
        pass.draw(0..3, 0..1);
    }
}
