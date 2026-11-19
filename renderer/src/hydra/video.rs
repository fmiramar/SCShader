//! Opt-in local FFmpeg subprocess. All decode, pacing and process waits are off the GPU thread.
use std::{
    io::Read,
    path::Path,
    process::{Child, Command, Stdio},
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
pub const WIDTH: u32 = 640;
pub const HEIGHT: u32 = 360;
const FPS: f64 = 30.;
#[derive(Clone)]
enum Input {
    File(String),
    Capture(super::capture::Device),
}
#[derive(Default)]
pub struct Snapshot {
    pub state: String,
    pub position: f32,
    pub decoded: i32,
    pub dropped: i32,
    pub error: String,
    pub kind: String,
    pub input_width: i32,
    pub input_height: i32,
}
struct State {
    stop: bool,
    version: u64,
    playing: bool,
    looping: bool,
    rate: f32,
    position: f32,
    ended: bool,
    error: String,
    decoded: u64,
    dropped: u64,
    ready: Option<Arc<image::RgbaImage>>,
    kind: &'static str,
    input_width: i32,
    input_height: i32,
    last_frame: Instant,
    permission_wait: bool,
    diagnostic_tail: String,
    helper_pid: i32,
}
pub struct Video {
    shared: Arc<(Mutex<State>, Condvar)>,
    child: Arc<Mutex<Option<Child>>>,
    worker: Option<JoinHandle<()>>,
    watchdog: Option<JoinHandle<()>>,
    live: bool,
    #[cfg(target_os = "macos")]
    camera_socket: Option<super::camera::SocketDirectory>,
}
impl Video {
    pub fn new(path: &str, decoder: &str, looping: bool, rate: f32) -> Result<Self, String> {
        if !Path::new(path).is_file() {
            return Err("video file does not exist".into());
        }
        Self::start(Input::File(path.into()), decoder, looping, rate)
    }
    pub fn capture(device: super::capture::Device, decoder: &str) -> Result<Self, String> {
        if !cfg!(target_os = "macos") {
            return Err("live capture is currently supported only on macOS/AVFoundation".into());
        }
        Self::start(Input::Capture(device), decoder, false, 1.)
    }
    fn start(input: Input, decoder: &str, looping: bool, rate: f32) -> Result<Self, String> {
        if !Path::new(decoder).is_absolute() || !Path::new(decoder).is_file() {
            return Err("supply an absolute path to an installed FFmpeg executable".into());
        }
        if !rate.is_finite() || !(0.1..=4.).contains(&rate) {
            return Err("video rate must be 0.1..4".into());
        }
        let kind = match &input {
            Input::File(_) => "file",
            Input::Capture(d) => d.kind,
        };
        let live = kind != "file";
        #[cfg(target_os = "macos")]
        let camera_socket = if kind == "camera" {
            super::camera::bundle_path()?;
            Some(super::camera::SocketDirectory::new()?)
        } else {
            None
        };
        #[cfg(target_os = "macos")]
        let socket_path = camera_socket.as_ref().map(|s| s.0.clone());
        #[cfg(not(target_os = "macos"))]
        let socket_path = None;
        let shared = Arc::new((
            Mutex::new(State {
                stop: false,
                version: 1,
                playing: true,
                looping,
                rate,
                position: 0.,
                ended: false,
                error: String::new(),
                decoded: 0,
                dropped: 0,
                ready: None,
                kind,
                input_width: 0,
                input_height: 0,
                last_frame: Instant::now(),
                permission_wait: false,
                diagnostic_tail: String::new(),
                helper_pid: 0,
            }),
            Condvar::new(),
        ));
        let child = Arc::new(Mutex::new(None));
        let (s, c, d) = (shared.clone(), child.clone(), decoder.to_owned());
        let worker = thread::Builder::new()
            .name("hydra-video".into())
            .spawn(move || run(s, c, input, &d, socket_path))
            .map_err(|e| e.to_string())?;
        let mut video = Self {
            shared,
            child,
            worker: Some(worker),
            watchdog: None,
            live,
            #[cfg(target_os = "macos")]
            camera_socket,
        };
        if live {
            let (s, c) = (video.shared.clone(), video.child.clone());
            video.watchdog = Some(
                thread::Builder::new()
                    .name("hydra-capture-watchdog".into())
                    .spawn(move || watchdog(s, c))
                    .map_err(|e| e.to_string())?,
            );
        }
        Ok(video)
    }
    pub fn control(&self, command: &str, value: f32) -> Result<(), String> {
        if self.live && !["play", "pause"].contains(&command) {
            return Err("live capture supports play/pause; seek/rate/loop are file-only".into());
        }
        let mut s = self.shared.0.lock().unwrap();
        match command {
            "play" if value == 0. => {
                if s.ended {
                    s.position = 0.;
                }
                s.playing = true;
                s.ended = false;
            }
            "pause" if value == 0. => s.playing = false,
            "loop" if value == 0. || value == 1. => s.looping = value == 1.,
            "rate" if value.is_finite() && (0.1..=4.).contains(&value) => s.rate = value,
            "seek" if value.is_finite() && (0.0..=86400.).contains(&value) => {
                s.position = value;
                s.ended = false;
            }
            _ => return Err("invalid video control/value".into()),
        }
        s.version = s.version.wrapping_add(1);
        s.ready = None;
        s.error.clear();
        s.last_frame = Instant::now();
        s.permission_wait = false;
        s.diagnostic_tail.clear();
        s.helper_pid = 0;
        // Interrupt a pipe read; the worker owns wait/reap and restart.
        if let Some(child) = self.child.lock().unwrap().as_mut() {
            let _ = child.kill();
        }
        self.shared.1.notify_all();
        Ok(())
    }
    pub fn take(&self) -> Option<Arc<image::RgbaImage>> {
        self.shared.0.lock().unwrap().ready.take()
    }
    pub fn snapshot(&self) -> Snapshot {
        let s = self.shared.0.lock().unwrap();
        Snapshot {
            state: if !s.error.is_empty() {
                "error"
            } else if s.ended {
                "ended"
            } else if !s.playing {
                "paused"
            } else if s.permission_wait {
                "permission"
            } else if s.decoded == 0 {
                "loading"
            } else {
                "playing"
            }
            .into(),
            position: s.position,
            decoded: s.decoded.min(i32::MAX as u64) as i32,
            dropped: s.dropped.min(i32::MAX as u64) as i32,
            error: s.error.clone(),
            kind: s.kind.into(),
            input_width: s.input_width,
            input_height: s.input_height,
        }
    }
}
impl Drop for Video {
    fn drop(&mut self) {
        {
            let mut s = self.shared.0.lock().unwrap();
            s.stop = true;
            self.shared.1.notify_all();
            if let Some(c) = self.child.lock().unwrap().as_mut() {
                let _ = c.kill();
            }
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        if let Some(worker) = self.watchdog.take() {
            let _ = worker.join();
        }
        #[cfg(target_os = "macos")]
        {
            self.camera_socket = None;
        }
    }
}
fn run(
    shared: Arc<(Mutex<State>, Condvar)>,
    child: Arc<Mutex<Option<Child>>>,
    input: Input,
    decoder: &str,
    socket_path: Option<std::path::PathBuf>,
) {
    loop {
        let (version, position, rate, looping) = {
            let mut s = shared.0.lock().unwrap();
            while !s.stop && !s.playing {
                s = shared.1.wait(s).unwrap();
            }
            if s.stop {
                return;
            }
            (s.version, s.position, s.rate, s.looping)
        };
        let mut cmd = Command::new(decoder);
        let live = matches!(input, Input::Capture(_));
        cmd.args([
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            if live { "info" } else { "error" },
            "-nostats",
            "-threads",
            "1",
            "-filter_threads",
            "1",
            "-protocol_whitelist",
            "file,pipe",
        ]);
        match &input {
            Input::File(path) => {
                if looping {
                    cmd.args(["-stream_loop", "-1"]);
                }
                cmd.args(["-ss", &position.to_string(), "-i", path]);
            }
            Input::Capture(device) => {
                cmd.args([
                    "-f",
                    "avfoundation",
                    "-pixel_format",
                    "bgr0",
                    "-framerate",
                    "30",
                    "-drop_late_frames",
                    "true",
                ]);
                if device.kind == "camera" {
                    cmd.args(["-video_size", "640x480"]);
                }
                cmd.args(["-i", &format!("{}:none", device.name)]);
            }
        }
        cmd.args(["-map","0:v:0","-an","-sn","-dn","-vf","fps=30,scale=640:360:force_original_aspect_ratio=decrease,pad=640:360:(ow-iw)/2:(oh-ih)/2","-pix_fmt","rgba","-threads","1","-f","rawvideo","pipe:1"]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
        #[cfg(target_os = "macos")]
        if let Input::Capture(device) = &input
            && device.kind == "camera"
        {
            let bundle = super::camera::bundle_path().expect("camera bundle preflight");
            let folder = socket_path.as_ref().expect("camera socket preflight");
            // The socket path is removed after each connection; resume recreates its owned directory.
            if !folder.exists() {
                let _ = std::fs::create_dir(folder);
            }
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(folder, std::fs::Permissions::from_mode(0o700));
            let executable = std::env::var_os("SCSHADER_CAMERA_BRIDGE")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::env::current_exe().expect("renderer executable"));
            cmd = Command::new(executable);
            cmd.arg("--camera-bridge")
                .arg(bundle)
                .arg(&device.name)
                .arg(folder)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
        }
        #[cfg(not(target_os = "macos"))]
        let _ = &socket_path;
        let mut process = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let mut s = shared.0.lock().unwrap();
                if s.version == version {
                    s.error = format!("could not start decoder: {e}");
                    s.playing = false;
                }
                continue;
            }
        };
        let mut stdout = process.stdout.take().unwrap();
        let mut stderr = process.stderr.take().unwrap();
        let metadata = shared.clone();
        let diagnostic = thread::Builder::new()
            .name("hydra-video-stderr".into())
            .spawn(move || {
                let mut tail = Vec::new();
                let mut bytes = [0u8; 1024];
                let mut lines = String::new();
                let mut input_header = true;
                while let Ok(n) = stderr.read(&mut bytes) {
                    if n == 0 {
                        break;
                    }
                    tail.extend_from_slice(&bytes[..n]);
                    if live && input_header {
                        lines.push_str(&String::from_utf8_lossy(&bytes[..n]));
                        while let Some(end) = lines.find('\n') {
                            let line = lines.drain(..=end).collect::<String>();
                            if line.starts_with("Output #") {
                                input_header = false;
                                break;
                            }
                            let mut s = metadata.0.lock().unwrap();
                            if s.version == version {
                                if let Some(state) =
                                    line.trim().strip_prefix("SCSHADER_CAMERA_PERMISSION ")
                                {
                                    s.permission_wait = state == "waiting";
                                    s.last_frame = Instant::now();
                                }
                                if let Some(pid) = line
                                    .trim()
                                    .strip_prefix("SCSHADER_CAMERA_PID ")
                                    .and_then(|v| v.parse::<i32>().ok())
                                {
                                    s.helper_pid = pid;
                                }
                            }
                            drop(s);
                            if let Some((w, h)) = super::capture::input_dimensions(&line) {
                                let mut s = metadata.0.lock().unwrap();
                                if s.version == version {
                                    s.input_width = w;
                                    s.input_height = h;
                                }
                            }
                        }
                        if lines.len() > 8192 {
                            lines.clear();
                        }
                    }
                    if tail.len() > 2048 {
                        tail.drain(..tail.len() - 2048);
                    }
                    if live {
                        let mut s = metadata.0.lock().unwrap();
                        if s.version == version {
                            let mut text = String::from_utf8_lossy(&tail).into_owned();
                            text.truncate(text.floor_char_boundary(1024));
                            s.diagnostic_tail = text;
                        }
                    }
                }
                String::from_utf8_lossy(&tail).into_owned()
            });
        let diagnostic = match diagnostic {
            Ok(worker) => worker,
            Err(e) => {
                let _ = process.kill();
                let _ = process.wait();
                let mut s = shared.0.lock().unwrap();
                if s.version == version {
                    s.error = format!("could not start decoder diagnostic worker: {e}");
                    s.playing = false;
                }
                continue;
            }
        };
        {
            let s = shared.0.lock().unwrap();
            let mut c = child.lock().unwrap();
            if s.stop || s.version != version {
                let _ = process.kill();
            }
            *c = Some(process);
        }
        let anchor = Instant::now();
        let mut index = 0u64;
        let mut eof = false;
        loop {
            let mut bytes = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
            if stdout.read_exact(&mut bytes).is_err() {
                eof = true;
                break;
            }
            let due = anchor + Duration::from_secs_f64(index as f64 / (FPS * f64::from(rate)));
            let mut s = shared.0.lock().unwrap();
            while !s.stop && s.version == version && Instant::now() < due {
                s = shared
                    .1
                    .wait_timeout(s, due.saturating_duration_since(Instant::now()))
                    .unwrap()
                    .0;
            }
            if s.stop || s.version != version {
                break;
            }
            s.decoded = s.decoded.saturating_add(1);
            s.last_frame = Instant::now();
            if s.ready.is_some() {
                s.dropped = s.dropped.saturating_add(1);
            }
            s.position = if live {
                anchor.elapsed().as_secs_f32()
            } else {
                position + index as f32 / 30.
            };
            s.ready = Some(Arc::new(
                image::RgbaImage::from_raw(WIDTH, HEIGHT, bytes).unwrap(),
            ));
            index = index.saturating_add(1);
        }
        let status = {
            let mut c = child.lock().unwrap().take().unwrap();
            if !eof {
                let _ = c.kill();
            }
            // Do not hold the child mutex while reaping. A decoder that closes
            // stdout without exiting gets a short grace period, then is killed.
            let deadline = Instant::now() + Duration::from_secs(1);
            loop {
                match c.try_wait() {
                    Ok(Some(status)) => break Ok(status),
                    Err(error) => {
                        let _ = c.kill();
                        let _ = c.wait();
                        break Err(error);
                    }
                    Ok(None) => {}
                }
                if Instant::now() >= deadline {
                    let _ = c.kill();
                    break c.wait();
                }
                thread::sleep(Duration::from_millis(5));
            }
        };
        let mut detail = diagnostic.join().unwrap_or_default();
        detail.truncate(detail.floor_char_boundary(1024));
        let mut s = shared.0.lock().unwrap();
        if s.stop {
            return;
        }
        if s.version == version {
            s.playing = false;
            s.ended = true;
            if s.error.is_empty() && (live || !matches!(status,Ok(status) if status.success())) {
                s.error = if live {
                    format!(
                        "capture stopped/disconnected or unavailable. Check macOS Camera or Screen Recording permission for the launching app, then enumerate/reopen. {detail}"
                    )
                } else {
                    format!("decoder failed: {detail}")
                };
            } else if live && !detail.is_empty() && !s.error.contains(&detail) {
                s.error.push_str(" Decoder: ");
                s.error.push_str(&detail);
            }
        }
    }
}

