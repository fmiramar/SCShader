//! Opt-in macOS AVFoundation discovery via the caller's FFmpeg executable.
use std::{
    io::Read,
    path::Path,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Device {
    pub kind: &'static str,
    pub index: i32,
    pub name: String,
}
#[derive(Clone, Default)]
pub struct Devices {
    pub state: String,
    pub devices: Vec<Device>,
    pub error: String,
}
pub struct Discovery {
    pub decoder: String,
    result: Arc<Mutex<Devices>>,
    stop: Arc<AtomicBool>,
    child: Arc<Mutex<Option<Child>>>,
    worker: Option<JoinHandle<()>>,
}
impl Discovery {
    pub fn new(decoder: &str) -> Result<Self, String> {
        if !cfg!(target_os = "macos") {
            return Err("capture is currently supported only on macOS/AVFoundation".into());
        }
        if !Path::new(decoder).is_absolute() || !Path::new(decoder).is_file() {
            return Err("supply an absolute path to FFmpeg with AVFoundation support".into());
        }
        let result = Arc::new(Mutex::new(Devices {
            state: "loading".into(),
            ..Default::default()
        }));
        let stop = Arc::new(AtomicBool::new(false));
        let child = Arc::new(Mutex::new(None));
        let (r, s, c, d) = (
            result.clone(),
            stop.clone(),
            child.clone(),
            decoder.to_owned(),
        );
        let worker = thread::Builder::new()
            .name("hydra-capture-discovery".into())
            .spawn(move || {
                let outcome = discover(&d, &s, &c);
                let mut r = r.lock().unwrap();
                match outcome {
                    Ok(devices) => {
                        r.state = "ready".into();
                        r.devices = devices;
                    }
                    Err(error) => {
                        r.state = "error".into();
                        r.error = error;
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            decoder: decoder.into(),
            result,
            stop,
            child,
            worker: Some(worker),
        })
    }
    pub fn snapshot(&self) -> Devices {
        self.result.lock().unwrap().clone()
    }
    pub fn select(&self, kind: &str, index: i32) -> Result<Device, String> {
        let s = self.snapshot();
        if s.state != "ready" {
            return Err(format!(
                "capture device discovery is {}: {}",
                s.state, s.error
            ));
        }
        let device = s
            .devices
            .iter()
            .find(|d| d.index == index && d.kind == kind)
            .ok_or("selected capture device is absent or has the wrong kind; enumerate again")?;
        if s.devices.iter().filter(|d| d.name == device.name).count() != 1
            || device.name.contains(':')
            || device.name.starts_with(|c: char| c.is_ascii_digit())
            || device.name.starts_with("default")
            || device.name.starts_with("none")
        {
            return Err("capture device name is ambiguous for this FFmpeg adapter".into());
        }
        Ok(device.clone())
    }
}
impl Drop for Discovery {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(child) = self.child.lock().unwrap().as_mut() {
            let _ = child.kill();
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
fn discover(
    decoder: &str,
    stop: &AtomicBool,
    child: &Mutex<Option<Child>>,
) -> Result<Vec<Device>, String> {
    let mut process = Command::new(decoder)
        .args([
            "-nostdin",
            "-hide_banner",
            "-f",
            "avfoundation",
            "-list_devices",
            "true",
            "-i",
            "",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not start capture discovery: {e}"))?;
    let mut stderr = process.stderr.take().unwrap();
    let reader = thread::Builder::new()
        .name("hydra-capture-list".into())
        .spawn(move || {
            let mut output = Vec::new();
            let mut bytes = [0; 1024];
            while let Ok(n) = stderr.read(&mut bytes) {
                if n == 0 {
                    break;
                }
                // Keep a bounded prefix, but continue draining the child's pipe.
                let remaining = 16384usize.saturating_sub(output.len());
                output.extend_from_slice(&bytes[..n.min(remaining)]);
            }
            String::from_utf8_lossy(&output).into_owned()
        });
    let reader = match reader {
        Ok(r) => r,
        Err(e) => {
            let _ = process.kill();
            let _ = process.wait();
            return Err(e.to_string());
        }
    };
    {
        let mut c = child.lock().unwrap();
        if stop.load(Ordering::Relaxed) {
            let _ = process.kill();
        }
        *c = Some(process);
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut failed = None;
    loop {
        let mut c = child.lock().unwrap();
        match c.as_mut().unwrap().try_wait() {
            Ok(Some(_)) => break,
            Err(e) => {
                failed = Some(e.to_string());
                break;
            }
            Ok(None) => {}
        }
        if stop.load(Ordering::Relaxed) || Instant::now() >= deadline {
            let _ = c.as_mut().unwrap().kill();
            failed = Some(
                "capture discovery stopped/timed out; check FFmpeg and macOS permissions".into(),
            );
            break;
        }
        drop(c);
        thread::sleep(Duration::from_millis(20));
    }
    if let Some(mut c) = child.lock().unwrap().take() {
        let _ = c.kill();
        let _ = c.wait();
    }
    let output = reader.join().unwrap_or_default();
    if let Some(error) = failed {
        return Err(error);
    }
    parse_devices(&output)
}
fn parse_devices(output: &str) -> Result<Vec<Device>, String> {
    let mut devices = Vec::new();
    let mut video = false;
    for line in output.lines() {
        if line.contains("AVFoundation video devices:") {
            video = true;
            continue;
        }
        if line.contains("AVFoundation audio devices:") {
            break;
        }
        if !video {
            continue;
        }
        let Some((_, entry)) = line.split_once("] [") else {
            continue;
        };
        let Some((index, name)) = entry.split_once("] ") else {
            continue;
        };
        let Ok(index) = index.parse::<i32>() else {
            continue;
        };
        let name = name.trim();
        if !(0..=255).contains(&index) || name.is_empty() || name.len() > 512 {
            return Err("unsupported capture device identifier".into());
        }
        if devices.len() == 16 {
            return Err("capture discovery supports at most 16 devices".into());
        }
        if devices.iter().any(|d: &Device| d.index == index) {
            return Err("duplicate capture device index".into());
        }
        let kind = if name
            .strip_prefix("Capture screen ")
            .is_some_and(|v| v.parse::<u32>().is_ok())
        {
            "screen"
        } else {
            "camera"
        };
        devices.push(Device {
            kind,
            index,
            name: name.into(),
        });
    }
    if !video {
        return Err(
            "FFmpeg did not list AVFoundation devices; check the executable/platform".into(),
        );
    }
    Ok(devices)
}
pub fn input_dimensions(line: &str) -> Option<(i32, i32)> {
    if !line.contains("Stream #0:0") || !line.contains("Video:") {
        return None;
    }
    line.split_whitespace().find_map(|word| {
        let (w, h) = word.trim_end_matches(',').split_once('x')?;
        let (w, h) = (w.parse::<i32>().ok()?, h.parse::<i32>().ok()?);
        ((1..=32768).contains(&w) && (1..=32768).contains(&h)).then_some((w, h))
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discovery_distinguishes_camera_screen_and_ignores_audio() {
        let devices=parse_devices("[AVFoundation indev @ x] AVFoundation video devices:\n[AVFoundation indev @ x] [0] Câmera USB\n[AVFoundation indev @ x] [1] Capture screen 0\n[AVFoundation indev @ x] AVFoundation audio devices:\n[AVFoundation indev @ x] [0] Microphone").unwrap();
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].kind, "camera");
        assert_eq!(devices[1].kind, "screen");
        assert!(parse_devices("unknown input format").is_err());
        assert_eq!(
            input_dimensions(
                "Stream #0:0: Video: rawvideo (BGR[0] / 0x524742), bgr0, 2940x1912, 30 fps"
            ),
            Some((2940, 1912))
        );
    }
}
