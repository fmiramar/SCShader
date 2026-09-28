use std::net::SocketAddr;

use crate::controls::{ControlType, ControlValue, Interpolation};
use rosc::{OscMessage, OscType};

pub const PROTOCOL_MAJOR: i32 = 1;
pub const RENDERER_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const RESOURCE_ID: i32 = 1;
pub const MAX_BUFFER_VALUES: u32 = 16_384;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplyTarget {
    pub address: SocketAddr,
    pub versioned: bool,
}

impl ReplyTarget {
    pub fn from_sender(sender: SocketAddr, versioned: bool) -> Self {
        Self {
            address: sender,
            versioned,
        }
    }
}

#[derive(Debug)]
pub enum Command {
    Batch {
        commands: Vec<Command>,
        target_time: Option<f64>,
    },
    ScheduleClear {
        reply_to: ReplyTarget,
    },
    Hello {
        reply_to: ReplyTarget,
    },
    Ping {
        reply_to: ReplyTarget,
        sequence: i32,
        client_time: f64,
    },
    Status {
        reply_to: ReplyTarget,
    },
    DiagnosticsOverlay {
        reply_to: ReplyTarget,
        enabled: bool,
    },
    #[cfg(feature = "gpu-test-hooks")]
    TestDeviceLoss {
        reply_to: ReplyTarget,
    },
    #[cfg(feature = "gpu-test-hooks")]
    TestPixel {
        reply_to: ReplyTarget,
    },
    #[cfg(feature = "gpu-test-hooks")]
    TestSurfaceLoss {
        reply_to: ReplyTarget,
    },
    ShaderCreate {
        reply_to: ReplyTarget,
        resource_id: i32,
        path: String,
        source_type: String,
    },
    ShaderReload {
        reply_to: ReplyTarget,
        resource_id: i32,
    },
    ShaderFree {
        reply_to: ReplyTarget,
        resource_id: i32,
    },
    UniformFloat {
        reply_to: ReplyTarget,
        resource_id: i32,
        name: String,
        value: f32,
    },
    UniformSet {
        reply_to: ReplyTarget,
        resource_id: i32,
        name: String,
        value: ControlValue,
        duration: f64,
        mode: Interpolation,
    },
    UniformGet {
        reply_to: ReplyTarget,
        resource_id: i32,
        name: String,
        request_id: i32,
    },
    TextureCreate {
        reply_to: ReplyTarget,
        resource_id: i32,
        path: String,
    },
    TextureFree {
        reply_to: ReplyTarget,
        resource_id: i32,
    },
    ShaderTexture {
        reply_to: ReplyTarget,
        resource_id: i32,
        name: String,
        texture_id: i32,
    },
    ShaderFeedback {
        reply_to: ReplyTarget,
        resource_id: i32,
        source: String,
        amount: f32,
    },
    BufferCreate {
        reply_to: ReplyTarget,
        resource_id: i32,
        length: u32,
    },
    BufferWrite {
        reply_to: ReplyTarget,
        resource_id: i32,
        start: u32,
        values: Vec<f32>,
    },
    BufferFree {
        reply_to: ReplyTarget,
        resource_id: i32,
    },
    ShaderBuffer {
        reply_to: ReplyTarget,
        resource_id: i32,
        name: String,
        buffer_id: i32,
    },
    GraphSet {
        reply_to: ReplyTarget,
        resource_ids: Vec<i32>,
    },
    TimingMarker {
        reply_to: ReplyTarget,
        sequence: i32,
    },
    WindowResize {
        reply_to: ReplyTarget,
        width: u32,
        height: u32,
    },
    WindowTitle {
        reply_to: ReplyTarget,
        title: String,
    },
    WindowPosition {
        reply_to: ReplyTarget,
        x: i32,
        y: i32,
    },
    WindowFullscreen {
        reply_to: ReplyTarget,
        enabled: bool,
    },
    WindowBorderless {
        reply_to: ReplyTarget,
        enabled: bool,
    },
    WindowVsync {
        reply_to: ReplyTarget,
        enabled: bool,
    },
    WindowCursorVisible {
        reply_to: ReplyTarget,
        visible: bool,
    },
    WindowFront {
        reply_to: ReplyTarget,
    },
    WindowMetrics {
        reply_to: ReplyTarget,
    },
    WindowInputEnabled {
        reply_to: ReplyTarget,
        enabled: bool,
    },
    WindowClose {
        reply_to: ReplyTarget,
    },
    Quit,
}

impl Command {
    pub fn is_continuous(&self) -> bool {
        if let Self::Batch { commands, .. } = self {
            return commands.iter().all(Self::is_continuous);
        }
        matches!(
            self,
            Self::UniformFloat { .. } | Self::UniformSet { .. } | Self::BufferWrite { .. }
        )
    }

