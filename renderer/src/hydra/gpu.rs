//! One renderer-owned compiled candidate, not an active output or presentation mode.
use super::{compiler, ir::Program};

#[derive(Clone)]
pub struct Candidate {
    pub source: String,
    pub _inputs: Option<wgpu::BindGroupLayout>,
    pub _pipeline: wgpu::RenderPipeline,
    pub _layout: wgpu::BindGroupLayout,
}
impl Candidate {
    pub fn prepare(
        device: &wgpu::Device,
        program: &Program,
        current: &mut Option<Self>,
    ) -> Result<(), String> {
        let source = compiler::compile(program)?;
        if current
            .as_ref()
            .is_some_and(|candidate| candidate.source == source)
        {
            return Ok(());
        }
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hydra frame layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            }],
        });
        let inputs = program
            .nodes
            .iter()
            .any(|node| node.name == "src")
            .then(|| {
                let mut entries = (0..4)
                    .map(|binding| wgpu::BindGroupLayoutEntry {
                        binding,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    })
                    .collect::<Vec<_>>();
                entries.push(wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                });
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("Hydra output inputs"),
                    entries: &entries,
                })
            });
        let mut layouts = vec![Some(&layout)];
        if let Some(inputs) = &inputs {
            layouts.push(Some(inputs));
        }
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Hydra candidate layout"),
            bind_group_layouts: &layouts,
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Hydra candidate WGSL"),
            source: wgpu::ShaderSource::Wgsl(source.clone().into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Hydra compiled candidate"),
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
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
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
        *current = Some(Self {
            source,
            _inputs: inputs,
            _pipeline: pipeline,
            _layout: layout,
        });
        Ok(())
    }
}
