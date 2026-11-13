#[path = "../src/hydra/parameters.rs"]
mod parameters;
// Small numerical differential check. Reads AGPL reference GLSL from an external
// pinned checkout at runtime; no upstream shader bodies are distributed here.
#[path = "../src/hydra/compiler.rs"]
mod compiler;
#[path = "../src/hydra/families.rs"]
mod families;
#[path = "../src/hydra/gpu.rs"]
mod gpu;
#[path = "../src/hydra/ir.rs"]
mod ir;
#[path = "../src/hydra/modulation.rs"]
mod modulation;
#[path = "../src/hydra/registry.rs"]
mod registry;
use ir::{Argument, Node, Program};
use std::{error::Error, fmt::Write, sync::mpsc, time::Duration};
fn scalars(values: &[f32]) -> Vec<Argument> {
    values.iter().map(|v| Argument::Float(*v)).collect()
}
fn node(id: u32, name: &str, receiver: u32, args: Vec<Argument>) -> Node {
    Node {
        id,
        name: name.into(),
        category: registry::spec(name).unwrap().category,
        receiver,
        arguments: args,
    }
}
fn defaults(spec: &registry::Spec) -> Vec<Argument> {
    let mut cursor = 0;
    spec.args
        .iter()
        .map(|kind| match kind {
            registry::ValueType::Output => unreachable!(),
            registry::ValueType::Expression => Argument::Expression(6),
            registry::ValueType::Float => {
                let value = spec._defaults[cursor];
                cursor += 1;
                Argument::Float(value)
            }
            registry::ValueType::Vector => {
                let value = spec._defaults[cursor..cursor + 4].try_into().unwrap();
                cursor += 4;
                Argument::Vector(value)
            }
        })
        .collect()
}
fn body(text: &str) -> Result<&str, Box<dyn Error>> {
    let start = text.find('`').ok_or("reference template missing")? + 1;
    Ok(&text[start
        ..start
            + text[start..]
                .find('`')
                .ok_or("reference template not terminated")?])
}
fn reference(
    program: &Program,
    functions: &str,
    utilities: &str,
) -> Result<String, Box<dyn Error>> {
    let mut source = String::from(
        "#version 450\nlayout(set=0,binding=0,std140) uniform Frame {vec4 clock;vec4 resolution;vec4 pointer;uvec4 counters;} hydra;\n#define time hydra.clock.x\n#define resolution hydra.resolution.xy\nlayout(location=0) in vec2 uv;layout(location=0) out vec4 result;\n",
    );
    for utility in utilities.split("glsl:").skip(1) {
        source.push_str(body(utility)?);
        source.push('\n');
    }
    let mut emitted = std::collections::HashSet::new();
    for n in &program.nodes {
        let spec = registry::spec(&n.name).unwrap();
        if emitted.insert(n.name.as_str()) {
            let marker = format!("\n  name: '{}',", n.name);
            let rest = functions
                .split("export default () => [")
                .nth(1)
                .ok_or("missing reference exports")?
                .split(&marker)
                .nth(1)
                .ok_or("missing reference operation")?;
            let function_body = body(rest.split("glsl:").nth(1).ok_or("reference GLSL missing")?)?;
            write!(
                source,
                "{} {}({} {}",
                if matches!(spec.category, 1 | 4) {
                    "vec2"
                } else {
                    "vec4"
                },
                n.name,
                if matches!(spec.category, 2 | 3) {
                    "vec4"
                } else {
                    "vec2"
                },
                if matches!(spec.category, 2 | 3) {
                    "_c0"
                } else {
                    "_st"
                }
            )?;
            for (name, kind) in spec._names.iter().zip(spec.args) {
                write!(
                    source,
                    ",{} {}",
                    if matches!(
                        kind,
                        registry::ValueType::Vector | registry::ValueType::Expression
                    ) {
                        "vec4"
                    } else {
                        "float"
                    },
                    if *kind == registry::ValueType::Expression {
                        if spec.category == 4 { "_c0" } else { "_c1" }
                    } else {
                        name
                    }
                )?;
            }
            writeln!(source, "){{{function_body}}}")?;
        }
        let args = n
            .arguments
            .iter()
            .map(|arg| match arg {
                Argument::Parameter(_) => unreachable!("numeric sources have a separate fixture"),
                Argument::Float(v) => format!("uintBitsToFloat(0x{:08x}u)", v.to_bits()),
                Argument::Vector(v) => format!(
                    "vec4({})",
                    v.iter()
                        .map(|v| format!("uintBitsToFloat(0x{:08x}u)", v.to_bits()))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
                Argument::Output(_) => unreachable!(),
                Argument::Expression(id) => format!("ref{id}(p)"),
            })
            .collect::<Vec<_>>()
            .join(",");
        let separator = if args.is_empty() { "" } else { "," };
        let call = match spec.category {
            0 => format!("{}(p{separator}{args})", n.name),
            1 | 4 => format!("ref{}({}(p{separator}{args}))", n.receiver, n.name),
            _ => format!("{}(ref{}(p){separator}{args})", n.name, n.receiver),
        };
        writeln!(source, "vec4 ref{}(vec2 p){{return {call};}}", n.id)?;
    }
    writeln!(source, "void main(){{result=ref{}(uv);}}", program.root)?;
    Ok(source)
}
fn draw(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,
    layout: &wgpu::BindGroupLayout,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let size = wgpu::Extent3d {
        width: 3,
        height: 3,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let frame = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let data: [f32; 16] = [
        0.37,
        1.,
        30.,
        0.,
        3.,
        3.,
        1. / 3.,
        1. / 3.,
        0.,
        0.,
        0.,
        0.,
        0.,
        0.,
        0.,
        0.,
    ];
    queue.write_buffer(&frame, 0, bytemuck::cast_slice(&data));
    let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: frame.as_entire_binding(),
        }],
    });
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: 768,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let view = texture.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
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
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bindings, &[]);
        pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: Some(3),
            },
        },
        size,
    );
    queue.submit(Some(encoder.finish()));
    let (tx, rx) = mpsc::channel();
    buffer.map_async(wgpu::MapMode::Read, .., move |r| {
        let _ = tx.send(r);
    });
    device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(Duration::from_secs(5)),
    })?;
    rx.recv_timeout(Duration::from_secs(5))??;
    let mapped = buffer.get_mapped_range(..)?;
    let mut pixels = Vec::new();
    for row in 0..3 {
        pixels.extend_from_slice(&mapped[row * 256..row * 256 + 12]);
    }
    drop(mapped);
    buffer.unmap();
    Ok(pixels)
}
fn checked(program: Program) -> Program {
    let mut bytes = b"HYIR\0\x01\0\0".to_vec();
    for value in [program.nodes.len() as u32, 0, program.root] {
        bytes.extend(value.to_be_bytes());
    }
    for node in program.nodes {
        bytes.extend(node.id.to_be_bytes());
        bytes.extend([node.category, node.arguments.len() as u8]);
        bytes.extend((node.name.len() as u16).to_be_bytes());
        bytes.extend(node.name.as_bytes());
        bytes.extend(node.receiver.to_be_bytes());
        for arg in node.arguments {
            match arg {
                Argument::Parameter(_) => unreachable!("numeric sources have a separate fixture"),
                Argument::Float(value) => {
                    bytes.push(0);
                    bytes.extend(value.to_be_bytes());
                }
                Argument::Vector(values) => {
                    bytes.push(5);
                    for value in values {
                        bytes.extend(value.to_be_bytes());
                    }
                }
                Argument::Output(slot) => {
                    bytes.extend([4, slot]);
                }
                Argument::Expression(id) => {
                    bytes.push(2);
                    bytes.extend(id.to_be_bytes());
                }
            }
        }
    }
    Program::decode(&bytes).unwrap()
}
fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::args()
        .nth(1)
        .ok_or("pass the pinned external hydra-synth checkout path")?;
    let sha = std::process::Command::new("git")
        .args(["-C", &root, "rev-parse", "HEAD"])
        .output()?;
    if !sha.status.success()
        || String::from_utf8(sha.stdout)?.trim() != "9d29a9f4fd8f9081b9759943f38db36f05b9a88f"
    {
        return Err("reference SHA mismatch".into());
    }
    let read_reference = |path: &str| -> Result<String, Box<dyn Error>> {
        let object = format!("9d29a9f4fd8f9081b9759943f38db36f05b9a88f:{path}");
        let result = std::process::Command::new("git")
            .args(["--no-replace-objects", "-C", &root, "show", &object])
            .output()?;
        if !result.status.success() {
            return Err("cannot read pinned reference object".into());
        }
        Ok(String::from_utf8(result.stdout)?)
    };
    let functions = read_reference("src/glsl/glsl-functions.js")?;
    let utilities = read_reference("src/glsl/utility-functions.js")?;
    let backend = std::env::args().nth(2).unwrap_or_else(|| "metal".into());
    let backends = match backend.as_str() {
        "metal" => wgpu::Backends::METAL,
        "vulkan" => wgpu::Backends::VULKAN,
        "dx12" => wgpu::Backends::DX12,
        _ => return Err("invalid backend".into()),
    };
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))?;
    if adapter.get_info().device_type == wgpu::DeviceType::Cpu {
        return Err("CPU adapter cannot qualify native pixels".into());
    }
    println!("adapter={:?}", adapter.get_info());
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))?;
    let mut cached = None;
    // Texture/history operations have the separate production output fixture.
    for spec in registry::supported().filter(|spec| !matches!(spec.name, "src" | "prev")) {
        let mut args = defaults(spec);
        match spec.name {
            "add" | "sub" | "mult" => args[1] = Argument::Float(0.35),
            "modulateRepeat" => args[1..].clone_from_slice(&scalars(&[-3., 2., 0.35, -0.2])),
            "modulateRepeatX" | "modulateRepeatY" => {
                args[1..].clone_from_slice(&scalars(&[-3., -0.4]))
            }
            "modulateScrollX" | "modulateScrollY" => {
                args[1..].clone_from_slice(&scalars(&[-0.8, 0.13]))
            }
            "noise" => args = scalars(&[1.4, 0.3]),
            "voronoi" => args = scalars(&[3., 0.2, 0.4]),
            "shape" => args = scalars(&[5., 0.4, 0.08]),
            "osc" => args = scalars(&[7., 0.1, 0.8]),
            "repeat" => args = scalars(&[-3., 2., 0.35, -0.2]),
            "repeatX" | "repeatY" => args = scalars(&[-3., 0.4]),
            "scroll" => args = scalars(&[-0.8, -0.6, 0.1, -0.2]),
            "scrollX" | "scrollY" => args = scalars(&[-0.8, -0.2]),
            "shift" => args = scalars(&[-0.2, 0.9, 0.7, -0.25]),
            "luma" | "thresh" => args = scalars(&[0.35, 0.2]),
            "sum" => args = vec![Argument::Vector([0.1, 0.2, 0.3, 0.4])],
            _ => {}
        }
        let nodes = if spec.category == 4 {
            vec![
                node(1, "osc", 0, scalars(&[10., 0.1, 0.8])),
                node(2, "rotate", 1, scalars(&[0.24, 0.03])),
                node(3, "gradient", 0, scalars(&[0.4])),
                node(4, "color", 3, scalars(&[0.7, 0.4, 0.8, 0.6])),
                node(
                    5,
                    "blend",
                    4,
                    vec![Argument::Expression(3), Argument::Float(0.2)],
                ),
                node(6, "rotate", 5, scalars(&[-0.13, 0.02])),
                node(
                    7,
                    "modulate",
                    2,
                    vec![Argument::Expression(6), Argument::Float(0.07)],
                ),
                node(8, spec.name, 7, args),
                node(9, "rotate", 8, scalars(&[-0.35, 0.01])),
            ]
        } else if spec.category == 3 {
            vec![
                node(1, "osc", 0, scalars(&[10., 0.1, 0.])),
                node(2, "shape", 0, scalars(&[4., 0.4, 0.01])),
                node(3, "color", 2, scalars(&[0.7, 0.5, 0.3, 0.35])),
                node(4, "solid", 0, scalars(&[0.1, 0.3, 0.8, 0.2])),
                node(
                    5,
                    "blend",
                    3,
                    vec![Argument::Expression(4), Argument::Float(0.25)],
                ),
                node(6, "rotate", 5, scalars(&[0.2, 0.])),
                node(7, spec.name, 1, args),
                node(8, "rotate", 7, scalars(&[0.13, 0.02])),
            ]
        } else if spec.category == 0 {
            vec![node(1, spec.name, 0, args)]
        } else {
            vec![
                node(
                    1,
                    if spec.category == 1 {
                        "gradient"
                    } else {
                        "solid"
                    },
                    0,
                    if spec.category == 1 {
                        scalars(&[0.4])
                    } else {
                        scalars(&[0.17, 0.38, 0.63, 0.72])
                    },
                ),
                node(2, spec.name, 1, args),
            ]
        };
        let program = checked(Program {
            parameters: vec![],
            root: nodes.last().unwrap().id,
            nodes,
        });
        gpu::Candidate::prepare(&device, &program, &mut cached)?;
        let candidate = cached.as_ref().unwrap().clone();
        let glsl = reference(&program, &functions, &utilities)?;
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let vertex = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(candidate.source.clone().into()),
        });
        let fragment = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("External pinned reference"),
            source: wgpu::ShaderSource::Glsl {
                shader: glsl.into(),
                stage: wgpu::naga::ShaderStage::Fragment,
                defines: &[],
            },
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&candidate._layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &vertex,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::TextureFormat::Rgba8Unorm.into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });
        if let Some(error) = pollster::block_on(scope.pop()) {
            return Err(format!("{} reference compile: {error}", spec.name).into());
        }
        let native = draw(&device, &queue, &candidate._pipeline, &candidate._layout)?;
        let reference = draw(&device, &queue, &pipeline, &candidate._layout)?;
        let error = native
            .iter()
            .zip(&reference)
            .map(|(a, b)| a.abs_diff(*b))
            .max()
            .unwrap();
        if error > 2 {
            return Err(format!(
                "{} difference {error}: native={native:?} reference={reference:?}",
                spec.name
            )
            .into());
        }
        if spec.category >= 3 {
            let mut bad = program.clone();
            bad.nodes[0].arguments[0] = Argument::Float(0.0);
            assert!(gpu::Candidate::prepare(&device, &bad, &mut cached).is_err());
            let kept = cached.as_ref().unwrap();
            assert_eq!(kept.source, candidate.source);
            assert_eq!(
                draw(&device, &queue, &kept._pipeline, &kept._layout)?,
                native
            );
        }
        println!("{} PASS max_channel_error={error}", spec.name);
    }
    println!("SCSHADER_HYDRA_FAMILIES_GPU_OK");
    Ok(())
}