    pub fn reply_target(&self) -> Option<ReplyTarget> {
        match self {
            Self::Batch { commands, .. } => commands.iter().find_map(Self::reply_target),
            Self::ScheduleClear { reply_to } => Some(*reply_to),
            #[cfg(feature = "gpu-test-hooks")]
            Self::TestDeviceLoss { reply_to } => Some(*reply_to),
            #[cfg(feature = "gpu-test-hooks")]
            Self::TestPixel { reply_to } => Some(*reply_to),
            #[cfg(feature = "gpu-test-hooks")]
            Self::TestSurfaceLoss { reply_to } => Some(*reply_to),
            Self::Hello { reply_to }
            | Self::Ping { reply_to, .. }
            | Self::Status { reply_to }
            | Self::DiagnosticsOverlay { reply_to, .. }
            | Self::ShaderCreate { reply_to, .. }
            | Self::ShaderReload { reply_to, .. }
            | Self::ShaderFree { reply_to, .. }
            | Self::UniformFloat { reply_to, .. }
            | Self::UniformSet { reply_to, .. }
            | Self::UniformGet { reply_to, .. }
            | Self::TextureCreate { reply_to, .. }
            | Self::TextureFree { reply_to, .. }
            | Self::ShaderTexture { reply_to, .. }
            | Self::ShaderFeedback { reply_to, .. }
            | Self::BufferCreate { reply_to, .. }
            | Self::BufferWrite { reply_to, .. }
            | Self::BufferFree { reply_to, .. }
            | Self::ShaderBuffer { reply_to, .. }
            | Self::GraphSet { reply_to, .. }
            | Self::TimingMarker { reply_to, .. }
            | Self::WindowResize { reply_to, .. }
            | Self::WindowTitle { reply_to, .. }
            | Self::WindowPosition { reply_to, .. }
            | Self::WindowFullscreen { reply_to, .. }
            | Self::WindowBorderless { reply_to, .. }
            | Self::WindowVsync { reply_to, .. }
            | Self::WindowCursorVisible { reply_to, .. }
            | Self::WindowFront { reply_to }
            | Self::WindowMetrics { reply_to }
            | Self::WindowInputEnabled { reply_to, .. }
            | Self::WindowClose { reply_to } => Some(*reply_to),
            Self::Quit => None,
        }
    }

    /// Heap allocation charged to the bounded future scheduler (fixed fields are
    /// charged separately). Packet preflight bounds recursion before decoding.
    pub fn heap_bytes(&self) -> usize {
        match self {
            Self::Batch { commands, .. } => {
                commands.capacity() * std::mem::size_of::<Self>()
                    + commands.iter().map(Self::heap_bytes).sum::<usize>()
            }
            Self::ShaderCreate {
                path, source_type, ..
            } => path.capacity() + source_type.capacity(),
            Self::TextureCreate { path, .. } => path.capacity(),
            Self::UniformFloat { name, .. }
            | Self::UniformSet { name, .. }
            | Self::UniformGet { name, .. }
            | Self::ShaderTexture { name, .. }
            | Self::ShaderBuffer { name, .. } => name.capacity(),
            Self::ShaderFeedback { source, .. } => source.capacity(),
            Self::BufferWrite { values, .. } => values.capacity() * std::mem::size_of::<f32>(),
            Self::GraphSet { resource_ids, .. } => {
                resource_ids.capacity() * std::mem::size_of::<i32>()
            }
            Self::WindowTitle { title, .. } => title.capacity(),
            _ => 0,
        }
    }

