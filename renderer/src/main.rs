mod app;
mod controls;
mod diagnostics;
mod osc;
mod overlay;
mod platform;
mod protocol;
mod renderer;
mod scheduler;
mod timing;

use std::path::PathBuf;

use app::{AppConfig, WindowConfig};

const DEFAULT_OSC_PORT: u16 = 57_140;

fn main() {
    match parse_args().and_then(app::run) {
        Ok(code) => std::process::exit(code),
        Err(message) => {
            report_launch_error(&format!("SCShader renderer error: {message}"));
            std::process::exit(1);
        }
    }
}

pub(crate) fn report_launch_error(message: &str) {
    // Array.unixCmd captures stdout for the SC IDE; a GUI Windows parent has
    // no usable stderr console. Keep startup failures visible in the Post pane.
    #[cfg(target_os = "windows")]
    println!("{message}");
    #[cfg(not(target_os = "windows"))]
    eprintln!("{message}");
}

fn parse_args() -> Result<AppConfig, String> {
    let mut listen_port = DEFAULT_OSC_PORT;
    let mut shader_path = None;
    let mut window = WindowConfig::default();
    let mut backend = platform::GpuBackend::default();
    let mut gpu_selection = platform::GpuSelection::default();
    let mut window_system = platform::WindowSystem::default();
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--adapter" => {
                let name = args
                    .next()
                    .ok_or("--adapter requires a GPU name or substring")?;
                if name.trim().is_empty() {
                    return Err("--adapter requires a non-empty GPU name or substring".to_owned());
                }
                gpu_selection.adapter_name = Some(name);
            }
            "--power-preference" => {
                gpu_selection.power_preference = platform::GpuSelection::parse_power_preference(
                    &args.next().ok_or("--power-preference requires a value")?,
                )?;
            }
            "--backend" => {
                backend =
                    platform::GpuBackend::parse(&args.next().ok_or("--backend requires a value")?)?;
            }
            "--window-system" => {
                window_system = platform::WindowSystem::parse(
                    &args.next().ok_or("--window-system requires a value")?,
                )?;
            }
            "--version" => {
                println!("scshader-renderer {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--listen-port" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--listen-port requires a port number".to_owned())?;
                listen_port = value
                    .parse::<u16>()
                    .map_err(|_| format!("invalid OSC port: {value}"))?;
                if listen_port == 0 {
                    return Err("OSC port must be between 1 and 65535".to_owned());
                }
            }
            "--shader" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--shader requires a WGSL file path".to_owned())?;
                shader_path = Some(PathBuf::from(value));
            }
            "--width" => {
                window.logical_width = parse_positive_dimension(args.next(), "--width")?;
            }
            "--height" => {
                window.logical_height = parse_positive_dimension(args.next(), "--height")?;
            }
            "--title" => {
                let title = args
                    .next()
                    .ok_or_else(|| "--title requires a non-empty string".to_owned())?;
                if title.is_empty() {
                    return Err("--title requires a non-empty string".to_owned());
                }
                window.title = title;
            }
            "--position" => {
                let x = parse_i32(args.next(), "--position requires integer X and Y")?;
                let y = parse_i32(args.next(), "--position requires integer X and Y")?;
                window.position = Some((x, y));
            }
            "--fullscreen" => window.fullscreen = true,
            "--borderless" => window.borderless = true,
            "--monitor" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--monitor requires a zero-based monitor index".to_owned())?;
                window.monitor_index = Some(
                    value
                        .parse::<usize>()
                        .map_err(|_| format!("invalid monitor index: {value}"))?,
                );
            }
            "--no-vsync" => window.vsync = false,
            "--hide-cursor" => window.cursor_visible = false,
            "-h" | "--help" => {
                println!(
                    "SCShader desktop renderer\n\n\
                     Usage: scshader-renderer [--listen-port PORT] [--shader FILE] \\\n                     [--width PIXELS] [--height PIXELS] [--title TITLE] [--position X Y] \\\n                     [--fullscreen] [--borderless] [--monitor INDEX] [--no-vsync] [--hide-cursor]\n\n\
                     Backend: --backend auto|metal|dx12|vulkan|gl\n\
                     GPU: --adapter NAME (unique case-insensitive substring, no fallback)\n\
                     Preference: --power-preference high-performance|low-power|none\n\
                     Display: --window-system auto|x11|wayland (overrides are Linux-only)\n\
                     The OSC listener always binds to 127.0.0.1."
                );
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument: {arg}")),
        }
    }

    Ok(AppConfig {
        listen_port,
        shader_path,
        window,
        backend,
        gpu_selection,
        window_system,
    })
}

fn parse_positive_dimension(value: Option<String>, flag: &str) -> Result<u32, String> {
    let value = value.ok_or_else(|| format!("{flag} requires a positive integer"))?;
    value
        .parse::<u32>()
        .ok()
        .filter(|value| *value > 0)
        .ok_or_else(|| format!("{flag} requires a positive integer"))
}

fn parse_i32(value: Option<String>, message: &str) -> Result<i32, String> {
    let value = value.ok_or_else(|| message.to_owned())?;
    value.parse::<i32>().map_err(|_| message.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{parse_i32, parse_positive_dimension};

    #[test]
    fn parses_positive_window_dimension() {
        assert_eq!(
            parse_positive_dimension(Some("640".to_owned()), "--width"),
            Ok(640)
        );
        assert!(parse_positive_dimension(Some("0".to_owned()), "--width").is_err());
    }

    #[test]
    fn parses_signed_window_position() {
        assert_eq!(parse_i32(Some("-20".to_owned()), "position"), Ok(-20));
        assert!(parse_i32(Some("bad".to_owned()), "position").is_err());
    }
}
