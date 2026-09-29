use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize},
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::Key,
    window::{Fullscreen, Window, WindowId},
};

use crate::{
    osc::{IncomingCommand, OscServer, WindowMetrics},
    platform::{GpuBackend, GpuSelection, WindowSystem},
    protocol::{Command, ErrorReply, ReplyTarget},
    renderer::{HotReload, Renderer},
    scheduler::{ScheduledCommand, Scheduler},
};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const LATE_EVENT_TOLERANCE_SECONDS: f64 = 0.050;

#[derive(Clone, Debug)]
pub struct WindowConfig {
    pub logical_width: u32,
    pub logical_height: u32,
    pub title: String,
    pub position: Option<(i32, i32)>,
    pub fullscreen: bool,
    pub borderless: bool,
    pub monitor_index: Option<usize>,
    pub vsync: bool,
    pub cursor_visible: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            logical_width: WINDOW_WIDTH,
            logical_height: WINDOW_HEIGHT,
            title: "SCShader".to_owned(),
            position: None,
            fullscreen: false,
            borderless: false,
            monitor_index: None,
            vsync: true,
            cursor_visible: true,
        }
    }
}

#[derive(Debug)]
pub struct AppConfig {
    pub listen_port: u16,
    pub shader_path: Option<PathBuf>,
    pub window: WindowConfig,
    pub backend: GpuBackend,
    pub gpu_selection: GpuSelection,
    pub window_system: WindowSystem,
}

pub fn run(config: AppConfig) -> Result<i32, String> {
    config.backend.mask(std::env::consts::OS)?;
    config.window_system.validate(std::env::consts::OS)?;
    let started_at = Instant::now();
    let osc = OscServer::bind(config.listen_port, started_at)?;
    let event_loop = config.window_system.event_loop()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(config, osc, started_at);
    event_loop
        .run_app(&mut app)
        .map_err(|error| error.to_string())?;
    Ok(app.exit_code)
}

struct App {
    config: AppConfig,
    osc: OscServer,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    started_at: Instant,
    last_reply_target: Option<ReplyTarget>,
    scheduler: Scheduler,
    scheduled_sequence: u64,
    input_enabled: bool,
    pending_mouse_position: Option<PhysicalPosition<f64>>,
    last_mouse_position: PhysicalPosition<f64>,
    fullscreen: bool,
    borderless: bool,
    vsync: bool,
    cursor_visible: bool,
    title: String,
    recovery_count: u32,
    exit_code: i32,
    late_events: u64,
}

impl App {
    fn new(config: AppConfig, osc: OscServer, started_at: Instant) -> Self {
        let window_config = config.window.clone();
        Self {
            config,
            osc,
            window: None,
            renderer: None,
            started_at,
            last_reply_target: None,
            scheduler: Scheduler::default(),
            scheduled_sequence: 0,
            input_enabled: false,
            pending_mouse_position: None,
            last_mouse_position: PhysicalPosition::new(0.0, 0.0),
            fullscreen: window_config.fullscreen,
            borderless: window_config.borderless,
            vsync: window_config.vsync,
            cursor_visible: window_config.cursor_visible,
            title: window_config.title,
            recovery_count: 0,
            exit_code: 0,
            late_events: 0,
        }
    }

    fn initialize(&mut self, event_loop: &ActiveEventLoop) -> Result<(), String> {
        let selected_monitor = if self.config.window.fullscreen {
            match self.config.window.monitor_index {
                Some(index) => Some(event_loop.available_monitors().nth(index).ok_or_else(|| {
                    format!(
                        "requested monitor index {index} is not available for the SCShader window"
                    )
                })?),
                // Wayland has no primary monitor. None lets the compositor choose.
                None => event_loop.primary_monitor(),
            }
        } else {
            None
        };
        let mut attributes = Window::default_attributes()
            .with_title(&self.config.window.title)
            .with_inner_size(LogicalSize::new(
                self.config.window.logical_width,
                self.config.window.logical_height,
            ))
            .with_decorations(!self.config.window.borderless);
        if let Some((x, y)) = self.config.window.position {
            attributes = attributes.with_position(LogicalPosition::new(x, y));
        }
        if self.config.window.fullscreen {
            attributes = attributes.with_fullscreen(Some(Fullscreen::Borderless(selected_monitor)));
        }
        let window = Arc::new(
            event_loop
                .create_window(attributes)
                .map_err(|error| format!("could not create window: {error}"))?,
        );
        window.set_cursor_visible(self.config.window.cursor_visible);

        let custom_shader = self
            .config
            .shader_path
            .as_ref()
            .map(|path| {
                std::fs::read_to_string(path)
                    .map_err(|error| format!("could not read shader {}: {error}", path.display()))
            })
            .transpose()?;

        let renderer = Renderer::new(
            Arc::clone(&window),
            custom_shader.as_deref(),
            self.config.window.vsync,
            self.config.backend,
            self.config.gpu_selection.clone(),
            event_loop.owned_display_handle(),
        )?;
        println!(
            "SCShader renderer ready: OSC 127.0.0.1:{}, backend {}, device {}",
            self.config.listen_port,
            renderer.backend_name(),
            renderer.device_name()
        );

        if let Some(diagnostic) = renderer.startup_diagnostic() {
            eprintln!("SCShader shader diagnostic: {diagnostic}");
        }

        self.window = Some(window);
        self.renderer = Some(renderer);
        Ok(())
    }