    pub fn command_count(&self) -> usize {
        match self {
            Self::Batch { commands, .. } => {
                1 + commands.iter().map(Self::command_count).sum::<usize>()
            }
            _ => 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ErrorReply {
    pub reply_to: ReplyTarget,
    pub severity: &'static str,
    pub subsystem: &'static str,
    pub resource_id: i32,
    pub code: &'static str,
    pub message: String,
}

impl ErrorReply {
    pub fn protocol(reply_to: ReplyTarget, message: impl Into<String>) -> Self {
        Self {
            reply_to,
            severity: "error",
            subsystem: "protocol",
            resource_id: 0,
            code: "E_PROTOCOL",
            message: message.into(),
        }
    }

    pub fn bad_argument(
        reply_to: ReplyTarget,
        resource_id: i32,
        message: impl Into<String>,
    ) -> Self {
        Self {
            reply_to,
            severity: "error",
            subsystem: "uniforms",
            resource_id,
            code: "E_BAD_ARGUMENT",
            message: message.into(),
        }
    }

    pub fn queue_full(reply_to: ReplyTarget, dropped: u64) -> Self {
        Self {
            reply_to,
            severity: "warning",
            subsystem: "osc",
            resource_id: 0,
            code: "E_QUEUE_FULL",
            message: format!("bounded OSC queue full; dropped {dropped} continuous update(s)"),
        }
    }

    pub fn shader_validate(reply_to: ReplyTarget, message: impl Into<String>) -> Self {
        Self {
            reply_to,
            severity: "error",
            subsystem: "shader",
            resource_id: RESOURCE_ID,
            code: "E_SHADER_VALIDATE",
            message: message.into(),
        }
    }

    pub fn shader_compile(
        reply_to: ReplyTarget,
        resource_id: i32,
        message: impl Into<String>,
    ) -> Self {
        Self {
            reply_to,
            severity: "error",
            subsystem: "shader",
            resource_id,
            code: "E_SHADER_COMPILE",
            message: message.into(),
        }
    }

    pub fn resource_not_found(
        reply_to: ReplyTarget,
        resource_id: i32,
        message: impl Into<String>,
    ) -> Self {
        Self {
            reply_to,
            severity: "error",
            subsystem: "resources",
            resource_id,
            code: "E_RESOURCE_NOT_FOUND",
            message: message.into(),
        }
    }

    pub fn gpu(reply_to: ReplyTarget, message: impl Into<String>) -> Self {
        Self {
            reply_to,
            severity: "error",
            subsystem: "gpu",
            resource_id: 0,
            code: "E_GPU_DEVICE_LOST",
            message: message.into(),
        }
    }

    pub fn late(reply_to: ReplyTarget, message: impl Into<String>) -> Self {
        Self {
            reply_to,
            severity: "warning",
            subsystem: "timing",
            resource_id: 0,
            code: "E_LATE_EVENT",
            message: message.into(),
        }
    }

    pub fn window(reply_to: ReplyTarget, message: impl Into<String>) -> Self {
        Self {
            reply_to,
            severity: "error",
            subsystem: "window",
            resource_id: 0,
            code: "E_WINDOW",
            message: message.into(),
        }
    }
}

pub fn parse_message(message: OscMessage, sender: SocketAddr) -> Result<Command, ErrorReply> {
    let versioned = message.addr.starts_with("/scshader/v1/");
    let default_target = ReplyTarget::from_sender(sender, versioned);

    match message.addr.as_str() {
        "/scshader/hello" => parse_legacy_hello(&message.args, sender),
        "/scshader/v1/hello" => parse_v1_hello(&message.args, sender),
        "/scshader/ping" | "/scshader/v1/ping" => parse_ping(&message.args, default_target),
        "/scshader/v1/status" => parse_status(&message.args, default_target),
        "/scshader/v1/diagnostics/overlay" => parse_window_bool(
            &message.args,
            default_target,
            "diagnostics/overlay",
            |reply_to, enabled| Command::DiagnosticsOverlay { reply_to, enabled },
        ),
        #[cfg(feature = "gpu-test-hooks")]
        "/scshader/v1/test/device-loss" if message.args.is_empty() => Ok(Command::TestDeviceLoss {
            reply_to: default_target,
        }),
        #[cfg(feature = "gpu-test-hooks")]
        "/scshader/v1/test/pixel" if message.args.is_empty() => Ok(Command::TestPixel {
            reply_to: default_target,
        }),
        #[cfg(feature = "gpu-test-hooks")]
        "/scshader/v1/test/surface-loss" if message.args.is_empty() => {
            Ok(Command::TestSurfaceLoss {
                reply_to: default_target,
            })
        }
        "/scshader/v1/schedule/clear" => {
            if !message.args.is_empty() {
                return Err(ErrorReply::protocol(
                    default_target,
                    "schedule/clear expects no arguments",
                ));
            }
            Ok(Command::ScheduleClear {
                reply_to: default_target,
            })
        }
        "/scshader/v1/shader/create" => parse_shader_create(&message.args, default_target),
        "/scshader/v1/shader/reload" => parse_shader_resource(&message.args, default_target, true),
        "/scshader/v1/shader/free" => parse_shader_resource(&message.args, default_target, false),
        "/scshader/uniform/f" | "/scshader/v1/uniform/f" => {
            parse_uniform(&message.args, default_target)
        }
        "/scshader/v1/uniform/set" => parse_typed_uniform(&message.args, default_target, false),
        "/scshader/v1/uniform/glide" => parse_typed_uniform(&message.args, default_target, true),
        "/scshader/v1/uniform/get" => {
            let [id, OscType::String(name), request_id] = message.args.as_slice() else {
                return Err(ErrorReply::protocol(
                    default_target,
                    "uniform/get expects resource ID, name, and request ID",
                ));
            };
            Ok(Command::UniformGet {
                reply_to: default_target,
                resource_id: parse_resource_id(id, default_target)?,
                name: name.clone(),
                request_id: as_i32(request_id).ok_or_else(|| {
                    ErrorReply::protocol(
                        default_target,
                        "uniform/get request ID must be an integer",
                    )
                })?,
            })
        }
        "/scshader/v1/texture/create" => parse_texture_create(&message.args, default_target),
        "/scshader/v1/texture/free" => parse_texture_resource(&message.args, default_target),
        "/scshader/v1/shader/texture" => parse_shader_texture(&message.args, default_target),
        "/scshader/v1/shader/feedback" => parse_shader_feedback(&message.args, default_target),
        "/scshader/v1/buffer/create" => parse_buffer_create(&message.args, default_target),
        "/scshader/v1/buffer/write" => parse_buffer_write(&message.args, default_target),
        "/scshader/v1/buffer/free" => parse_buffer_free(&message.args, default_target),
        "/scshader/v1/shader/buffer" => parse_shader_buffer(&message.args, default_target),
        "/scshader/v1/graph/set" => parse_graph_set(&message.args, default_target),
        "/scshader/v1/timing/marker" => parse_timing_marker(&message.args, default_target),
        "/scshader/v1/window/resize" => parse_window_resize(&message.args, default_target),
        "/scshader/v1/window/title" => parse_window_title(&message.args, default_target),
        "/scshader/v1/window/position" => parse_window_position(&message.args, default_target),
        "/scshader/v1/window/fullscreen" => parse_window_bool(
            &message.args,
            default_target,
            "fullscreen",
            |reply_to, enabled| Command::WindowFullscreen { reply_to, enabled },
        ),
        "/scshader/v1/window/borderless" => parse_window_bool(
            &message.args,
            default_target,
            "borderless",
            |reply_to, enabled| Command::WindowBorderless { reply_to, enabled },
        ),
        "/scshader/v1/window/vsync" => parse_window_bool(
            &message.args,
            default_target,
            "vsync",
            |reply_to, enabled| Command::WindowVsync { reply_to, enabled },
        ),
        "/scshader/v1/window/cursor-visible" => parse_window_bool(
            &message.args,
            default_target,
            "cursor-visible",
            |reply_to, visible| Command::WindowCursorVisible { reply_to, visible },
        ),
        "/scshader/v1/window/front" => {
            parse_window_empty(&message.args, default_target, "front", |reply_to| {
                Command::WindowFront { reply_to }
            })
        }
        "/scshader/v1/window/metrics" => {
            parse_window_empty(&message.args, default_target, "metrics", |reply_to| {
                Command::WindowMetrics { reply_to }
            })
        }
        "/scshader/v1/window/input-enabled" => parse_window_bool(
            &message.args,
            default_target,
            "input-enabled",
            |reply_to, enabled| Command::WindowInputEnabled { reply_to, enabled },
        ),
        "/scshader/v1/window/close" => {
            parse_window_empty(&message.args, default_target, "close", |reply_to| {
                Command::WindowClose { reply_to }
            })
        }
        "/scshader/quit" | "/scshader/v1/quit" => {
            if message.args.is_empty() {
                Ok(Command::Quit)
            } else {
                Err(ErrorReply::protocol(
                    default_target,
                    "quit expects no arguments",
                ))
            }
        }
        _ => Err(ErrorReply::protocol(
            default_target,
            format!("unsupported OSC address: {}", message.addr),
        )),
    }
}

fn parse_window_resize(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [width, height] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "window/resize expects positive logical width and height",
        ));
    };
    let width = as_i32(width)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| ErrorReply::window(reply_to, "window width must be a positive integer"))?;
    let height = as_i32(height)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0)
        .ok_or_else(|| ErrorReply::window(reply_to, "window height must be a positive integer"))?;
    Ok(Command::WindowResize {
        reply_to,
        width,
        height,
    })
}

