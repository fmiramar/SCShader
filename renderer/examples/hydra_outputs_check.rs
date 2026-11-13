//! Native production output/sampling/presentation proof; no device hooks.
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
#[path = "../src/hydra/outputs.rs"]
mod outputs;
#[path = "../src/hydra/registry.rs"]
mod registry;
use ir::{Argument, Node, Program};
use std::{error::Error, sync::mpsc, time::Duration};
fn program(name: &str, args: Vec<Argument>) -> Program {
    Program {
        nodes: vec![Node {
            id: 1,
            name: name.into(),
            category: 0,
            receiver: 0,
            arguments: args,
        }],
        root: 1,
    }
}
fn solid(r: f32, g: f32, b: f32) -> Program {
    program("solid", [r, g, b, 1.].map(Argument::Float).into())
}
fn sample(slot: u8) -> Program {
    let mut bytes = b"HYIR\0\x01\0\0".to_vec();
    for word in [1u32, 0, 1, 1] {
        bytes.extend(word.to_be_bytes());
    }
    bytes.extend([0, 1, 0, 3]);
    bytes.extend(b"src");
    bytes.extend(0u32.to_be_bytes());
    bytes.extend([4, slot]);
    Program::decode(&bytes).unwrap()
}
fn read(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let width = texture.width();
    let height = texture.height();
    let stride = (width * 4).div_ceil(256) * 256;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(stride * height),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride),
                rows_per_image: Some(height),
            },
        },
        texture.size(),
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
    let mut result = Vec::new();
    for row in 0..height {
        let begin = (row * stride) as usize;
        result.extend(&mapped[begin..begin + (width * 4) as usize]);
    }
    drop(mapped);
    buffer.unmap();
    Ok(result)
}
fn main() -> Result<(), Box<dyn Error>> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::METAL,
        ..wgpu::InstanceDescriptor::new_without_display_handle()
    });
    let adapter = pollster::block_on(instance.request_adapter(&Default::default()))?;
    if adapter.get_info().device_type == wgpu::DeviceType::Cpu {
        return Err("native GPU required".into());
    }
    println!("adapter={:?}", adapter.get_info());
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))?;
    for format in [
        wgpu::TextureFormat::Rgba8Unorm,
        wgpu::TextureFormat::Rgba8UnormSrgb,
    ] {
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 8,
                height: 8,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = target.create_view(&Default::default());
        let mut set = outputs::Outputs::new(&device, 8, 8, format)?;
        let present = |set: &outputs::Outputs, mode| -> Result<Vec<u8>, Box<dyn Error>> {
            set.encode_outputs(&device, &queue, 0.37, 0.01, 1);
            let mut encoder = device.create_command_encoder(&Default::default());
            set.encode_present(&mut encoder, &view, mode, 8, 8);
            queue.submit(Some(encoder.finish()));
            read(&device, &queue, &target)
        };
        assert!(present(&set, 4)?.iter().all(|v| *v == 0));
        set.prepare(&device, 3, &sample(2))?;
        assert!(present(&set, 3)?.iter().all(|v| *v == 0));
        let colors = [
            [255, 0, 0, 255],
            [0, 255, 0, 255],
            [0, 0, 255, 255],
            [255, 255, 0, 255],
        ];
        for (i, c) in colors.iter().enumerate() {
            set.prepare(
                &device,
                i,
                &solid(c[0] as f32 / 255., c[1] as f32 / 255., c[2] as f32 / 255.),
            )?;
        }
        let pixels = present(&set, 4)?;
        for (i, (x, y)) in [(1, 1), (1, 6), (6, 1), (6, 6)].into_iter().enumerate() {
            let begin = (y * 8 + x) * 4;
            assert_eq!(&pixels[begin..begin + 4], &colors[i]);
        }
        for (i, color) in colors.iter().enumerate() {
            assert!(present(&set, i as i32)?.chunks_exact(4).all(|p| p == color));
        }
        assert_eq!(set.texture_bytes(), 4 + 4 * 8 * 8 * 4);
        set.prepare(&device, 0, &program("gradient", vec![Argument::Float(0.)]))?;
        set.prepare(&device, 1, &sample(0))?;
        let mut inverted = sample(1);
        inverted.nodes.push(Node {
            id: 2,
            name: "invert".into(),
            category: 2,
            receiver: 1,
            arguments: vec![Argument::Float(1.)],
        });
        inverted.root = 2;
        set.prepare(&device, 2, &inverted)?;
        let _ = present(&set, 2)?;
        let a = read(&device, &queue, &set.slots[0].as_ref().unwrap().texture)?;
        let b = read(&device, &queue, &set.slots[1].as_ref().unwrap().texture)?;
        assert_eq!(a, b, "src must preserve gradient orientation");
        let c = read(&device, &queue, &set.slots[2].as_ref().unwrap().texture)?;
        for (left, right) in a.chunks_exact(4).zip(c.chunks_exact(4)) {
            for channel in 0..3 {
                assert!((255 - left[channel]).abs_diff(right[channel]) <= 1);
            }
            assert_eq!(right[3], 255);
        }
        assert!(set.prepare(&device, 0, &sample(2)).is_err());
        assert!(set.prepare(&device, 3, &sample(3)).is_err());
        assert_eq!(
            a,
            read(&device, &queue, &set.slots[0].as_ref().unwrap().texture)?
        );
        let mut scaled = sample(0);
        scaled.nodes.push(Node {
            id: 2,
            name: "scale".into(),
            category: 1,
            receiver: 1,
            arguments: [2., 1., 1., 0.5, 0.5].map(Argument::Float).into(),
        });
        scaled.root = 2;
        set.prepare(&device, 2, &scaled)?;
        let _ = present(&set, 2)?;
        let filtered = read(&device, &queue, &set.slots[2].as_ref().unwrap().texture)?;
        for y in 0..8 {
            for x in 0..8 {
                let px = x as f32 * 0.5 + 1.75;
                let py = y as f32 * 0.5 + 1.75;
                let ix = px.floor() as usize;
                let iy = py.floor() as usize;
                let fx = px - px.floor();
                let fy = py - py.floor();
                for channel in 0..4 {
                    let get = |dx, dy| a[((iy + dy) * 8 + ix + dx) * 4 + channel] as f32;
                    let expected = (get(0, 0) * (1. - fx) + get(1, 0) * fx) * (1. - fy)
                        + (get(0, 1) * (1. - fx) + get(1, 1) * fx) * fy;
                    assert!(
                        (filtered[(y * 8 + x) * 4 + channel] as f32 - expected).abs() <= 1.0,
                        "bilinear sampling mismatch"
                    );
                }
            }
        }
        // A backwards-numbered dependency must be evaluated producer first.
        set.prepare(&device, 0, &solid(0.2, 0.4, 0.6))?;
        set.prepare(&device, 1, &solid(0., 0., 0.))?;
        set.prepare(&device, 2, &solid(0., 0., 0.))?;
        set.prepare(&device, 0, &sample(3))?;
        set.prepare(&device, 3, &sample(2))?;
        set.prepare(&device, 2, &solid(0.2, 0.4, 0.6))?;
        let pixels = present(&set, 0)?;
        assert!(pixels.chunks_exact(4).all(|p| {
            p.iter()
                .zip([51, 102, 153, 255])
                .all(|(a, b)| a.abs_diff(b) <= 1)
        }));
        assert!(set.resized(&device, 4097, 8).is_err());
        let resized = set.resized(&device, 16, 12)?;
        assert_eq!(resized.dimensions(), (16, 12));
        resized.encode_outputs(&device, &queue, 0.5, 0.01, 2);
        assert_eq!(resized.texture_bytes(), 4 + 4 * 16 * 12 * 4);
        let pixels = read(&device, &queue, &resized.slots[0].as_ref().unwrap().texture)?;
        assert!(pixels.chunks_exact(4).all(|p| p == [51, 102, 153, 255]));
        if let Some(error) = pollster::block_on(scope.pop()) {
            return Err(error.to_string().into());
        }
        println!(
            "{format:?}: blank, quadrants, switching, sampling, rollback, reverse dependencies and resize PASS"
        );
    }
    println!("SCSHADER_HYDRA_OUTPUTS_GPU_OK");
    Ok(())
}
