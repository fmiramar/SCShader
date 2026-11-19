//! Opt-in presentation export: one readback buffer, one queued frame, one writer.
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread,
    time::{Duration, Instant},
};

pub const MAX_DIMENSION: u32 = 2048;
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub state: String,
    pub path: String,
    pub error: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub written: u64,
    pub dropped: u64,
    pub buffers: i32,
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            state: "idle".into(),
            path: String::new(),
            error: String::new(),
            width: 0,
            height: 0,
            fps: 0,
            written: 0,
            dropped: 0,
            buffers: 0,
        }
    }
}
fn dimensions(width: u32, height: u32, fps: u32) -> Result<u32, String> {
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err("export dimensions must be 1..2048 pixels; use setResolution first".into());
    }
    if fps > 0 && (fps > 60 || !width.is_multiple_of(2) || !height.is_multiple_of(2)) {
        return Err("MP4 requires even dimensions and 1..60 FPS".into());
    }
    Ok((width * 4).div_ceil(256) * 256)
}
fn destination(path: &str, video: bool) -> Result<(), String> {
    let path = Path::new(path);
    if !path.is_absolute()
        || path.exists()
        || !path.parent().is_some_and(Path::is_dir)
        || path.extension().and_then(|s| s.to_str()) != Some(if video { "mp4" } else { "png" })
    {
        return Err("choose a new absolute .png/.mp4 path in an existing directory; export never overwrites files".into());
    }
    Ok(())
}
struct Readback {
    presenter: super::output::Presenter,
    texture: wgpu::Texture,
    buffer: wgpu::Buffer,
    padded: u32,
    mapping: Option<Receiver<Result<(), wgpu::BufferAsyncError>>>,
    submitted: bool,
}
impl Readback {
    fn new(device: &wgpu::Device, width: u32, height: u32, padded: u32) -> Result<Self, String> {
        let scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Hydra export presentation"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Hydra reusable export readback"),
            size: u64::from(padded) * u64::from(height),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let presenter = super::output::Presenter::new(device, wgpu::TextureFormat::Rgba8Unorm);
        if let Some(error) = pollster::block_on(scope.pop()) {
            return Err(error.to_string());
        }
        Ok(Self {
            presenter,
            texture,
            buffer,
            padded,
            mapping: None,
            submitted: false,
        })
    }
}
type Encoder = Arc<Mutex<Option<Child>>>;
struct Writer {
    input: Option<SyncSender<Vec<u8>>>,
    result: Receiver<Result<(), String>>,
    progress: Arc<AtomicU64>,
    encoder: Encoder,
    finishing: Option<Instant>,
}
fn kill(encoder: &Encoder) {
    if let Some(child) = encoder.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
        let _ = child.kill();
    }
}
fn wait_encoder(encoder: &Encoder) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let result = encoder
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_mut()
            .unwrap()
            .try_wait()
            .map_err(|e| e.to_string())?;
        if let Some(status) = result {
            return if status.success() {
                Ok(())
            } else {
                Err(format!(
                    "FFmpeg failed ({status}); verify the supplied encoder supports libx264 and MP4"
                ))
            };
        }
        if Instant::now() >= deadline {
            kill(encoder);
            return Err("encoder finalization exceeded five seconds".into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}
#[derive(Default)]
pub struct Export {
    status: Snapshot,
    readback: Option<Readback>,
    writer: Option<Writer>,
    next: Option<Instant>,
    started: Option<Instant>,
}
impl Export {
    pub fn busy(&self) -> bool {
        self.writer.is_some()
    }
    pub fn snapshot(&self) -> Snapshot {
        self.status.clone()
    }
    pub fn texture_bytes(&self) -> u64 {
        self.readback.as_ref().map_or(0, |r| {
            u64::from(r.texture.width()) * u64::from(r.texture.height()) * 4
        })
    }
    pub fn start(
        &mut self,
        device: &wgpu::Device,
        path: &str,
        encoder: &str,
        width: u32,
        height: u32,
        fps: u32,
    ) -> Result<(), String> {
        if self.busy() {
            return Err("an export is active; stop it and wait for finished/error status".into());
        }
        let padded = dimensions(width, height, fps)?;
        destination(path, fps > 0)?;
        if fps > 0 && (!Path::new(encoder).is_absolute() || !Path::new(encoder).is_file()) {
            return Err("recording requires an explicit absolute FFmpeg executable path".into());
        }
        let readback = Readback::new(device, width, height, padded)?;
        let process: Encoder = Arc::new(Mutex::new(None));
        let mut pipe = None;
        let mut png: Option<File> = None;
        if fps == 0 {
            png = Some(
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)
                    .map_err(|e| e.to_string())?,
            );
        } else {
            let mut child = Command::new(encoder)
                .args([
                    "-nostdin",
                    "-hide_banner",
                    "-loglevel",
                    "error",
                    "-f",
                    "rawvideo",
                    "-pixel_format",
                    "rgba",
                    "-video_size",
                    &format!("{width}x{height}"),
                    "-framerate",
                    &fps.to_string(),
                    "-i",
                    "pipe:0",
                    "-an",
                    "-c:v",
                    "libx264",
                    "-preset",
                    "ultrafast",
                    "-crf",
                    "20",
                    "-pix_fmt",
                    "yuv420p",
                    "-n",
                    path,
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| e.to_string())?;
            pipe = child.stdin.take();
            *process.lock().unwrap() = Some(child);
        }
        let (input, frames) = mpsc::sync_channel::<Vec<u8>>(1);
        let (result, receiver) = mpsc::channel();
        let progress = Arc::new(AtomicU64::new(0));
        let count = progress.clone();
        let child = process.clone();
        if let Err(error) = thread::Builder::new()
            .name("hydra-export-writer".into())
            .spawn(move || {
                let outcome = (|| {
                    for bytes in frames {
                        if let Some(file) = png.as_mut() {
                            image::codecs::png::PngEncoder::new(file)
                                .write_image(&bytes, width, height, image::ExtendedColorType::Rgba8)
                                .map_err(|e| e.to_string())?;
                        } else {
                            pipe.as_mut()
                                .unwrap()
                                .write_all(&bytes)
                                .map_err(|e| format!("encoder pipe failed: {e}"))?;
                        }
                        count.fetch_add(1, Ordering::Relaxed);
                    }
                    drop(pipe);
                    if fps > 0 {
                        wait_encoder(&child)?;
                    }
                    if count.load(Ordering::Relaxed) == 0 {
                        return Err("export ended without a captured frame".into());
                    }
                    Ok(())
                })();
                if outcome.is_err() {
                    kill(&child);
                }
                // Reap even on failed writes or forced cancellation.
                if let Some(mut process) = child.lock().unwrap_or_else(|e| e.into_inner()).take() {
                    let _ = process.wait();
                }
                let _ = result.send(outcome);
            })
        {
            kill(&process);
            if let Some(mut child) = process.lock().unwrap_or_else(|e| e.into_inner()).take() {
                let _ = child.wait();
            }
            return Err(error.to_string());
        }
        self.status = Snapshot {
            state: if fps == 0 { "screenshot" } else { "recording" }.into(),
            path: path.into(),
            width,
            height,
            fps,
            buffers: 1,
            ..Default::default()
        };
        self.readback = Some(readback);
        self.writer = Some(Writer {
            input: Some(input),
            result: receiver,
            progress,
            encoder: process,
            finishing: None,
        });
        self.next = Some(Instant::now());
        self.started = self.next;
        Ok(())
    }
    pub fn stop(&mut self) -> Result<(), String> {
        let writer = self.writer.as_mut().ok_or("no export is active")?;
        if writer.finishing.is_none() {
            writer.finishing = Some(Instant::now());
            writer.input = None;
            self.status.state = "stopping".into();
            self.next = None;
        }
        Ok(())
    }
    pub fn poll(&mut self) {
        if self.status.fps == 0
            && self
                .started
                .is_some_and(|t| t.elapsed() > Duration::from_secs(10))
            && self.writer.as_ref().is_some_and(|w| w.finishing.is_none())
        {
            self.status.error = "screenshot did not complete within ten seconds".into();
            let _ = self.stop();
        }
        if self
            .started
            .is_some_and(|t| t.elapsed() >= Duration::from_secs(600))
            && self.next.is_some()
        {
            let _ = self.stop();
        }
        let Some(writer) = self.writer.as_mut() else {
            return;
        };
        self.status.written = writer.progress.load(Ordering::Relaxed);
        if writer
            .finishing
            .is_some_and(|t| t.elapsed() > Duration::from_millis(5500))
        {
            kill(&writer.encoder);
        }
        let outcome = match writer.result.try_recv() {
            Ok(outcome) => Some(outcome),
            Err(mpsc::TryRecvError::Disconnected) => Some(Err("export worker disconnected".into())),
            Err(mpsc::TryRecvError::Empty) => None,
        };
        if let Some(outcome) = outcome {
            self.status.written = writer.progress.load(Ordering::Relaxed);
            let outcome = outcome.and_then(|()| {
                if self.status.error.is_empty() {
                    Ok(())
                } else {
                    Err(self.status.error.clone())
                }
            });
            if outcome.is_err() {
                kill(&writer.encoder);
            }
            self.status.state = if outcome.is_ok() { "finished" } else { "error" }.into();
            self.status.error = outcome.err().unwrap_or_default();
            self.status
                .error
                .truncate(self.status.error.floor_char_boundary(512));
            self.writer = None;
            self.readback = None;
            self.next = None;
            self.status.buffers = 0;
            return;
        }
        let Some(readback) = self.readback.as_mut() else {
            return;
        };
        let ready = readback.mapping.as_ref().and_then(|r| match r.try_recv() {
            Ok(result) => Some(result.map_err(|e| e.to_string())),
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err("readback callback disconnected".into()))
            }
            Err(mpsc::TryRecvError::Empty) => None,
        });
        if let Some(ready) = ready {
            readback.mapping = None;
            let bytes = ready.and_then(|()| {
                let mapped = readback
                    .buffer
                    .get_mapped_range(..)
                    .map_err(|e| e.to_string())?;
                let bytes = mapped
                    .chunks_exact(readback.padded as usize)
                    .flat_map(|row| row[..self.status.width as usize * 4].iter().copied())
                    .collect::<Vec<_>>();
                drop(mapped);
                Ok(bytes)
            });
            readback.buffer.unmap();
            match bytes {
                Ok(bytes) => {
                    if let Some(input) = &writer.input {
                        if input.try_send(bytes).is_err() {
                            self.status.dropped = self.status.dropped.saturating_add(1);
                        }
                        if self.status.fps == 0 {
                            let _ = self.stop();
                        }
                    }
                }
                Err(error) => {
                    self.status.error = error;
                    kill(&writer.encoder);
                    let _ = self.stop();
                }
            }
        }
    }
    pub fn encode(
        &mut self,
        device: &wgpu::Device,
        outputs: &super::outputs::Outputs,
        mode: i32,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let now = Instant::now();
        let Some(next) = self.next else {
            return;
        };
        if now < next {
            return;
        }
        let period = Duration::from_secs_f64(1.0 / f64::from(self.status.fps.max(1)));
        if self.status.fps > 0 {
            let missed = (now.duration_since(next).as_nanos() / period.as_nanos())
                .min(u128::from(u32::MAX - 1)) as u32;
            self.status.dropped = self.status.dropped.saturating_add(u64::from(missed));
            self.next = Some(next + period * (missed + 1));
        } else {
            self.next = None;
        }
        let readback = self.readback.as_mut().unwrap();
        if readback.mapping.is_some() || readback.submitted {
            self.status.dropped = self.status.dropped.saturating_add(1);
            return;
        }
        let view = readback.texture.create_view(&Default::default());
        // Export uses the same presenter and selected/quadrant arrangement as the window.
        outputs.encode_export(
            device,
            &readback.presenter,
            encoder,
            &view,
            mode,
            (self.status.width, self.status.height),
        );
        encoder.copy_texture_to_buffer(
            readback.texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback.buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(readback.padded),
                    rows_per_image: Some(self.status.height),
                },
            },
            readback.texture.size(),
        );
        readback.submitted = true;
    }
    pub fn submitted(&mut self) {
        let Some(readback) = self.readback.as_mut() else {
            return;
        };
        if std::mem::take(&mut readback.submitted) {
            let (send, receive) = mpsc::sync_channel(1);
            readback
                .buffer
                .map_async(wgpu::MapMode::Read, .., move |result| {
                    let _ = send.try_send(result);
                });
            readback.mapping = Some(receive);
        }
    }
}
impl Drop for Export {
    fn drop(&mut self) {
        if let Some(writer) = &mut self.writer {
            writer.input = None;
            kill(&writer.encoder);
        }
    }
}
use image::ImageEncoder;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn padding_limits_and_destination_guards() {
        assert_eq!(dimensions(319, 181, 0).unwrap(), 1280);
        assert!(dimensions(319, 181, 30).is_err());
        assert!(dimensions(2049, 10, 0).is_err());
        assert!(dimensions(320, 180, 61).is_err());
        assert!(destination("relative.png", false).is_err());
        assert!(destination("/missing-directory/movie.mp4", true).is_err());
    }
}