fn parse_window_title(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [OscType::String(title)] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "window/title expects one non-empty string",
        ));
    };
    if title.is_empty() {
        return Err(ErrorReply::window(
            reply_to,
            "window title must not be empty",
        ));
    }
    Ok(Command::WindowTitle {
        reply_to,
        title: title.clone(),
    })
}

fn parse_window_position(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [x, y] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "window/position expects integer x and y",
        ));
    };
    let x = as_i32(x).ok_or_else(|| ErrorReply::window(reply_to, "window x must be an integer"))?;
    let y = as_i32(y).ok_or_else(|| ErrorReply::window(reply_to, "window y must be an integer"))?;
    Ok(Command::WindowPosition { reply_to, x, y })
}

fn parse_window_bool(
    args: &[OscType],
    reply_to: ReplyTarget,
    name: &str,
    command: impl FnOnce(ReplyTarget, bool) -> Command,
) -> Result<Command, ErrorReply> {
    let [value] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            format!("window/{name} expects integer 0 or 1"),
        ));
    };
    let enabled = match as_i32(value) {
        Some(0) => false,
        Some(1) => true,
        _ => {
            return Err(ErrorReply::window(
                reply_to,
                format!("window/{name} must be integer 0 or 1"),
            ));
        }
    };
    Ok(command(reply_to, enabled))
}

fn parse_window_empty(
    args: &[OscType],
    reply_to: ReplyTarget,
    name: &str,
    command: impl FnOnce(ReplyTarget) -> Command,
) -> Result<Command, ErrorReply> {
    if args.is_empty() {
        Ok(command(reply_to))
    } else {
        Err(ErrorReply::protocol(
            reply_to,
            format!("window/{name} expects no arguments"),
        ))
    }
}

fn parse_timing_marker(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [sequence] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "timing/marker expects one integer sequence",
        ));
    };
    let sequence = as_i32(sequence).ok_or_else(|| {
        ErrorReply::protocol(reply_to, "timing marker sequence must be an integer")
    })?;
    Ok(Command::TimingMarker { reply_to, sequence })
}

fn parse_shader_create(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let ([resource_id, path] | [resource_id, path, _]) = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "shader/create expects resourceID, source path, and optional source type",
        ));
    };
    let resource_id = parse_resource_id(resource_id, reply_to)?;
    let OscType::String(path) = path else {
        return Err(ErrorReply::bad_argument(
            reply_to,
            resource_id,
            "shader source path must be a string",
        ));
    };
    if path.is_empty() {
        return Err(ErrorReply::bad_argument(
            reply_to,
            resource_id,
            "shader source path must not be empty",
        ));
    }
    let source_type = match args.get(2) {
        None => "wgsl".to_owned(),
        Some(OscType::String(source_type)) => source_type.clone(),
        Some(_) => {
            return Err(ErrorReply::protocol(
                reply_to,
                "shader source type must be a string",
            ));
        }
    };
    if !matches!(source_type.as_str(), "wgsl" | "glsl" | "shadertoy") {
        return Err(ErrorReply::protocol(
            reply_to,
            "shader source type must be wgsl, glsl, or shadertoy",
        ));
    }
    Ok(Command::ShaderCreate {
        reply_to,
        resource_id,
        path: path.clone(),
        source_type,
    })
}

fn parse_shader_resource(
    args: &[OscType],
    reply_to: ReplyTarget,
    reload: bool,
) -> Result<Command, ErrorReply> {
    let [resource_id] = args else {
        let command = if reload {
            "shader/reload"
        } else {
            "shader/free"
        };
        return Err(ErrorReply::protocol(
            reply_to,
            format!("{command} expects one resourceID"),
        ));
    };
    let resource_id = parse_resource_id(resource_id, reply_to)?;
    Ok(if reload {
        Command::ShaderReload {
            reply_to,
            resource_id,
        }
    } else {
        Command::ShaderFree {
            reply_to,
            resource_id,
        }
    })
}

fn parse_status(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    if args.is_empty() {
        Ok(Command::Status { reply_to })
    } else {
        Err(ErrorReply::protocol(
            reply_to,
            "status expects no arguments",
        ))
    }
}

