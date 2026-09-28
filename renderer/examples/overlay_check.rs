//! Native offscreen overlay pixel regression; optional output PNG for visual review.
#[path = "../src/overlay.rs"]
mod overlay;
use std::{error::Error, sync::mpsc, time::Duration};

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
    let extent = wgpu::Extent3d {
        width: 960,
        height: 200,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("overlay regression target"),
        size: extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let mut encoder = device.create_command_encoder(&Default::default());
    drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: None,
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    }));
    queue.submit(Some(encoder.finish()));
    let mut panel = overlay::Overlay::new(&device, wgpu::TextureFormat::Rgba8Unorm);
    assert!(panel.bytes() < 256 * 1024);
    panel.update(
        &queue,
        extent.width,
        extent.height,
        &[
            "SCSHADER GPU DIAGNOSTICS".to_owned(),
            "FPS 60.0  CPU 0.80 MS  GPU N/A".to_owned(),
            format!("BACKEND {:?}", adapter.get_info().backend),
            format!("DEVICE {}", adapter.get_info().name),
            "SURFACE 960 X 200  FRAME 12345".to_owned(),
            "LAST COMPILE OK".to_owned(),
            "SCHEDULED 4  LATE 0".to_owned(),
            "OWNED TEXTURES 4.25 MIB (EST.)".to_owned(),
            "OVERLAY EXCLUDED FROM FEEDBACK".to_owned(),
        ],
    );
    let mut encoder = device.create_command_encoder(&Default::default());
    panel.encode(&mut encoder, &view);
    queue.submit(Some(encoder.finish()));
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(extent.width * extent.height * 4),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(extent.width * 4),
                rows_per_image: Some(extent.height),
            },
        },
        extent,
    );
    queue.submit(Some(encoder.finish()));
    let (send, receive) = mpsc::channel();
    buffer.map_async(wgpu::MapMode::Read, .., move |result| {
        let _ = send.send(result);
    });
    device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(Duration::from_secs(5)),
    })?;
    receive.recv_timeout(Duration::from_secs(5))??;
    let bytes = buffer.get_mapped_range(..)?;
    let pixel = |x: usize, y: usize| &bytes[(y * extent.width as usize + x) * 4..][..4];
    assert_eq!(pixel(10, 8), [220, 245, 255, 255], "glyph not rendered");
    assert_eq!(
        pixel(900, 100),
        [0, 0, 0, 255],
        "overlay touched outside its rectangle"
    );
    assert!(
        (6..=8).contains(&pixel(0, 0)[0]),
        "panel background not blended"
    );
    if let Some(path) = std::env::args().nth(1) {
        image::save_buffer(
            path,
            &bytes,
            extent.width,
            extent.height,
            image::ColorType::Rgba8,
        )?;
    }
    drop(bytes);
    buffer.unmap();
    println!("PASS overlay glyph/background/outside pixels");
    Ok(())
}