    fn process_commands(&mut self, event_loop: &ActiveEventLoop) {
        let started = Instant::now();
        // Fair service for due and incoming work, yielding between atomic OSC
        // packets. A single compile/atomic bundle can still exceed this budget.
        for _ in 0..128 {
            if event_loop.exiting() || !self.check_gpu(event_loop) {
                break;
            }
            let mut progressed = false;
            if let Some(scheduled) = self.scheduler.pop_due(self.renderer_time()) {
                progressed = true;
                self.execute_command(scheduled.command, Some(scheduled.target_time), event_loop);
            }
            if let Ok(incoming) = self.osc.try_recv() {
                progressed = true;
                self.accept_incoming(incoming, event_loop);
            }
            if !progressed || started.elapsed() >= Duration::from_millis(2) {
                break;
            }
        }
    }

    fn accept_incoming(&mut self, incoming: IncomingCommand, event_loop: &ActiveEventLoop) {
        match incoming.target_time {
            Some(target_time) if target_time > self.renderer_time() => {
                self.scheduled_sequence = self.scheduled_sequence.saturating_add(1);
                let reply_to = incoming.command.reply_target().or(self.last_reply_target);
                let continuous = incoming.command.is_continuous();
                let result = self.scheduler.push(
                    ScheduledCommand {
                        command: incoming.command,
                        target_time,
                        sequence: self.scheduled_sequence,
                    },
                    self.renderer_time(),
                );
                if let Err(error) = result
                    && (!continuous || self.scheduler.rejected().is_power_of_two())
                    && let Some(reply_to) = reply_to
                {
                    self.send_error(ErrorReply {
                        reply_to,
                        severity: "warning",
                        subsystem: "timing",
                        resource_id: 0,
                        code: "E_SCHEDULE_LIMIT",
                        message: format!(
                            "{}; rejected {} packet(s)",
                            error.message(),
                            self.scheduler.rejected()
                        ),
                    });
                }
            }
            target_time => self.execute_command(incoming.command, target_time, event_loop),
        }
    }