fn parse_legacy_hello(args: &[OscType], sender: SocketAddr) -> Result<Command, ErrorReply> {
    let default_target = ReplyTarget::from_sender(sender, false);
    let [port] = args else {
        return Err(ErrorReply::protocol(
            default_target,
            "hello expects one integer reply port",
        ));
    };
    let port = as_port(port).ok_or_else(|| {
        ErrorReply::protocol(
            default_target,
            "hello reply port must be an integer from 1 to 65535",
        )
    })?;
    Ok(Command::Hello {
        reply_to: ReplyTarget::from_sender(with_port(sender, port), false),
    })
}

fn parse_v1_hello(args: &[OscType], sender: SocketAddr) -> Result<Command, ErrorReply> {
    let default_target = ReplyTarget::from_sender(sender, true);
    let [version, port, client_name] = args else {
        return Err(ErrorReply::protocol(
            default_target,
            "v1 hello expects protocolVersion, replyPort, clientName",
        ));
    };
    let Some(version) = as_i32(version) else {
        return Err(ErrorReply::protocol(
            default_target,
            "protocolVersion must be an integer",
        ));
    };
    if version != PROTOCOL_MAJOR {
        return Err(ErrorReply::protocol(
            default_target,
            format!("unsupported protocol major {version}; renderer supports {PROTOCOL_MAJOR}"),
        ));
    }
    let port = as_port(port).ok_or_else(|| {
        ErrorReply::protocol(
            default_target,
            "replyPort must be an integer from 1 to 65535",
        )
    })?;
    if !matches!(client_name, OscType::String(name) if !name.is_empty()) {
        return Err(ErrorReply::protocol(
            default_target,
            "clientName must be a non-empty string",
        ));
    }
    Ok(Command::Hello {
        reply_to: ReplyTarget::from_sender(with_port(sender, port), true),
    })
}

fn parse_ping(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [sequence, client_time] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "ping expects integer sequence and numeric clientTime",
        ));
    };
    let sequence = as_i32(sequence)
        .ok_or_else(|| ErrorReply::protocol(reply_to, "ping sequence must be an integer"))?;
    let client_time = as_f64(client_time)
        .filter(|value| value.is_finite())
        .ok_or_else(|| ErrorReply::protocol(reply_to, "ping clientTime must be finite numeric"))?;
    Ok(Command::Ping {
        reply_to,
        sequence,
        client_time,
    })
}

fn parse_texture_create(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [resource_id, OscType::String(path)] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "texture/create expects positive ID and image path",
        ));
    };
    let resource_id = as_i32(resource_id)
        .filter(|id| *id > 0)
        .ok_or_else(|| ErrorReply::protocol(reply_to, "texture ID must be a positive integer"))?;
    if path.is_empty() {
        return Err(ErrorReply::protocol(
            reply_to,
            "texture path must not be empty",
        ));
    }
    Ok(Command::TextureCreate {
        reply_to,
        resource_id,
        path: path.clone(),
    })
}

fn parse_texture_resource(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [resource_id] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "texture/free expects one positive ID",
        ));
    };
    let resource_id = as_i32(resource_id)
        .filter(|id| *id > 0)
        .ok_or_else(|| ErrorReply::protocol(reply_to, "texture ID must be a positive integer"))?;
    Ok(Command::TextureFree {
        reply_to,
        resource_id,
    })
}

fn parse_shader_texture(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [resource_id, OscType::String(name), texture_id] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "shader/texture expects shader ID, name, and texture ID",
        ));
    };
    let resource_id = as_i32(resource_id)
        .filter(|id| *id > 0)
        .ok_or_else(|| ErrorReply::protocol(reply_to, "shader ID must be positive"))?;
    let texture_id = as_i32(texture_id)
        .filter(|id| *id > 0)
        .ok_or_else(|| ErrorReply::protocol(reply_to, "texture ID must be positive"))?;
    Ok(Command::ShaderTexture {
        reply_to,
        resource_id,
        name: name.clone(),
        texture_id,
    })
}

fn parse_shader_feedback(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [resource_id, OscType::String(source), amount] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "shader/feedback expects shader ID, source, and amount",
        ));
    };
    let resource_id = as_i32(resource_id)
        .filter(|id| *id > 0)
        .ok_or_else(|| ErrorReply::protocol(reply_to, "shader ID must be positive"))?;
    let amount = as_f64(amount)
        .filter(|value| {
            value.is_finite() && *value >= f64::from(f32::MIN) && *value <= f64::from(f32::MAX)
        })
        .map(|value| value as f32)
        .ok_or_else(|| ErrorReply::protocol(reply_to, "feedback amount must be finite"))?;
    Ok(Command::ShaderFeedback {
        reply_to,
        resource_id,
        source: source.clone(),
        amount,
    })
}

fn parse_typed_uniform(
    args: &[OscType],
    reply_to: ReplyTarget,
    glide: bool,
) -> Result<Command, ErrorReply> {
    if args.len() != if glide { 6 } else { 4 } {
        return Err(ErrorReply::protocol(
            reply_to,
            "uniform/set expects ID, name, type, blob; uniform/glide additionally expects duration and mode",
        ));
    }
    let resource_id = parse_resource_id(&args[0], reply_to)?;
    let bad = |message: String| ErrorReply::bad_argument(reply_to, resource_id, message);
    let (OscType::String(name), OscType::String(kind), OscType::Blob(bytes)) =
        (&args[1], &args[2], &args[3])
    else {
        return Err(bad(
            "uniform expects name/type strings and a big-endian 32-bit blob".to_owned(),
        ));
    };
    let value =
        ControlValue::from_blob(ControlType::parse(kind).map_err(bad)?, bytes).map_err(bad)?;
    let (duration, mode) = if glide {
        let duration = as_f64(&args[4])
            .filter(|v| v.is_finite() && *v >= 0.0)
            .ok_or_else(|| bad("glide duration must be finite and non-negative".to_owned()))?;
        let OscType::String(mode) = &args[5] else {
            return Err(bad("interpolation mode must be a string".to_owned()));
        };
        (duration, Interpolation::parse(mode).map_err(bad)?)
    } else {
        (0.0, Interpolation::Step)
    };
    Ok(Command::UniformSet {
        reply_to,
        resource_id,
        name: name.clone(),
        value,
        duration,
        mode,
    })
}