fn watchdog(shared: Arc<(Mutex<State>, Condvar)>, child: Arc<Mutex<Option<Child>>>) {
    let mut s = shared.0.lock().unwrap();
    while !s.stop {
        let timeout = if s.permission_wait { 60 } else { 10 };
        if s.playing && s.last_frame.elapsed() > Duration::from_secs(timeout) {
            let owner = if s.kind == "camera" {
                "SCShader Camera permission prompt"
            } else {
                "Screen Recording permission for the launching app"
            };
            s.error = format!(
                "capture delivered no frame for {timeout} seconds; inspect the device or {owner}. {}",
                s.diagnostic_tail
            );
            s.playing = false;
            if let Some(child) = child.lock().unwrap().as_mut() {
                let _ = child.kill();
            }
        }
        s = shared
            .1
            .wait_timeout(s, Duration::from_millis(250))
            .unwrap()
            .0;
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    fn until(video: &Video, predicate: impl Fn(&Snapshot) -> bool) -> Snapshot {
        let deadline = Instant::now() + Duration::from_secs(if video.live { 75 } else { 5 });
        loop {
            let s = video.snapshot();
            assert!(s.error.is_empty(), "{}", s.error);
            if predicate(&s) {
                return s;
            }
            assert!(
                Instant::now() < deadline,
                "video state timeout: {} at {}",
                s.state,
                s.position
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn pid(video: &Video) -> u32 {
        let id = video.child.lock().unwrap().as_ref().map(Child::id);
        id.expect("decoder is running")
    }
    fn reaped(pid: u32) {
        assert!(
            !Command::new("/bin/kill")
                .args(["-0", &pid.to_string()])
                .stderr(Stdio::null())
                .status()
                .unwrap()
                .success(),
            "decoder PID {pid} survived cleanup"
        );
    }
    #[cfg(target_os = "macos")]
    fn helper_exited(pid: i32) {
        if pid <= 1 {
            return;
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        while Command::new("/bin/kill")
            .args(["-0", &pid.to_string()])
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
        {
            assert!(
                Instant::now() < deadline,
                "native camera helper {pid} survived closing its controller"
            );
            thread::sleep(Duration::from_millis(20));
        }
    }
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "authorized real camera and screen capture; SCSHADER_FFMPEG"]
    fn native_capture_lifecycle() {
        let decoder = std::env::var("SCSHADER_FFMPEG").unwrap();
        let discovery = super::super::capture::Discovery::new(&decoder).unwrap();
        let deadline = Instant::now() + Duration::from_secs(6);
        while discovery.snapshot().state == "loading" {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(20));
        }
        let devices = discovery.snapshot();
        assert_eq!(devices.state, "ready", "{}", devices.error);
        for kind in ["camera", "screen"] {
            let selected = devices
                .devices
                .iter()
                .find(|d| d.kind == kind)
                .expect("native capture device");
            assert!(
                discovery
                    .select(
                        if kind == "camera" { "screen" } else { "camera" },
                        selected.index
                    )
                    .is_err()
            );
            let video =
                Video::capture(discovery.select(kind, selected.index).unwrap(), &decoder).unwrap();
            let state = until(&video, |s| {
                s.decoded > 2 && s.input_width > 0 && s.input_height > 0
            });
            println!(
                "{kind}: native {}x{}, RGBA {}x{}, frames {}",
                state.input_width, state.input_height, WIDTH, HEIGHT, state.decoded
            );
            let first_pid = pid(&video);
            let first_helper = video.shared.0.lock().unwrap().helper_pid;
            if kind == "camera" {
                assert!(first_helper > 1);
            }
            let warmup = Instant::now() + Duration::from_secs(5);
            // Native cameras can deliver black exposure/warm-up frames at startup.
            // Keep the visible-image assertion and require a nonblack frame within a bound.
            loop {
                if let Some(image) = video.take() {
                    assert_eq!(image.dimensions(), (WIDTH, HEIGHT));
                    if image.pixels().any(|p| p.0[..3].iter().any(|&v| v > 20)) {
                        break;
                    }
                }
                assert!(
                    Instant::now() < warmup,
                    "capture remained entirely black after five seconds of warm-up"
                );
                assert!(
                    video.snapshot().error.is_empty(),
                    "{}",
                    video.snapshot().error
                );
                thread::sleep(Duration::from_millis(20));
            }
            assert!(video.control("seek", 1.).is_err());
            video.control("pause", 0.).unwrap();
            let paused = video.snapshot().decoded;
            thread::sleep(Duration::from_millis(100));
            assert_eq!(video.snapshot().decoded, paused);
            video.control("play", 0.).unwrap();
            until(&video, |s| s.decoded > paused + 2);
            reaped(first_pid);
            helper_exited(first_helper);
            let final_pid = pid(&video);
            let final_helper = video.shared.0.lock().unwrap().helper_pid;
            if kind == "screen" {
                // Stop only this test-owned capture process to simulate a stalled source.
                assert!(
                    Command::new("/bin/kill")
                        .args(["-STOP", &final_pid.to_string()])
                        .status()
                        .unwrap()
                        .success()
                );
                // Advance the timeout state without making the live check wait ten seconds.
                thread::sleep(Duration::from_millis(100));
                video.shared.0.lock().unwrap().last_frame =
                    Instant::now() - Duration::from_secs(11);
                video.shared.1.notify_all();
                let deadline = Instant::now() + Duration::from_secs(3);
                while video.snapshot().state != "error" {
                    assert!(Instant::now() < deadline);
                    thread::sleep(Duration::from_millis(20));
                }
                assert!(video.snapshot().error.contains("permission"));
            }
            drop(video);
            reaped(final_pid);
            helper_exited(final_helper);
        }
    }
    #[test]
    #[ignore = "opt-in native FFmpeg check: SCSHADER_TEST_VIDEO and SCSHADER_FFMPEG"]
    fn native_video_lifecycle() {
        let clip = std::env::var("SCSHADER_TEST_VIDEO").expect("one-second local clip");
        let decoder = std::env::var("SCSHADER_FFMPEG").expect("absolute FFmpeg executable");
        let video = Video::new(&clip, &decoder, true, 1.).unwrap();
        until(&video, |s| s.decoded > 0);
        let first_pid = pid(&video);
        // Deliberately do not consume: the one-entry queue replaces old frames.
        let state = until(&video, |s| s.position > 1.2);
        assert!(state.dropped > 20);
        assert!(state.decoded < 90, "decoder ran ahead of presentation time");
        let frame = video.take().unwrap();
        assert_eq!(frame.dimensions(), (WIDTH, HEIGHT));
        assert!(video.take().is_none());
        video.control("pause", 0.).unwrap();
        let paused = video.snapshot().decoded;
        thread::sleep(Duration::from_millis(100));
        assert_eq!(video.snapshot().decoded, paused);
        assert_eq!(video.snapshot().state, "paused");
        video.control("play", 0.).unwrap();
        until(&video, |s| s.decoded > paused && s.position > 1.2);
        video.control("pause", 0.).unwrap();
        video.control("seek", 0.2).unwrap();
        video.control("rate", 2.).unwrap();
        video.control("loop", 0.).unwrap();
        video.control("play", 0.).unwrap();
        until(&video, |s| s.state == "ended");
        reaped(first_pid);
        let last = video.take().unwrap();
        assert!(last.as_raw().iter().any(|&b| b != 0));
        assert!(video.snapshot().position < 1.1);
        let ended_count = video.snapshot().decoded;
        video.control("loop", 1.).unwrap();
        video.control("play", 0.).unwrap();
        until(&video, |s| {
            s.decoded > ended_count && s.state == "playing" && s.position < 0.5
        });
        let final_pid = pid(&video);
        let start = Instant::now();
        drop(video);
        assert!(start.elapsed() < Duration::from_secs(2));
        reaped(final_pid);
        let bad = Video::new(&clip, "/usr/bin/false", true, 1.).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while bad.snapshot().state != "error" {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        assert!(bad.snapshot().error.len() <= 1050);
    }
}