    fn execute_command(
        &mut self,
        command: Command,
        scheduled_time: Option<f64>,
        event_loop: &ActiveEventLoop,
    ) {
        if event_loop.exiting() || !self.check_gpu(event_loop) {
            return;
        }
        let applied_renderer_time = self.renderer_time();
        if let Some(target_time) = scheduled_time
            && !matches!(&command, Command::Batch { .. })
            && applied_renderer_time - target_time > LATE_EVENT_TOLERANCE_SECONDS
        {
            self.late_events = self.late_events.saturating_add(1);
            if let Some(reply_to) = command.reply_target() {
                self.send_error(ErrorReply::late(
                    reply_to,
                    format!(
                        "scheduled event was {:.3} ms late and was applied immediately",
                        (applied_renderer_time - target_time) * 1_000.0
                    ),
                ));
            }
        }

        match command {
            Command::DiagnosticsOverlay { reply_to, enabled } => {
                self.last_reply_target = Some(reply_to);
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.set_show_stats(enabled);
                }
            }
            #[cfg(feature = "gpu-test-hooks")]
            Command::TestDeviceLoss { reply_to } => {
                self.last_reply_target = Some(reply_to);
                if let Some(renderer) = &self.renderer {
                    renderer.simulate_device_loss();
                }
            }
            #[cfg(feature = "gpu-test-hooks")]
            Command::TestPixel { reply_to } => {
                if let Some(renderer) = &self.renderer {
                    match renderer.feedback_pixel() {
                        Ok(bytes) => {
                            if let Err(error) = self.osc.replies().send_test_pixel(reply_to, bytes)
                            {
                                eprintln!("SCShader OSC reply error: {error}");
                            }
                        }
                        Err(message) => self.send_error(ErrorReply::gpu(reply_to, message)),
                    }
                }
            }
            #[cfg(feature = "gpu-test-hooks")]
            Command::TestSurfaceLoss { reply_to } => {
                self.last_reply_target = Some(reply_to);
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.simulate_surface_loss();
                }
            }
            Command::Batch { commands, .. } => {
                // Direct members stay consecutive. Nested future bundles are
                // separately scheduled, as allowed by OSC's atomicity exception.
                for command in commands {
                    if let Command::Batch { target_time, .. } = &command {
                        let target_time = *target_time;
                        self.accept_incoming(
                            IncomingCommand {
                                command,
                                target_time,
                            },
                            event_loop,
                        );
                    } else {
                        self.execute_command(command, scheduled_time, event_loop);
                    }
                }
            }
            Command::ScheduleClear { reply_to } => {
                let count = self.scheduler.clear();
                if let Err(error) = self.osc.replies().send_schedule_cleared(reply_to, count) {
                    eprintln!("SCShader OSC reply error: {error}");
                }
            }
            Command::Hello { reply_to } => {
                self.last_reply_target = Some(reply_to);
                self.send_ready(reply_to);
                self.send_window_metrics(reply_to);
            }
            Command::Ping {
                reply_to,
                sequence,
                client_time,
            } => {
                let renderer_time = self.started_at.elapsed().as_secs_f64();
                if let Err(error) =
                    self.osc
                        .replies()
                        .send_pong(reply_to, sequence, client_time, renderer_time)
                {
                    eprintln!("SCShader OSC reply error: {error}");
                }
            }
            Command::Status { reply_to } => {
                self.last_reply_target = Some(reply_to);
                match self.renderer.as_ref() {
                    Some(renderer) => {
                        let stats = renderer.status();
                        if let Err(error) = self.osc.replies().send_status(
                            reply_to,
                            stats.fps,
                            stats.frame_index,
                            stats.gpu_frame_ms,
                            stats.cpu_frame_ms,
                            self.scheduler.len() as u64,
                            renderer.shader_count(),
                            renderer.texture_count(),
                            renderer.buffer_count(),
                            1,
                            self.scheduler.bytes(),
                            self.scheduler.rejected(),
                            self.osc.dropped_updates(),
                            self.late_events,
                            renderer.texture_bytes(),
                            renderer.show_stats(),
                            self.recovery_count,
                            renderer.last_compile_ok(),
                        ) {
                            eprintln!("SCShader OSC reply error: {error}");
                        }
                    }
                    None => self.send_error(ErrorReply::protocol(
                        reply_to,
                        "renderer is not initialized",
                    )),
                }
            }
            Command::ShaderCreate {
                reply_to,
                resource_id,
                path,
                source_type,
            } => {
                self.last_reply_target = Some(reply_to);
                let result = match self.renderer.as_mut() {
                    Some(renderer) => {
                        let result = renderer.create_shader(resource_id, path.into(), &source_type);
                        renderer.set_compile_result(result.is_ok());
                        result
                    }
                    None => Err("renderer is not initialized".to_owned()),
                };
                match result {
                    Ok(()) => {
                        self.send_shader_reflection(reply_to, resource_id);
                        self.send_shader_created(reply_to, resource_id);
                    }
                    Err(message) => {
                        self.send_error(ErrorReply::shader_compile(reply_to, resource_id, message))
                    }
                }
            }
            Command::ShaderReload {
                reply_to,
                resource_id,
            } => {
                self.last_reply_target = Some(reply_to);
                let result = match self.renderer.as_mut() {
                    Some(renderer) => {
                        let result = renderer.reload_shader(resource_id);
                        renderer.set_compile_result(result.is_ok());
                        result
                    }
                    None => Err("renderer is not initialized".to_owned()),
                };
                match result {
                    Ok(()) => {
                        self.send_shader_reflection(reply_to, resource_id);
                        self.send_shader_reloaded(reply_to, resource_id);
                    }
                    Err(message) if message.contains("does not exist") => self.send_error(
                        ErrorReply::resource_not_found(reply_to, resource_id, message),
                    ),
                    Err(message) => {
                        self.send_error(ErrorReply::shader_compile(reply_to, resource_id, message))
                    }
                }
            }
            Command::ShaderFree {
                reply_to,
                resource_id,
            } => {
                self.last_reply_target = Some(reply_to);
                let result = match self.renderer.as_mut() {
                    Some(renderer) => renderer.free_shader(resource_id),
                    None => Err("renderer is not initialized".to_owned()),
                };
                match result {
                    Ok(()) => self.send_shader_freed(reply_to, resource_id),
                    Err(message) => self.send_error(ErrorReply::resource_not_found(
                        reply_to,
                        resource_id,
                        message,
                    )),
                }
            }
            Command::UniformFloat {
                reply_to,
                resource_id,
                name,
                value,
            } => {
                let result = match self.renderer.as_mut() {
                    Some(renderer) => renderer.set_float(resource_id, &name, value),
                    None => Err("renderer is not initialized".to_owned()),
                };
                if let Err(message) = result {
                    self.send_error(ErrorReply::bad_argument(reply_to, resource_id, message));
                }
            }
            Command::UniformSet {
                reply_to,
                resource_id,
                name,
                value,
                duration,
                mode,
            } => {
                let result = self
                    .renderer
                    .as_mut()
                    .ok_or_else(|| "renderer is not initialized".to_owned())
                    .and_then(|renderer| {
                        renderer.set_control(resource_id, &name, value, duration, mode)
                    });
                if let Err(message) = result {
                    self.send_error(ErrorReply::bad_argument(reply_to, resource_id, message));
                }
            }
            Command::UniformGet {
                reply_to,
                resource_id,
                name,
                request_id,
            } => {
                let result = self
                    .renderer
                    .as_mut()
                    .ok_or_else(|| "renderer is not initialized".to_owned())
                    .and_then(|renderer| renderer.get_control(resource_id, &name));
                match result {
                    Ok(value) => {
                        if let Err(error) = self.osc.replies().send_uniform_value(
                            reply_to,
                            resource_id,
                            &name,
                            request_id,
                            &value,
                        ) {
                            eprintln!("SCShader OSC reply error: {error}");
                        }
                    }
                    Err(message) => {
                        self.send_error(ErrorReply::bad_argument(reply_to, resource_id, message))
                    }
                }
            }
            Command::TextureCreate {
                reply_to,
                resource_id,
                path,
            } => {
                self.last_reply_target = Some(reply_to);
                let result = self
                    .renderer
                    .as_mut()
                    .ok_or_else(|| "renderer is not initialized".to_owned())
                    .and_then(|renderer| renderer.create_texture(resource_id, path.into()));
                if let Err(message) = result {
                    let mut error = ErrorReply::bad_argument(reply_to, resource_id, message);
                    error.code = "E_TEXTURE_LOAD";
                    self.send_error(error);
                }
            }
            Command::TextureFree {
                reply_to,
                resource_id,
            } => {
                self.last_reply_target = Some(reply_to);
                let result = self
                    .renderer
                    .as_mut()
                    .ok_or_else(|| "renderer is not initialized".to_owned())
                    .and_then(|renderer| renderer.free_texture(resource_id));
                if let Err(message) = result {
                    self.send_error(ErrorReply::resource_not_found(
                        reply_to,
                        resource_id,
                        message,
                    ));
                }
            }
            Command::ShaderTexture {
                reply_to,
                resource_id,
                name,
                texture_id,
            } => {
                let result = self
                    .renderer
                    .as_mut()
                    .ok_or_else(|| "renderer is not initialized".to_owned())
                    .and_then(|renderer| renderer.set_texture(resource_id, &name, texture_id));
                if let Err(message) = result {
                    self.send_error(ErrorReply::bad_argument(reply_to, resource_id, message));
                }
            }
            Command::ShaderFeedback {
                reply_to,
                resource_id,
                source,
                amount,
            } => {
                let result = self
                    .renderer
                    .as_mut()
                    .ok_or_else(|| "renderer is not initialized".to_owned())
                    .and_then(|renderer| renderer.set_feedback(resource_id, &source, amount));
                if let Err(message) = result {
                    self.send_error(ErrorReply::bad_argument(reply_to, resource_id, message));
                }
            }
            Command::BufferCreate {
                reply_to,
                resource_id,
                length,
            } => {
                self.last_reply_target = Some(reply_to);
                let result = self
                    .renderer
                    .as_mut()
                    .ok_or_else(|| "renderer is not initialized".to_owned())
                    .and_then(|renderer| renderer.create_buffer(resource_id, length));
                if let Err(message) = result {
                    self.send_error(ErrorReply::bad_argument(reply_to, resource_id, message));
                }
            }
            Command::BufferWrite {
                reply_to,
                resource_id,
                start,
                values,
            } => {
                let result = self
                    .renderer
                    .as_mut()
                    .ok_or_else(|| "renderer is not initialized".to_owned())
                    .and_then(|renderer| renderer.write_buffer(resource_id, start, &values));
                if let Err(message) = result {
                    self.send_error(ErrorReply::bad_argument(reply_to, resource_id, message));
                }
            }
            Command::BufferFree {
                reply_to,
                resource_id,
            } => {
                self.last_reply_target = Some(reply_to);
                let result = self
                    .renderer
                    .as_mut()
                    .ok_or_else(|| "renderer is not initialized".to_owned())
                    .and_then(|renderer| renderer.free_buffer(resource_id));
                if let Err(message) = result {
                    self.send_error(ErrorReply::resource_not_found(
                        reply_to,
                        resource_id,
                        message,
                    ));
                }
            }
            Command::ShaderBuffer {
                reply_to,
                resource_id,
                name,
                buffer_id,
            } => {
                let result = self
                    .renderer
                    .as_mut()
                    .ok_or_else(|| "renderer is not initialized".to_owned())
                    .and_then(|renderer| renderer.set_buffer(resource_id, &name, buffer_id));
                if let Err(message) = result {
                    self.send_error(ErrorReply::bad_argument(reply_to, resource_id, message));
                }
            }
            Command::GraphSet {
                reply_to,
                resource_ids,
            } => {
                self.last_reply_target = Some(reply_to);
                let result = self
                    .renderer
                    .as_mut()
                    .ok_or_else(|| "renderer is not initialized".to_owned())
                    .and_then(|renderer| renderer.set_graph(resource_ids));
                if let Err(message) = result {
                    self.send_error(ErrorReply::bad_argument(reply_to, 0, message));
                }
            }
            Command::TimingMarker { reply_to, sequence } => {
                let scheduled_renderer_time = scheduled_time.unwrap_or(applied_renderer_time);
                self.last_reply_target = Some(reply_to);
                self.send_timing_marker(
                    reply_to,
                    sequence,
                    scheduled_renderer_time,
                    applied_renderer_time,
                );
            }
            Command::WindowResize {
                reply_to,
                width,
                height,
            } => {
                self.last_reply_target = Some(reply_to);
                if let Some(window) = self.window.as_ref() {
                    // Wayland can apply a client resize synchronously without
                    // emitting Resized. Use the returned physical size (which
                    // may also be the unchanged size if the compositor refuses).
                    if let Some(size) = window.request_inner_size(LogicalSize::new(width, height)) {
                        self.resize_window(size);
                    }
                } else {
                    self.send_error(ErrorReply::window(
                        reply_to,
                        "renderer window is not initialized",
                    ));
                }
            }
            Command::WindowTitle { reply_to, title } => {
                self.last_reply_target = Some(reply_to);
                if let Some(window) = self.window.as_ref() {
                    window.set_title(&title);
                    self.title = title;
                    self.send_window_metrics(reply_to);
                } else {
                    self.send_error(ErrorReply::window(
                        reply_to,
                        "renderer window is not initialized",
                    ));
                }
            }
            Command::WindowPosition { reply_to, x, y } => {
                self.last_reply_target = Some(reply_to);
                if let Some(window) = self.window.as_ref() {
                    window.set_outer_position(LogicalPosition::new(x, y));
                    self.send_window_metrics(reply_to);
                } else {
                    self.send_error(ErrorReply::window(
                        reply_to,
                        "renderer window is not initialized",
                    ));
                }
            }
            Command::WindowFullscreen { reply_to, enabled } => {
                self.last_reply_target = Some(reply_to);
                if let Some(window) = self.window.as_ref() {
                    window.set_fullscreen(
                        enabled.then(|| Fullscreen::Borderless(window.current_monitor())),
                    );
                    self.fullscreen = enabled;
                    self.send_window_metrics(reply_to);
                } else {
                    self.send_error(ErrorReply::window(
                        reply_to,
                        "renderer window is not initialized",
                    ));
                }
            }
            Command::WindowBorderless { reply_to, enabled } => {
                self.last_reply_target = Some(reply_to);
                if let Some(window) = self.window.as_ref() {
                    window.set_decorations(!enabled);
                    self.borderless = enabled;
                    self.send_window_metrics(reply_to);
                } else {
                    self.send_error(ErrorReply::window(
                        reply_to,
                        "renderer window is not initialized",
                    ));
                }
            }
            Command::WindowVsync { reply_to, enabled } => {
                self.last_reply_target = Some(reply_to);
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.set_vsync(enabled);
                    self.vsync = enabled;
                    self.send_window_metrics(reply_to);
                } else {
                    self.send_error(ErrorReply::window(
                        reply_to,
                        "renderer window is not initialized",
                    ));
                }
            }
            Command::WindowCursorVisible { reply_to, visible } => {
                self.last_reply_target = Some(reply_to);
                if let Some(window) = self.window.as_ref() {
                    window.set_cursor_visible(visible);
                    self.cursor_visible = visible;
                    self.send_window_metrics(reply_to);
                } else {
                    self.send_error(ErrorReply::window(
                        reply_to,
                        "renderer window is not initialized",
                    ));
                }
            }
            Command::WindowFront { reply_to } => {
                self.last_reply_target = Some(reply_to);
                if let Some(window) = self.window.as_ref() {
                    window.set_visible(true);
                    window.focus_window();
                    self.send_window_metrics(reply_to);
                } else {
                    self.send_error(ErrorReply::window(
                        reply_to,
                        "renderer window is not initialized",
                    ));
                }
            }
            Command::WindowMetrics { reply_to } => {
                self.last_reply_target = Some(reply_to);
                self.send_window_metrics(reply_to);
            }
            Command::WindowInputEnabled { reply_to, enabled } => {
                self.last_reply_target = Some(reply_to);
                self.input_enabled = enabled;
                if !enabled {
                    self.pending_mouse_position = None;
                }
                self.send_window_metrics(reply_to);
            }
            Command::WindowClose { reply_to } => {
                self.last_reply_target = Some(reply_to);
                event_loop.exit();
            }
            Command::Quit => event_loop.exit(),
        }
    }

    fn renderer_time(&self) -> f64 {
        self.started_at.elapsed().as_secs_f64()
    }

    fn send_ready(&self, reply_to: ReplyTarget) {
        let Some(renderer) = self.renderer.as_ref() else {
            self.send_error(ErrorReply::protocol(
                reply_to,
                "renderer is not initialized",
            ));
            return;
        };

        if let Err(error) =
            self.osc
                .replies()
                .send_ready(reply_to, renderer.backend_name(), renderer.device_name())
        {
            eprintln!("SCShader OSC reply error: {error}");
        }

        if let Some(message) = renderer.startup_diagnostic() {
            self.send_error(ErrorReply::shader_validate(reply_to, message));
        }
    }

    fn send_error(&self, error: ErrorReply) {
        if let Err(message) = self.osc.replies().send_error(&error) {
            eprintln!("SCShader OSC reply error: {message}");
        }
    }

    fn send_shader_created(&self, reply_to: ReplyTarget, resource_id: i32) {
        if let Err(error) = self
            .osc
            .replies()
            .send_shader_created(reply_to, resource_id)
        {
            eprintln!("SCShader OSC reply error: {error}");
        }
    }

    fn send_shader_reloaded(&self, reply_to: ReplyTarget, resource_id: i32) {
        if let Err(error) = self
            .osc
            .replies()
            .send_shader_reloaded(reply_to, resource_id)
        {
            eprintln!("SCShader OSC reply error: {error}");
        }
    }

    fn send_shader_freed(&self, reply_to: ReplyTarget, resource_id: i32) {
        if let Err(error) = self.osc.replies().send_shader_freed(reply_to, resource_id) {
            eprintln!("SCShader OSC reply error: {error}");
        }
    }

    fn send_shader_reflection(&self, reply_to: ReplyTarget, resource_id: i32) {
        let Some(controls) = self
            .renderer
            .as_ref()
            .and_then(|renderer| renderer.controls(resource_id))
        else {
            return;
        };
        if let Err(error) =
            self.osc
                .replies()
                .send_shader_reflection(reply_to, resource_id, controls)
        {
            eprintln!("SCShader OSC reply error: {error}");
        }
    }

    fn send_timing_marker(
        &self,
        reply_to: ReplyTarget,
        sequence: i32,
        scheduled_renderer_time: f64,
        applied_renderer_time: f64,
    ) {
        if let Err(error) = self.osc.replies().send_timing_marker(
            reply_to,
            sequence,
            scheduled_renderer_time,
            applied_renderer_time,
        ) {
            eprintln!("SCShader OSC reply error: {error}");
        }
    }

    fn resize_window(&mut self, size: PhysicalSize<u32>) {
        if let Some(renderer) = self.renderer.as_mut() {
            renderer.resize(size);
        }
        if let Some(reply_to) = self.last_reply_target {
            self.send_window_metrics(reply_to);
        }
        self.send_resize_input();
    }

    fn send_window_metrics(&self, reply_to: ReplyTarget) {
        let Some(metrics) = self.window_metrics() else {
            self.send_error(ErrorReply::window(
                reply_to,
                "renderer window is not initialized",
            ));
            return;
        };
        if let Err(error) = self.osc.replies().send_window_metrics(reply_to, &metrics) {
            eprintln!("SCShader OSC reply error: {error}");
        }
    }

    fn window_metrics(&self) -> Option<WindowMetrics> {
        let window = self.window.as_ref()?;
        let pixel_size = window.inner_size();
        let pixel_ratio = window.scale_factor();
        let logical_size = pixel_size.to_logical::<f64>(pixel_ratio);
        let position = window
            .outer_position()
            .unwrap_or_else(|_| PhysicalPosition::new(0, 0))
            .to_logical::<f64>(pixel_ratio);
        Some(WindowMetrics {
            logical_width: f64_to_u32(logical_size.width),
            logical_height: f64_to_u32(logical_size.height),
            pixel_width: pixel_size.width,
            pixel_height: pixel_size.height,
            pixel_ratio,
            position_x: f64_to_i32(position.x),
            position_y: f64_to_i32(position.y),
            fullscreen: self.fullscreen,
            borderless: self.borderless,
            vsync: self.vsync,
            cursor_visible: self.cursor_visible,
            title: self.title.clone(),
        })
    }

    fn send_resize_input(&self) {
        let Some(reply_to) = self.input_reply_target() else {
            return;
        };
        let Some(metrics) = self.window_metrics() else {
            return;
        };
        if let Err(error) = self.osc.replies().send_input_resize(reply_to, &metrics) {
            eprintln!("SCShader OSC reply error: {error}");
        }
    }

    fn send_focus_input(&self, focused: bool) {
        let Some(reply_to) = self.input_reply_target() else {
            return;
        };
        if let Err(error) = self.osc.replies().send_input_focus(reply_to, focused) {
            eprintln!("SCShader OSC reply error: {error}");
        }
    }

    fn send_key_input(&self, event: &KeyEvent) {
        let Some(reply_to) = self.input_reply_target() else {
            return;
        };
        let state = match event.state {
            ElementState::Pressed => "down",
            ElementState::Released => "up",
        };
        if let Err(error) =
            self.osc
                .replies()
                .send_input_key(reply_to, state, &key_name(event), event.repeat)
        {
            eprintln!("SCShader OSC reply error: {error}");
        }
    }

    fn send_mouse_input(&self, kind: &str, position: PhysicalPosition<f64>, button: i32) {
        let Some(reply_to) = self.input_reply_target() else {
            return;
        };
        let (x, y) = self.normalized_mouse_position(position);
        if let Err(error) = self
            .osc
            .replies()
            .send_input_mouse(reply_to, kind, x, y, button)
        {
            eprintln!("SCShader OSC reply error: {error}");
        }
    }

    fn flush_mouse_input(&mut self) {
        if let Some(position) = self.pending_mouse_position.take() {
            self.send_mouse_input("move", position, -1);
        }
    }

    fn normalized_mouse_position(&self, position: PhysicalPosition<f64>) -> (f64, f64) {
        let Some(window) = self.window.as_ref() else {
            return (0.0, 0.0);
        };
        let size = window.inner_size();
        let width = f64::from(size.width.max(1));
        let height = f64::from(size.height.max(1));
        (
            (position.x / width).clamp(0.0, 1.0),
            (position.y / height).clamp(0.0, 1.0),
        )
    }

    fn input_reply_target(&self) -> Option<ReplyTarget> {
        if self.input_enabled {
            self.last_reply_target
        } else {
            None
        }
    }

    fn poll_hot_reload(&mut self) {
        let Some(reply_to) = self.last_reply_target else {
            return;
        };
        let outcomes = self
            .renderer
            .as_mut()
            .map(|renderer| renderer.poll_hot_reload())
            .unwrap_or_default();
        for outcome in outcomes {
            match outcome {
                HotReload::Reloaded { resource_id } => {
                    self.send_shader_reflection(reply_to, resource_id);
                    self.send_shader_reloaded(reply_to, resource_id);
                }
                HotReload::Failed {
                    resource_id,
                    message,
                } => self.send_error(ErrorReply::shader_compile(reply_to, resource_id, message)),
            }
        }
    }
}