fn parse_uniform(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [resource_id, name, value] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "uniform/f expects resourceID, name, value",
        ));
    };
    let resource_id = parse_resource_id(resource_id, reply_to)?;
    let OscType::String(name) = name else {
        return Err(ErrorReply::bad_argument(
            reply_to,
            resource_id,
            "uniform name must be a string",
        ));
    };
    let value = as_f64(value)
        .filter(|value| value.is_finite())
        .ok_or_else(|| {
            ErrorReply::bad_argument(
                reply_to,
                resource_id,
                "uniform value must be finite numeric",
            )
        })?;
    if value < f32::MIN as f64 || value > f32::MAX as f64 {
        return Err(ErrorReply::bad_argument(
            reply_to,
            resource_id,
            "uniform value is outside the float32 range",
        ));
    }
    Ok(Command::UniformFloat {
        reply_to,
        resource_id,
        name: name.clone(),
        value: value as f32,
    })
}

fn parse_buffer_create(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [resource_id, length] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "buffer/create expects positive buffer ID and length",
        ));
    };
    let resource_id = parse_resource_id(resource_id, reply_to)?;
    let length = as_i32(length)
        .and_then(|value| u32::try_from(value).ok())
        .filter(|value| *value > 0 && *value <= MAX_BUFFER_VALUES)
        .ok_or_else(|| {
            ErrorReply::bad_argument(
                reply_to,
                resource_id,
                format!("buffer length must be between 1 and {MAX_BUFFER_VALUES}"),
            )
        })?;
    Ok(Command::BufferCreate {
        reply_to,
        resource_id,
        length,
    })
}

fn parse_buffer_write(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [resource_id, start, OscType::Blob(bytes)] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "buffer/write expects buffer ID, non-negative start, and a float32 OSC blob",
        ));
    };
    let resource_id = parse_resource_id(resource_id, reply_to)?;
    let start = as_i32(start)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| {
            ErrorReply::bad_argument(reply_to, resource_id, "buffer start must be non-negative")
        })?;
    if bytes.is_empty() || bytes.len() % 4 != 0 {
        return Err(ErrorReply::bad_argument(
            reply_to,
            resource_id,
            "buffer blob must contain one or more big-endian float32 values",
        ));
    }
    let values: Vec<f32> = bytes
        .chunks_exact(4)
        .map(|bytes| f32::from_bits(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])))
        .collect();
    if values.len() > usize::try_from(MAX_BUFFER_VALUES).unwrap_or(0)
        || values.iter().any(|value| !value.is_finite())
    {
        return Err(ErrorReply::bad_argument(
            reply_to,
            resource_id,
            "buffer blob has too many values or contains a non-finite float32",
        ));
    }
    Ok(Command::BufferWrite {
        reply_to,
        resource_id,
        start,
        values,
    })
}

fn parse_buffer_free(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [resource_id] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "buffer/free expects one positive buffer ID",
        ));
    };
    Ok(Command::BufferFree {
        reply_to,
        resource_id: parse_resource_id(resource_id, reply_to)?,
    })
}

fn parse_shader_buffer(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let [resource_id, OscType::String(name), buffer_id] = args else {
        return Err(ErrorReply::protocol(
            reply_to,
            "shader/buffer expects shader ID, name, and buffer ID",
        ));
    };
    let resource_id = parse_resource_id(resource_id, reply_to)?;
    let buffer_id = parse_resource_id(buffer_id, reply_to)?;
    if name != "spectrum" {
        return Err(ErrorReply::bad_argument(
            reply_to,
            resource_id,
            "the current buffer ABI exposes only the spectrum binding",
        ));
    }
    Ok(Command::ShaderBuffer {
        reply_to,
        resource_id,
        name: name.clone(),
        buffer_id,
    })
}

fn parse_graph_set(args: &[OscType], reply_to: ReplyTarget) -> Result<Command, ErrorReply> {
    let mut resource_ids = Vec::with_capacity(args.len());
    for value in args {
        let resource_id = parse_resource_id(value, reply_to)?;
        if resource_ids.contains(&resource_id) {
            return Err(ErrorReply::bad_argument(
                reply_to,
                resource_id,
                "graph node IDs must be unique",
            ));
        }
        resource_ids.push(resource_id);
    }
    Ok(Command::GraphSet {
        reply_to,
        resource_ids,
    })
}

fn parse_resource_id(value: &OscType, reply_to: ReplyTarget) -> Result<i32, ErrorReply> {
    as_i32(value)
        .filter(|resource_id| *resource_id > 0)
        .ok_or_else(|| {
            ErrorReply::bad_argument(reply_to, 0, "resourceID must be a positive integer")
        })
}

fn as_i32(value: &OscType) -> Option<i32> {
    match value {
        OscType::Int(value) => Some(*value),
        OscType::Long(value) => i32::try_from(*value).ok(),
        _ => None,
    }
}

fn as_port(value: &OscType) -> Option<u16> {
    as_i32(value)
        .and_then(|value| u16::try_from(value).ok())
        .filter(|port| *port != 0)
}

