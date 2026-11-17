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
#[derive(Default)]
pub struct Snapshot {
    pub state: String,
    pub position: f32,
    pub decoded: i32,
    pub dropped: i32,
    pub error: String,
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
}
pub struct Video {
    shared: Arc<(Mutex<State>, Condvar)>,
    child: Arc<Mutex<Option<Child>>>,
    worker: Option<JoinHandle<()>>,
}
impl Video {
    pub fn new(path: &str, decoder: &str, looping: bool, rate: f32) -> Result<Self, String> {
        if !Path::new(path).is_file() {
            return Err("video file does not exist".into());
        }
        if !Path::new(decoder).is_absolute() || !Path::new(decoder).is_file() {
            return Err("supply an absolute path to an installed FFmpeg executable".into());
        }
        if !rate.is_finite() || !(0.1..=4.).contains(&rate) {
            return Err("video rate must be 0.1..4".into());
        }
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
            }),
            Condvar::new(),
        ));
        let child = Arc::new(Mutex::new(None));
        let (s, c, p, d) = (
            shared.clone(),
            child.clone(),
            path.to_owned(),
            decoder.to_owned(),
        );
        let worker = thread::Builder::new()
            .name("hydra-video".into())
            .spawn(move || run(s, c, &p, &d))
            .map_err(|e| e.to_string())?;
        Ok(Self {
            shared,
            child,
            worker: Some(worker),
        })
    }
    pub fn control(&self, command: &str, value: f32) -> Result<(), String> {
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
    }
}
fn run(
    shared: Arc<(Mutex<State>, Condvar)>,
    child: Arc<Mutex<Option<Child>>>,
    path: &str,
    decoder: &str,
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
        cmd.args([
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-threads",
            "1",
            "-filter_threads",
            "1",
            "-protocol_whitelist",
            "file,pipe",
        ]);
        if looping {
            cmd.args(["-stream_loop", "-1"]);
        }
        cmd.args(["-ss",&position.to_string(),"-i",path,"-map","0:v:0","-an","-sn","-dn","-vf","fps=30,scale=640:360:force_original_aspect_ratio=decrease,pad=640:360:(ow-iw)/2:(oh-ih)/2","-pix_fmt","rgba","-threads","1","-f","rawvideo","pipe:1"]).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
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
        let diagnostic = thread::Builder::new()
            .name("hydra-video-stderr".into())
            .spawn(move || {
                let mut tail = Vec::new();
                let mut bytes = [0u8; 1024];
                while let Ok(n) = stderr.read(&mut bytes) {
                    if n == 0 {
                        break;
                    }
                    tail.extend_from_slice(&bytes[..n]);
                    if tail.len() > 2048 {
                        tail.drain(..tail.len() - 2048);
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
            if s.ready.is_some() {
                s.dropped = s.dropped.saturating_add(1);
            }
            s.position = position + index as f32 / 30.;
            s.ready = Some(Arc::new(
                image::RgbaImage::from_raw(WIDTH, HEIGHT, bytes).unwrap(),
            ));
            index = index.saturating_add(1);
        }
        let status = {
            let mut c = child.lock().unwrap();
            let mut c = c.take().unwrap();
            if !eof {
                let _ = c.kill();
            }
            c.wait()
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
            if !matches!(status,Ok(status) if status.success()) {
                s.error = format!("decoder failed: {detail}");
            }
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    fn until(video: &Video, predicate: impl Fn(&Snapshot) -> bool) -> Snapshot {
        let deadline = Instant::now() + Duration::from_secs(5);
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