impl App {
    fn fatal_gpu(&mut self, event_loop: &ActiveEventLoop, message: String) {
        eprintln!("SCShader fatal GPU error: {message}");
        if let Some(target) = self.last_reply_target {
            let mut error = ErrorReply::gpu(target, message);
            error.code = "E_GPU_FATAL";
            self.send_error(error);
        }
        // Distinct from command-line/initialization failure (1) and normal quit (0).
        self.exit_code = 70;
        event_loop.exit();
    }

    fn check_gpu(&mut self, event_loop: &ActiveEventLoop) -> bool {
        if event_loop.exiting() {
            return false;
        }
        let Some(issue) = self.renderer.as_ref().and_then(Renderer::gpu_issue) else {
            return !event_loop.exiting();
        };
        if !issue.recoverable || self.recovery_count != 0 {
            self.fatal_gpu(
                event_loop,
                format!("{}; no further recovery attempts", issue.message),
            );
            return false;
        }
        self.recovery_count += 1;
        eprintln!("SCShader GPU recovery attempt 1: {}", issue.message);
        if let Some(target) = self.last_reply_target {
            let mut error = ErrorReply::gpu(
                target,
                format!("{}; attempting one device restart", issue.message),
            );
            error.severity = "warning";
            self.send_error(error);
        }
        let previous = self.renderer.take().expect("GPU issue requires a renderer");
        match previous.recover(self.vsync) {
            Ok(renderer) => {
                self.renderer = Some(renderer);
                eprintln!("SCShader GPU recovery complete; feedback history cleared");
                if let Some(target) = self.last_reply_target {
                    if let Err(error) = self.osc.replies().send_gpu_recovered(
                        target,
                        self.recovery_count,
                        self.renderer.as_ref().unwrap().backend_name(),
                        self.renderer.as_ref().unwrap().device_name(),
                    ) {
                        eprintln!("SCShader OSC reply error: {error}");
                    }
                    self.send_window_metrics(target);
                }
                true
            }
            Err(message) => {
                self.fatal_gpu(event_loop, format!("GPU recovery failed: {message}"));
                false
            }
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none()
            && let Err(message) = self.initialize(event_loop)
        {
            crate::report_launch_error(&format!("SCShader initialization error: {message}"));
            self.exit_code = 1;
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if self
            .window
            .as_ref()
            .is_none_or(|window| window.id() != window_id)
        {
            return;
        }
        if !self.check_gpu(event_loop) {
            return;
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => self.resize_window(size),
            WindowEvent::ScaleFactorChanged { .. } => {
                if let Some(reply_to) = self.last_reply_target {
                    self.send_window_metrics(reply_to);
                }
                self.send_resize_input();
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.last_mouse_position = position;
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.set_mouse_position(position.x, position.y);
                }
                if self.input_enabled {
                    self.pending_mouse_position = Some(position);
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.set_mouse_pressed(state == ElementState::Pressed);
                }
                let kind = match state {
                    ElementState::Pressed => "down",
                    ElementState::Released => "up",
                };
                self.send_mouse_input(kind, self.last_mouse_position, mouse_button_code(button));
            }
            WindowEvent::KeyboardInput { event, .. } => self.send_key_input(&event),
            WindowEvent::Focused(focused) => self.send_focus_input(focused),
            WindowEvent::CursorLeft { .. } => self.pending_mouse_position = None,
            WindowEvent::RedrawRequested => {
                self.process_commands(event_loop);
                if !self.check_gpu(event_loop) {
                    return;
                }
                self.flush_mouse_input();
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.set_schedule_stats(self.scheduler.len(), self.late_events);
                }
                if let Some(renderer) = self.renderer.as_mut()
                    && let Err(message) = renderer.render()
                {
                    // A device-lost callback can accompany a surface failure.
                    if self
                        .renderer
                        .as_ref()
                        .and_then(Renderer::gpu_issue)
                        .is_some()
                    {
                        self.check_gpu(event_loop);
                    } else {
                        self.fatal_gpu(event_loop, message);
                    }
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(renderer) = &self.renderer {
            renderer.poll_gpu();
        }
        self.process_commands(event_loop);
        if !self.check_gpu(event_loop) {
            return;
        }
        self.poll_hot_reload();
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }
}

fn f64_to_u32(value: f64) -> u32 {
    value.round().clamp(0.0, f64::from(u32::MAX)) as u32
}

fn f64_to_i32(value: f64) -> i32 {
    value
        .round()
        .clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32
}

fn mouse_button_code(button: MouseButton) -> i32 {
    match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Back => 3,
        MouseButton::Forward => 4,
        MouseButton::Other(value) => i32::from(value),
    }
}

fn key_name(event: &KeyEvent) -> String {
    match &event.logical_key {
        Key::Character(character) => character.to_string(),
        Key::Named(named) => format!("{named:?}"),
        Key::Dead(Some(character)) => format!("Dead({character})"),
        Key::Dead(None) => "Dead".to_owned(),
        Key::Unidentified(_) => "Unidentified".to_owned(),
    }
}