fn as_f64(value: &OscType) -> Option<f64> {
    match value {
        OscType::Float(value) => Some(f64::from(*value)),
        OscType::Double(value) => Some(*value),
        OscType::Int(value) => Some(f64::from(*value)),
        OscType::Long(value) => Some(*value as f64),
        _ => None,
    }
}

fn with_port(sender: SocketAddr, port: u16) -> SocketAddr {
    SocketAddr::new(sender.ip(), port)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sender() -> SocketAddr {
        "127.0.0.1:50000".parse().expect("valid test address")
    }

    fn message(addr: &str, args: Vec<OscType>) -> OscMessage {
        OscMessage {
            addr: addr.to_owned(),
            args,
        }
    }

    #[test]
    fn parses_versioned_hello_and_reply_port() {
        let command = parse_message(
            message(
                "/scshader/v1/hello",
                vec![1.into(), 57120.into(), "sclang-test".into()],
            ),
            sender(),
        )
        .expect("valid hello");

        let Command::Hello { reply_to } = command else {
            panic!("expected hello")
        };
        assert_eq!(reply_to.address.port(), 57120);
        assert!(reply_to.versioned);
    }

    #[test]
    fn parses_phase_zero_hello_alias() {
        let command = parse_message(message("/scshader/hello", vec![57121.into()]), sender())
            .expect("valid legacy hello");

        let Command::Hello { reply_to } = command else {
            panic!("expected hello")
        };
        assert_eq!(reply_to.address.port(), 57121);
        assert!(!reply_to.versioned);
    }

    #[test]
    fn rejects_protocol_major_mismatch() {
        let error = parse_message(
            message(
                "/scshader/v1/hello",
                vec![2.into(), 57120.into(), "sclang-test".into()],
            ),
            sender(),
        )
        .expect_err("major mismatch must fail");
        assert_eq!(error.code, "E_PROTOCOL");
    }

    #[test]
    fn accepts_supercollider_float_uniform() {
        let command = parse_message(
            message(
                "/scshader/v1/uniform/f",
                vec![1.into(), "amount".into(), OscType::Float(0.75)],
            ),
            sender(),
        )
        .expect("valid uniform");

        let Command::UniformFloat { value, .. } = command else {
            panic!("expected uniform")
        };
        assert_eq!(value, 0.75);
    }

    #[test]
    fn typed_uniform_protocol_is_exact_and_continuous() {
        let command = parse_message(
            message(
                "/scshader/v1/uniform/set",
                vec![
                    7.into(),
                    "seed".into(),
                    "uint".into(),
                    OscType::Blob(u32::MAX.to_be_bytes().to_vec()),
                ],
            ),
            sender(),
        )
        .unwrap();
        assert!(command.is_continuous());
        let Command::UniformSet {
            value,
            mode,
            duration,
            ..
        } = command
        else {
            panic!("expected typed set");
        };
        assert_eq!(value.blob(), u32::MAX.to_be_bytes());
        assert_eq!(mode, Interpolation::Step);
        assert_eq!(duration, 0.0);
        let command = parse_message(
            message(
                "/scshader/v1/uniform/glide",
                vec![
                    7.into(),
                    "gain".into(),
                    "float".into(),
                    OscType::Blob(1_f32.to_be_bytes().to_vec()),
                    0.25_f32.into(),
                    "smooth".into(),
                ],
            ),
            sender(),
        )
        .unwrap();
        assert!(matches!(
            command,
            Command::UniformSet {
                mode: Interpolation::Smooth,
                duration: 0.25,
                ..
            }
        ));
    }

    #[test]
    fn rejects_malformed_typed_controls() {
        for args in [
            vec![
                7.into(),
                "gain".into(),
                "float".into(),
                OscType::Blob(vec![0; 3]),
            ],
            vec![
                7.into(),
                "gain".into(),
                "vec3".into(),
                OscType::Blob(vec![0; 16]),
            ],
            vec![
                7.into(),
                "gain".into(),
                "float".into(),
                OscType::Blob(f32::INFINITY.to_be_bytes().to_vec()),
            ],
            vec![
                7.into(),
                "gain".into(),
                "unknown".into(),
                OscType::Blob(vec![0; 4]),
            ],
        ] {
            assert!(parse_message(message("/scshader/v1/uniform/set", args), sender()).is_err());
        }
        for duration in [-1.0_f32, f32::INFINITY, f32::NAN] {
            assert!(
                parse_message(
                    message(
                        "/scshader/v1/uniform/glide",
                        vec![
                            7.into(),
                            "gain".into(),
                            "float".into(),
                            OscType::Blob(vec![0; 4]),
                            duration.into(),
                            "linear".into()
                        ]
                    ),
                    sender()
                )
                .is_err()
            );
        }
    }

    #[test]
    fn uniform_get_keeps_request_identity() {
        let command = parse_message(
            message(
                "/scshader/v1/uniform/get",
                vec![7.into(), "gain".into(), 91.into()],
            ),
            sender(),
        )
        .unwrap();
        assert!(matches!(
            command,
            Command::UniformGet {
                resource_id: 7,
                request_id: 91,
                ..
            }
        ));
        assert!(
            parse_message(
                message("/scshader/v1/uniform/get", vec![7.into(), "gain".into()]),
                sender()
            )
            .is_err()
        );
    }

    #[test]
    fn rejects_non_numeric_uniform() {
        let error = parse_message(
            message(
                "/scshader/v1/uniform/f",
                vec![1.into(), "amount".into(), "loud".into()],
            ),
            sender(),
        )
        .expect_err("string value must fail");
        assert_eq!(error.code, "E_BAD_ARGUMENT");
    }

    #[test]
    fn parses_versioned_status() {
        let command = parse_message(message("/scshader/v1/status", vec![]), sender())
            .expect("valid status request");
        assert!(matches!(command, Command::Status { .. }));
    }

    #[test]
    fn parses_big_endian_float_buffer_blob() {
        let command = parse_message(
            message(
                "/scshader/v1/buffer/write",
                vec![
                    9.into(),
                    2.into(),
                    OscType::Blob(vec![0x3f, 0x80, 0x00, 0x00, 0xc0, 0x20, 0x00, 0x00]),
                ],
            ),
            sender(),
        )
        .expect("valid big-endian float blob");
        let Command::BufferWrite {
            resource_id,
            start,
            values,
            ..
        } = command
        else {
            panic!("expected buffer write")
        };
        assert_eq!(resource_id, 9);
        assert_eq!(start, 2);
        assert_eq!(values, vec![1.0, -2.5]);
    }

    #[test]
    fn rejects_misaligned_buffer_blob() {
        let error = parse_message(
            message(
                "/scshader/v1/buffer/write",
                vec![9.into(), 0.into(), OscType::Blob(vec![0, 1, 2])],
            ),
            sender(),
        )
        .expect_err("misaligned buffer blob must fail");
        assert_eq!(error.code, "E_BAD_ARGUMENT");
    }

    #[test]
    fn rejects_unsupported_shader_buffer_binding() {
        let error = parse_message(
            message(
                "/scshader/v1/shader/buffer",
                vec![1.into(), "other".into(), 2.into()],
            ),
            sender(),
        )
        .expect_err("unsupported buffer binding must fail");
        assert_eq!(error.code, "E_BAD_ARGUMENT");
    }

    #[test]
    fn parses_linear_graph_set() {
        let command = parse_message(
            message(
                "/scshader/v1/graph/set",
                vec![10.into(), 11.into(), 12.into()],
            ),
            sender(),
        )
        .expect("valid graph node sequence");
        let Command::GraphSet { resource_ids, .. } = command else {
            panic!("expected graph set")
        };
        assert_eq!(resource_ids, vec![10, 11, 12]);
    }

    #[test]
    fn rejects_duplicate_graph_nodes() {
        let error = parse_message(
            message("/scshader/v1/graph/set", vec![10.into(), 10.into()]),
            sender(),
        )
        .expect_err("duplicate graph nodes must fail");
        assert_eq!(error.code, "E_BAD_ARGUMENT");
    }

    #[test]
    fn parses_shader_create() {
        let command = parse_message(
            message(
                "/scshader/v1/shader/create",
                vec![7.into(), "example.wgsl".into()],
            ),
            sender(),
        )
        .expect("valid shader creation");
        let Command::ShaderCreate {
            resource_id, path, ..
        } = command
        else {
            panic!("expected shader creation")
        };
        assert_eq!(resource_id, 7);
        assert_eq!(path, "example.wgsl");
    }

    #[test]
    fn parses_shader_create_source_type() {
        let command = parse_message(
            message(
                "/scshader/v1/shader/create",
                vec![7.into(), "example.frag".into(), "shadertoy".into()],
            ),
            sender(),
        )
        .expect("valid Shadertoy shader creation");
        let Command::ShaderCreate { source_type, .. } = command else {
            panic!("expected shader creation")
        };
        assert_eq!(source_type, "shadertoy");
    }

    #[test]
    fn rejects_unknown_shader_source_type() {
        let error = parse_message(
            message(
                "/scshader/v1/shader/create",
                vec![7.into(), "example.frag".into(), "hlsl".into()],
            ),
            sender(),
        )
        .expect_err("unknown source type must fail");
        assert_eq!(error.code, "E_PROTOCOL");
    }

    #[test]
    fn rejects_zero_shader_resource_id() {
        let error = parse_message(
            message("/scshader/v1/shader/free", vec![0.into()]),
            sender(),
        )
        .expect_err("resource ID zero must fail");
        assert_eq!(error.code, "E_BAD_ARGUMENT");
    }

    #[test]
    fn parses_timing_marker() {
        let command = parse_message(
            message("/scshader/v1/timing/marker", vec![42.into()]),
            sender(),
        )
        .expect("valid timing marker");
        assert!(matches!(
            command,
            Command::TimingMarker { sequence: 42, .. }
        ));
    }

    #[test]
    fn parses_window_resize() {
        let command = parse_message(
            message("/scshader/v1/window/resize", vec![1920.into(), 1080.into()]),
            sender(),
        )
        .expect("valid window resize");
        assert!(matches!(
            command,
            Command::WindowResize {
                width: 1920,
                height: 1080,
                ..
            }
        ));
    }

    #[test]
    fn rejects_invalid_window_boolean() {
        let error = parse_message(
            message("/scshader/v1/window/input-enabled", vec![2.into()]),
            sender(),
        )
        .expect_err("invalid boolean must fail");
        assert_eq!(error.code, "E_WINDOW");
    }
    #[test]
    fn diagnostics_requires_a_boolean_and_test_hooks_are_feature_gated() {
        assert!(matches!(
            parse_message(
                message("/scshader/v1/diagnostics/overlay", vec![1.into()]),
                sender()
            ),
            Ok(Command::DiagnosticsOverlay { enabled: true, .. })
        ));
        for args in [vec![], vec![2.into()], vec![0.into(), 1.into()]] {
            assert!(
                parse_message(message("/scshader/v1/diagnostics/overlay", args), sender()).is_err()
            );
        }
        let result = parse_message(message("/scshader/v1/test/device-loss", vec![]), sender());
        assert_eq!(result.is_ok(), cfg!(feature = "gpu-test-hooks"));
    }
}
