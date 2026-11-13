#[path = "../src/hydra/parameters.rs"]
mod parameters;
// Short native offscreen pixel proof for the initial independently implemented compiler.
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
#[path = "../src/hydra/output.rs"]
mod output;
#[path = "../src/hydra/registry.rs"]
mod registry;
use ir::{Argument, Node, Program};
use std::{error::Error, sync::mpsc, time::Duration};
fn node(id: u32, name: &str, category: u8, receiver: u32, args: &[f32]) -> Node {
    Node {
        id,
        name: name.into(),
        category,
        receiver,
        arguments: args.iter().map(|v| Argument::Float(*v)).collect(),
    }
}
fn decoded(nodes: Vec<Node>, root: u32) -> Program {
    let mut bytes = b"HYIR\0\x01\0\0".to_vec();
    for value in [nodes.len() as u32, 0, root] {
        bytes.extend(value.to_be_bytes());
    }
    for node in nodes {
        bytes.extend(node.id.to_be_bytes());
        bytes.extend([node.category, node.arguments.len() as u8]);
        bytes.extend((node.name.len() as u16).to_be_bytes());
        bytes.extend(node.name.as_bytes());
        bytes.extend(node.receiver.to_be_bytes());
        for arg in node.arguments {
            let Argument::Float(value) = arg else {
                unreachable!()
            };
            bytes.push(0);
            bytes.extend(value.to_be_bytes());
        }
    }
    Program::decode(&bytes).unwrap()
}
fn main() -> Result<(), Box<dyn Error>> {
    let backend = std::env::args().nth(1).unwrap_or_else(|| {
        if cfg!(target_os = "macos") {
            "metal"
        } else if cfg!(target_os = "windows") {
            "dx12"
        } else {
            "vulkan"
        }
        .into()
    });
    let backends = match backend.as_str() {
        "metal" => wgpu::Backends::METAL,
        "dx12" => wgpu::Backends::DX12,
        "vulkan" => wgpu::Backends::VULKAN,
        _ => return Err("backend must be metal, dx12 or vulkan".into()),
    };
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        backend_options: wgpu::BackendOptions {
            dx12: wgpu::Dx12BackendOptions {
                shader_compiler: wgpu::Dx12Compiler::Fxc,
                ..Default::default()
            },
            ..Default::default()
        },
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))?;
    if adapter.get_info().device_type == wgpu::DeviceType::Cpu {
        return Err("software adapter does not qualify native GPU pixels".into());
    }
    println!("adapter={:?}", adapter.get_info());
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))?;
    let cases = [
        decoded(
            vec![
                node(1, "solid", 0, 0, &[0.25, 0.5, 0.75, 0.8]),
                node(2, "color", 2, 1, &[-1., 0.5, 2., 0.5]),
            ],
            2,
        ),
        decoded(
            vec![
                node(1, "osc", 0, 0, &[10., 0.1, 0.]),
                node(2, "rotate", 1, 1, &[0.2, 0.]),
            ],
            2,
        ),
        decoded(
            vec![
                node(1, "gradient", 0, 0, &[0.]),
                node(2, "scale", 1, 1, &[2., 1., 2., 0.3, 0.7]),
                node(3, "rotate", 1, 2, &[0.2, 0.]),
            ],
            3,
        ),
    ];
    let mut cached = None;
    for format in [
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Rgba8UnormSrgb,
    ] {
        for (index, program) in cases.iter().enumerate() {
            gpu::Candidate::prepare(&device, program, &mut cached)?;
            let before = cached.as_ref().unwrap().source.clone();
            let invalid = Program {
                parameters: vec![],
                nodes: vec![node(1, "osc", 0, 0, &[0., 0.1, 0.])],
                root: 1,
            };
            assert!(gpu::Candidate::prepare(&device, &invalid, &mut cached).is_err());
            assert_eq!(cached.as_ref().unwrap().source, before);
            let candidate = cached.as_ref().unwrap();
            let extent = wgpu::Extent3d {
                width: 3,
                height: 3,
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Hydra compiler pixel test"),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
                view_formats: &[],
            });
            let output = output::Output::new(&device, candidate.clone(), 3, 3, format)?;
            let readback = device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 256 * 3,
                usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            let view = texture.create_view(&Default::default());
            let mut encoder = device.create_command_encoder(&Default::default());
            output.encode_output(&queue, &mut encoder, 0., 0., 0);
            queue.submit(Some(encoder.finish()));
            let mut encoder = device.create_command_encoder(&Default::default());
            output.encode_present(&mut encoder, &view);
            encoder.copy_texture_to_buffer(
                texture.as_image_copy(),
                wgpu::TexelCopyBufferInfo {
                    buffer: &readback,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(256),
                        rows_per_image: Some(3),
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
            receive.recv_timeout(Duration::from_secs(5))??;
            let pixels = readback.get_mapped_range(..)?;
            for row in 0..3 {
                for col in 0..3 {
                    let x = (col as f64 + 0.5) / 3.;
                    let y = 1. - (row as f64 + 0.5) / 3.;
                    let angle = 0.2f32 as f64;
                    let u = 0.5 + angle.cos() * (x - 0.5) + angle.sin() * (y - 0.5);
                    let v = 0.5 - angle.sin() * (x - 0.5) + angle.cos() * (y - 0.5);
                    let expected = match index {
                        0 => [0.75, 0.25, 1.5, 0.4],
                        1 => {
                            let c = 0.5 + 0.5 * (10. * u).sin();
                            [c, c, c, 1.]
                        }
                        _ => [(u - 0.3) / 2. + 0.3, (v - 0.7) / 4. + 0.7, 0., 1.],
                    };
                    for channel in 0..4 {
                        let target = (expected[channel].clamp(0., 1.) * 255.).round() as i32;
                        let actual = pixels[row * 256 + col * 4 + channel] as i32;
                        assert!(
                            (actual - target).abs() <= 2,
                            "case={index} pixel=({col},{row}) channel={channel} actual={actual} expected={target}"
                        );
                    }
                }
            }
            drop(pixels);
            readback.unmap();
            println!(
                "format={format:?} case={index} nine RGBA pixels PASS; failed candidate preserved cache"
            );
        }
    }
    println!("SCSHADER_HYDRA_COMPILER_GPU_OK");
    Ok(())
}
