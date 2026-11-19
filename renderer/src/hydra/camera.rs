//! LaunchServices gives the camera its own app identity and privacy usage declaration.
#[cfg(target_os = "macos")]
mod mac {
    use std::{
        fs,
        io::{self, BufRead, BufReader, Read, Write},
        os::unix::{
            fs::PermissionsExt,
            net::{UnixListener, UnixStream},
        },
        path::{Path, PathBuf},
        process::{Command, Stdio},
        thread,
        time::{Duration, Instant},
    };

    pub fn bundle_path() -> Result<PathBuf, String> {
        let path = if let Some(path) = std::env::var_os("SCSHADER_CAMERA_HELPER") {
            PathBuf::from(path)
        } else {
            std::env::current_exe()
                .map_err(|e| e.to_string())?
                .parent()
                .ok_or("renderer directory unavailable")?
                .join("SCShaderCamera.app")
        };
        if !path.join("Contents/MacOS/SCShaderCamera").is_file()
            || !path.join("Contents/Info.plist").is_file()
        {
            return Err(
                "SCShader Camera helper is missing; install the complete macOS package".into(),
            );
        }
        Ok(path)
    }
    pub struct SocketDirectory(pub PathBuf);
    impl SocketDirectory {
        pub fn new() -> Result<Self, String> {
            let mut random = [0u8; 8];
            fs::File::open("/dev/urandom")
                .and_then(|mut f| f.read_exact(&mut random))
                .map_err(|e| e.to_string())?;
            let folder = Self(
                std::env::temp_dir().join(format!("sc-camera-{:x}", u64::from_ne_bytes(random))),
            );
            fs::create_dir(&folder.0).map_err(|e| e.to_string())?;
            fs::set_permissions(&folder.0, fs::Permissions::from_mode(0o700))
                .map_err(|e| e.to_string())?;
            Ok(folder)
        }
    }
    impl Drop for SocketDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_file(self.0.join("s"));
            let _ = fs::remove_dir(&self.0);
        }
    }
    fn line(reader: &mut impl BufRead) -> Result<String, String> {
        let mut bytes = Vec::new();
        let n = reader
            .take(4097)
            .read_until(b'\n', &mut bytes)
            .map_err(|e| format!("camera helper connection: {e}"))?;
        if n == 0 {
            return Err("camera helper closed before the next frame".into());
        }
        if n > 4096 || bytes.last() != Some(&b'\n') {
            return Err("invalid camera helper header".into());
        }
        String::from_utf8(bytes)
            .map(|s| s.trim_end().into())
            .map_err(|e| e.to_string())
    }
    fn blocking_stream(stream: UnixStream) -> io::Result<UnixStream> {
        // macOS accepts can inherit the listener's O_NONBLOCK flag.
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(Duration::from_secs(75)))?;
        Ok(stream)
    }
    pub fn bridge(bundle: &Path, name: &str, folder: &Path) -> Result<(), String> {
        if !bundle.join("Contents/MacOS/SCShaderCamera").is_file() {
            return Err("camera helper bundle is missing".into());
        }
        let path = folder.join("s");
        let listener =
            UnixListener::bind(&path).map_err(|e| format!("camera private socket: {e}"))?;
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let mut launcher = Command::new("/usr/bin/open")
            .args(["-n", "-a"])
            .arg(bundle)
            .arg("--args")
            .arg("--socket")
            .arg(&path)
            .arg("--camera")
            .arg(name)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| e.to_string())?;
        let deadline = Instant::now() + Duration::from_secs(10);
        let stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => {
                    let _ = launcher.kill();
                    let _ = launcher.wait();
                    return Err(e.to_string());
                }
            }
            if let Some(status) = launcher.try_wait().map_err(|e| e.to_string())?
                && !status.success()
            {
                return Err("macOS could not launch SCShader Camera".into());
            }
            if Instant::now() >= deadline {
                let _ = launcher.kill();
                let _ = launcher.wait();
                return Err("SCShader Camera did not connect within 10 seconds; check the helper app launch".into());
            }
            thread::sleep(Duration::from_millis(20));
        };
        // open exits after dispatching the app. Reap only this owned launcher.
        let _ = launcher.wait();
        drop(listener);
        let _ = fs::remove_file(&path);
        let _ = fs::remove_dir(folder);
        let stream = blocking_stream(stream).map_err(|e| e.to_string())?;
        let mut reader = BufReader::new(stream);
        let hello = line(&mut reader)?;
        let pid = hello
            .strip_prefix("HELLO ")
            .and_then(|s| s.parse::<i32>().ok())
            .filter(|p| *p > 1)
            .ok_or("invalid camera helper greeting")?;
        eprintln!("SCSHADER_CAMERA_PID {pid}");
        let mut ready = false;
        let mut pixels =
            vec![0; (crate::hydra::video::WIDTH * crate::hydra::video::HEIGHT * 4) as usize];
        let mut output = io::stdout().lock();
        loop {
            let message = line(&mut reader)?;
            if let Some(state) = message.strip_prefix("PERMISSION ") {
                if state != "waiting" && state != "authorized" {
                    return Err("invalid camera authorization response".into());
                }
                eprintln!("SCSHADER_CAMERA_PERMISSION {state}");
            } else if let Some(error) = message.strip_prefix("ERROR ") {
                return Err(error.into());
            } else if let Some(size) = message.strip_prefix("READY ") {
                let dims = size
                    .split_whitespace()
                    .map(str::parse::<u32>)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| e.to_string())?;
                if ready || dims.len() != 2 || dims.iter().any(|&v| v == 0 || v > 32768) {
                    return Err("invalid camera input dimensions".into());
                }
                ready = true;
                reader
                    .get_ref()
                    .set_read_timeout(Some(Duration::from_secs(15)))
                    .map_err(|e| e.to_string())?;
                eprintln!(
                    "Stream #0:0: Video: rawvideo, rgba, {}x{}, 30 fps",
                    dims[0], dims[1]
                );
            } else if message == "FRAME" && ready {
                reader.read_exact(&mut pixels).map_err(|e| e.to_string())?;
                output.write_all(&pixels).map_err(|e| e.to_string())?;
            } else {
                return Err("invalid camera helper message".into());
            }
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn helper_headers_are_bounded_and_require_newlines() {
            assert_eq!(line(&mut &b"READY 640 480\n"[..]).unwrap(), "READY 640 480");
            assert!(line(&mut &b"unfinished"[..]).is_err());
            assert!(line(&mut vec![b'x'; 4097].as_slice()).is_err());
        }
        #[test]
        fn inherited_nonblocking_socket_waits_for_permission_response() {
            let (input, mut output) = UnixStream::pair().unwrap();
            input.set_nonblocking(true).unwrap();
            let writer = thread::spawn(move || {
                thread::sleep(Duration::from_millis(20));
                output.write_all(b"PERMISSION authorized\n").unwrap();
            });
            let mut reader = BufReader::new(blocking_stream(input).unwrap());
            assert_eq!(line(&mut reader).unwrap(), "PERMISSION authorized");
            writer.join().unwrap();
        }
    }
}
#[cfg(target_os = "macos")]
pub use mac::{SocketDirectory, bridge, bundle_path};
