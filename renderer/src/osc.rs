use std::{
    io::ErrorKind,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{Receiver, SyncSender, TrySendError, sync_channel},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime},
};

use rosc::{OscMessage, OscPacket, OscTime, OscType, decoder, encoder};

use crate::diagnostics::{DiagnosticBudget, bounded_message};

use crate::protocol::{
    Command, ErrorReply, PROTOCOL_MAJOR, RENDERER_VERSION, ReplyTarget, parse_message,
};

const QUEUE_CAPACITY: usize = 256;
const MAX_DATAGRAM_SIZE: usize = 65_535;
const MAX_PACKET_ELEMENTS: usize = 256;
const MAX_BUNDLE_DEPTH: usize = 16;
const OSC_UNIX_OFFSET_SECONDS: u32 = 2_208_988_800;

#[derive(Debug)]
pub struct IncomingCommand {
    pub command: Command,
    /// A renderer-monotonic target time derived from the OSC bundle timetag.
    /// `None` means an ordinary immediate OSC message.
    pub target_time: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct WindowMetrics {
    pub logical_width: u32,
    pub logical_height: u32,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub pixel_ratio: f64,
    pub position_x: i32,
    pub position_y: i32,
    pub fullscreen: bool,
    pub borderless: bool,
    pub vsync: bool,
    pub cursor_visible: bool,
    pub title: String,
}

struct ReceiveContext<'a> {
    sender: &'a SyncSender<IncomingCommand>,
    replies: &'a ReplySender,
    dropped_updates: &'a AtomicU64,
    stop: &'a AtomicBool,
    clock_started_at: Instant,
}

pub struct OscServer {
    receiver: Receiver<IncomingCommand>,
    replies: ReplySender,
    dropped_updates: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl OscServer {
    pub fn dropped_updates(&self) -> u64 {
        self.dropped_updates.load(Ordering::Relaxed)
    }
    pub fn bind(port: u16, clock_started_at: Instant) -> Result<Self, String> {
        let socket = Arc::new(
            UdpSocket::bind((Ipv4Addr::LOCALHOST, port))
                .map_err(|error| format!("could not bind OSC 127.0.0.1:{port}: {error}"))?,
        );
        socket
            .set_read_timeout(Some(Duration::from_millis(100)))
            .map_err(|error| format!("could not configure OSC socket: {error}"))?;

        let (sender, receiver) = sync_channel(QUEUE_CAPACITY);
        let replies = ReplySender {
            socket: Arc::clone(&socket),
            diagnostics: Arc::new(DiagnosticBudget::new(Instant::now())),
        };
        let thread_replies = replies.clone();
        let dropped_updates = Arc::new(AtomicU64::new(0));
        let thread_dropped = Arc::clone(&dropped_updates);
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);

        let thread = thread::Builder::new()
            .name("scshader-osc".to_owned())
            .spawn(move || {
                receive_loop(
                    socket,
                    sender,
                    thread_replies,
                    thread_dropped,
                    thread_stop,
                    clock_started_at,
                );
            })
            .map_err(|error| format!("could not start OSC receiver thread: {error}"))?;

        Ok(Self {
            receiver,
            replies,
            dropped_updates,
            stop,
            thread: Some(thread),
        })
    }

    pub fn try_recv(&self) -> Result<IncomingCommand, std::sync::mpsc::TryRecvError> {
        let result = self.receiver.try_recv();
        if result.is_ok() {
            crate::timing::received();
        }
        result
    }

    pub fn replies(&self) -> &ReplySender {
        &self.replies
    }
}

impl Drop for OscServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take()
            && let Err(error) = thread.join()
        {
            eprintln!("SCShader OSC receiver thread panicked: {error:?}");
        }
        let dropped = self.dropped_updates.load(Ordering::Relaxed);
        if dropped > 0 {
            eprintln!("SCShader dropped {dropped} continuous OSC update(s) while overloaded");
        }
    }
}

#[derive(Clone)]
pub struct ReplySender {
    socket: Arc<UdpSocket>,
    diagnostics: Arc<DiagnosticBudget>,
}

impl ReplySender {
    pub fn send_ready(
        &self,
        target: ReplyTarget,
        backend: &str,
        device: &str,
    ) -> Result<(), String> {
        let address = if target.versioned {
            "/scshader/v1/ready"
        } else {
            "/scshader/ready"
        };
        self.send(
            target,
            address,
            vec![
                OscType::Int(PROTOCOL_MAJOR),
                OscType::String(RENDERER_VERSION.to_owned()),
                OscType::String(backend.to_owned()),
                OscType::String(device.to_owned()),
            ],
        )
    }

    pub fn send_pong(
        &self,
        target: ReplyTarget,
        sequence: i32,
        client_time: f64,
        renderer_time: f64,
    ) -> Result<(), String> {
        let address = if target.versioned {
            "/scshader/v1/pong"
        } else {
            "/scshader/pong"
        };
        self.send(
            target,
            address,
            vec![
                OscType::Int(sequence),
                OscType::Double(client_time),
                OscType::Double(renderer_time),
            ],
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn send_status(
        &self,
        target: ReplyTarget,
        fps: f64,
        frame_index: u64,
        gpu_frame_ms: f64,
        cpu_frame_ms: f64,
        scheduled_event_count: u64,
        shader_count: i32,
        texture_count: i32,
        buffer_count: i32,
        window_count: i32,
        scheduled_bytes: usize,
        rejected_scheduled: u64,
        dropped_continuous: u64,
        late_events: u64,
        texture_bytes: u64,
        show_stats: bool,
        recovery_count: u32,
        last_compile_ok: bool,
    ) -> Result<(), String> {
        let frame_index = i32::try_from(frame_index).unwrap_or(i32::MAX);
        let scheduled_event_count = i32::try_from(scheduled_event_count).unwrap_or(i32::MAX);
        self.send(
            target,
            "/scshader/v1/status.reply",
            vec![
                OscType::Double(fps),
                OscType::Int(frame_index),
                OscType::Double(gpu_frame_ms),
                OscType::Double(cpu_frame_ms),
                OscType::Int(scheduled_event_count),
                OscType::Int(shader_count),
                OscType::Int(texture_count),
                OscType::Int(buffer_count),
                OscType::Int(window_count),
                OscType::Int(i32::try_from(scheduled_bytes).unwrap_or(i32::MAX)),
                OscType::Double(rejected_scheduled as f64),
                OscType::Double(dropped_continuous as f64),
                OscType::Double(late_events as f64),
                OscType::Double(texture_bytes as f64),
                OscType::Int(i32::from(show_stats)),
                OscType::Int(recovery_count as i32),
                OscType::Int(i32::from(last_compile_ok)),
                OscType::Double(self.diagnostics.suppressed() as f64),
            ],
        )
    }

    pub fn send_shader_created(&self, target: ReplyTarget, resource_id: i32) -> Result<(), String> {
        self.send(
            target,
            "/scshader/v1/shader/created",
            vec![OscType::Int(resource_id)],
        )
    }

    pub fn send_gpu_recovered(
        &self,
        target: ReplyTarget,
        count: u32,
        backend: &str,
        device: &str,
    ) -> Result<(), String> {
        self.send(
            target,
            "/scshader/v1/gpu/recovered",
            vec![
                OscType::Int(count as i32),
                OscType::String(
                    "resources restored; feedback and intermediate frame history cleared"
                        .to_owned(),
                ),
                OscType::String(backend.to_owned()),
                OscType::String(device.to_owned()),
            ],
        )
    }

    #[cfg(feature = "gpu-test-hooks")]
    pub fn send_test_pixel(&self, target: ReplyTarget, bytes: Vec<u8>) -> Result<(), String> {
        self.send(
            target,
            "/scshader/v1/test/pixel.reply",
            vec![OscType::Blob(bytes)],
        )
    }

    pub fn send_schedule_cleared(&self, target: ReplyTarget, count: usize) -> Result<(), String> {
        self.send(
            target,
            "/scshader/v1/schedule/cleared",
            vec![OscType::Int(i32::try_from(count).unwrap_or(i32::MAX))],
        )
    }

    pub fn send_shader_reloaded(
        &self,
        target: ReplyTarget,
        resource_id: i32,
    ) -> Result<(), String> {
        self.send(
            target,
            "/scshader/v1/shader/reloaded",
            vec![OscType::Int(resource_id)],
        )
    }

    pub fn send_shader_freed(&self, target: ReplyTarget, resource_id: i32) -> Result<(), String> {
        self.send(
            target,
            "/scshader/v1/shader/freed",
            vec![OscType::Int(resource_id)],
        )
    }

    pub fn send_shader_reflection(
        &self,
        target: ReplyTarget,
        resource_id: i32,
        controls: &crate::controls::Controls,
    ) -> Result<(), String> {
        let mut args = vec![OscType::Int(resource_id)];
        for (name, kind) in controls.reflection() {
            args.push(OscType::String(name.to_owned()));
            args.push(OscType::String(kind.name().to_owned()));
        }
        self.send(target, "/scshader/v1/shader/reflection", args)
    }

    pub fn send_uniform_value(
        &self,
        target: ReplyTarget,
        resource_id: i32,
        name: &str,
        request_id: i32,
        value: &crate::controls::ControlValue,
    ) -> Result<(), String> {
        self.send(
            target,
            "/scshader/v1/uniform/value",
            vec![
                resource_id.into(),
                name.into(),
                request_id.into(),
                value.kind.name().into(),
                OscType::Blob(value.blob()),
            ],
        )
    }

    pub fn send_timing_marker(
        &self,
        target: ReplyTarget,
        sequence: i32,
        scheduled_renderer_time: f64,
        applied_renderer_time: f64,
    ) -> Result<(), String> {
        self.send(
            target,
            "/scshader/v1/timing/marker.reply",
            vec![
                OscType::Int(sequence),
                OscType::Double(scheduled_renderer_time),
                OscType::Double(applied_renderer_time),
            ],
        )
    }

    pub fn send_window_metrics(
        &self,
        target: ReplyTarget,
        metrics: &WindowMetrics,
    ) -> Result<(), String> {
        self.send(
            target,
            "/scshader/v1/window/metrics.reply",
            vec![
                OscType::Int(i32::try_from(metrics.logical_width).unwrap_or(i32::MAX)),
                OscType::Int(i32::try_from(metrics.logical_height).unwrap_or(i32::MAX)),
                OscType::Int(i32::try_from(metrics.pixel_width).unwrap_or(i32::MAX)),
                OscType::Int(i32::try_from(metrics.pixel_height).unwrap_or(i32::MAX)),
                OscType::Double(metrics.pixel_ratio),
                OscType::Int(metrics.position_x),
                OscType::Int(metrics.position_y),
                OscType::Int(i32::from(metrics.fullscreen)),
                OscType::Int(i32::from(metrics.borderless)),
                OscType::Int(i32::from(metrics.vsync)),
                OscType::Int(i32::from(metrics.cursor_visible)),
                OscType::String(metrics.title.clone()),
            ],
        )
    }

    pub fn send_input_mouse(
        &self,
        target: ReplyTarget,
        kind: &str,
        x: f64,
        y: f64,
        button: i32,
    ) -> Result<(), String> {
        self.send(
            target,
            "/scshader/v1/input/mouse",
            vec![
                OscType::String(kind.to_owned()),
                OscType::Double(x),
                OscType::Double(y),
                OscType::Int(button),
            ],
        )
    }

    pub fn send_input_key(
        &self,
        target: ReplyTarget,
        state: &str,
        key: &str,
        repeat: bool,
    ) -> Result<(), String> {
        self.send(
            target,
            "/scshader/v1/input/key",
            vec![
                OscType::String(state.to_owned()),
                OscType::String(key.to_owned()),
                OscType::Int(i32::from(repeat)),
            ],
        )
    }

    pub fn send_input_focus(&self, target: ReplyTarget, focused: bool) -> Result<(), String> {
        self.send(
            target,
            "/scshader/v1/input/focus",
            vec![OscType::Int(i32::from(focused))],
        )
    }

    pub fn send_input_resize(
        &self,
        target: ReplyTarget,
        metrics: &WindowMetrics,
    ) -> Result<(), String> {
        self.send(
            target,
            "/scshader/v1/input/resize",
            vec![
                OscType::Int(i32::try_from(metrics.logical_width).unwrap_or(i32::MAX)),
                OscType::Int(i32::try_from(metrics.logical_height).unwrap_or(i32::MAX)),
                OscType::Int(i32::try_from(metrics.pixel_width).unwrap_or(i32::MAX)),
                OscType::Int(i32::try_from(metrics.pixel_height).unwrap_or(i32::MAX)),
                OscType::Double(metrics.pixel_ratio),
            ],
        )
    }

    pub fn send_error(&self, error: &ErrorReply) -> Result<(), String> {
        // Internal GPU loss/fatal diagnostics cannot be selected by an OSC sender.
        let critical = matches!(error.code, "E_GPU_DEVICE_LOST" | "E_GPU_FATAL");
        if !self.diagnostics.allow(Instant::now(), critical) {
            return Ok(());
        }
        let message = bounded_message(&error.message);
        eprintln!(
            "SCShader {} error {} from {}: {}{}",
            error.subsystem,
            error.code,
            error.reply_to.address,
            message,
            if message.len() < error.message.len() {
                " [truncated]"
            } else {
                ""
            }
        );
        let address = if error.reply_to.versioned {
            "/scshader/v1/error"
        } else {
            "/scshader/error"
        };
        self.send(
            error.reply_to,
            address,
            vec![
                OscType::String(error.severity.to_owned()),
                OscType::String(error.subsystem.to_owned()),
                OscType::Int(error.resource_id),
                OscType::String(error.code.to_owned()),
                OscType::String(if message.len() < error.message.len() {
                    format!("{message} [truncated]")
                } else {
                    message.to_owned()
                }),
            ],
        )
    }

    fn send(&self, target: ReplyTarget, address: &str, args: Vec<OscType>) -> Result<(), String> {
        let packet = OscPacket::Message(OscMessage {
            addr: address.to_owned(),
            args,
        });
        let bytes = encoder::encode(&packet)
            .map_err(|error| format!("could not encode OSC reply: {error}"))?;
        self.socket
            .send_to(&bytes, target.address)
            .map_err(|error| format!("could not send OSC reply to {}: {error}", target.address))?;
        Ok(())
    }
}

fn receive_loop(
    socket: Arc<UdpSocket>,
    sender: SyncSender<IncomingCommand>,
    replies: ReplySender,
    dropped_updates: Arc<AtomicU64>,
    stop: Arc<AtomicBool>,
    clock_started_at: Instant,
) {
    let mut buffer = [0_u8; MAX_DATAGRAM_SIZE];

    while !stop.load(Ordering::Relaxed) {
        match socket.recv_from(&mut buffer) {
            Ok((length, source)) => {
                if let Err(message) = validate_packet_shape(&buffer[..length]) {
                    report_error(
                        &replies,
                        &ErrorReply::protocol(ReplyTarget::from_sender(source, true), message),
                    );
                    continue;
                }
                match decoder::decode_udp(&buffer[..length]) {
                    Ok(([], packet)) => {
                        let context = ReceiveContext {
                            sender: &sender,
                            replies: &replies,
                            dropped_updates: &dropped_updates,
                            stop: &stop,
                            clock_started_at,
                        };
                        if !route_packet(packet, source, &context) {
                            return;
                        }
                    }
                    Ok((_remainder, _packet)) => {
                        let error = ErrorReply::protocol(
                            ReplyTarget::from_sender(source, true),
                            "OSC datagram contains trailing bytes",
                        );
                        report_error(&replies, &error);
                    }
                    Err(error) => {
                        let reply = ErrorReply::protocol(
                            ReplyTarget::from_sender(source, true),
                            format!("malformed OSC datagram: {error}"),
                        );
                        report_error(&replies, &reply);
                    }
                }
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(error) => {
                eprintln!("SCShader OSC receive error: {error}");
            }
        }
    }
}

fn route_packet(packet: OscPacket, source: SocketAddr, context: &ReceiveContext<'_>) -> bool {
    match prepare_packet(packet, source, None, None, context.clock_started_at) {
        Ok(incoming) => enqueue(
            incoming,
            context.sender,
            context.replies,
            context.dropped_updates,
            context.stop,
        ),
        Err(error) => {
            report_error(context.replies, &error);
            true
        }
    }
}

fn prepare_packet(
    packet: OscPacket,
    source: SocketAddr,
    parent_tag: Option<OscTime>,
    parent_time: Option<f64>,
    started_at: Instant,
) -> Result<IncomingCommand, ErrorReply> {
    let bad = |message| ErrorReply::protocol(ReplyTarget::from_sender(source, true), message);
    match packet {
        OscPacket::Message(message) => Ok(IncomingCommand {
            command: parse_message(message, source)?,
            target_time: parent_time,
        }),
        OscPacket::Bundle(bundle) => {
            let immediate = bundle.timetag.seconds == 0 && bundle.timetag.fractional == 1;
            let tag = if immediate {
                parent_tag
            } else {
                Some(bundle.timetag)
            };
            if let (Some(parent), Some(child)) = (parent_tag, tag)
                && (child.seconds, child.fractional) < (parent.seconds, parent.fractional)
            {
                return Err(bad("nested bundle timetag precedes its parent".to_owned()));
            }
            let target_time = if tag == parent_tag {
                parent_time
            } else {
                tag.map(|tag| timetag_to_renderer_time(tag, started_at))
                    .transpose()
                    .map_err(bad)?
                    .flatten()
            };
            if target_time.is_some_and(|target| {
                target - started_at.elapsed().as_secs_f64()
                    > crate::scheduler::MAX_LOOKAHEAD_SECONDS
            }) {
                return Err(bad(
                    "bundle timetag is more than one hour in the future".to_owned()
                ));
            }
            let commands = bundle
                .content
                .into_iter()
                .map(|child| {
                    prepare_packet(child, source, tag, target_time, started_at)
                        .map(|incoming| incoming.command)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(IncomingCommand {
                command: Command::Batch {
                    commands,
                    target_time,
                },
                target_time,
            })
        }
    }
}

/// Bound the recursive third-party decoder before entering it. OSC arrays are
/// not part of SCShader's protocol; vectors/matrices/data travel in flat blobs.
fn validate_packet_shape(bytes: &[u8]) -> Result<(), &'static str> {
    let mut pending = vec![(bytes, 0)];
    let mut elements = 1;
    while let Some((packet, depth)) = pending.pop() {
        if packet.is_empty() || packet.len() % 4 != 0 {
            return Err("OSC packet size must be a positive multiple of four");
        }
        if depth > MAX_BUNDLE_DEPTH {
            return Err("OSC bundle nesting exceeds 16 levels");
        }
        if packet.starts_with(b"#bundle\0") {
            if packet.len() < 16 {
                return Err("truncated OSC bundle header");
            }
            let mut offset = 16;
            while offset < packet.len() {
                let length_bytes: [u8; 4] = packet
                    .get(offset..offset + 4)
                    .ok_or("truncated OSC bundle element size")?
                    .try_into()
                    .expect("four bytes");
                let length = u32::from_be_bytes(length_bytes) as usize;
                offset += 4;
                if length == 0 || !length.is_multiple_of(4) || length > packet.len() - offset {
                    return Err("invalid OSC bundle element size");
                }
                elements += 1;
                if elements > MAX_PACKET_ELEMENTS {
                    return Err("OSC datagram exceeds 256 packet elements");
                }
                pending.push((&packet[offset..offset + length], depth + 1));
                offset += length;
            }
        } else {
            let address_end = packet
                .iter()
                .position(|byte| *byte == 0)
                .ok_or("unterminated OSC address")?;
            let tags_at = (address_end + 4) & !3;
            let tags = packet.get(tags_at..).ok_or("missing OSC type tags")?;
            let tags_end = tags
                .iter()
                .position(|byte| *byte == 0)
                .ok_or("unterminated OSC type tags")?;
            if tags[..tags_end]
                .iter()
                .any(|byte| *byte == b'[' || *byte == b']')
            {
                return Err("OSC arrays are not supported; use typed blobs");
            }
        }
    }
    Ok(())
}

fn enqueue(
    incoming: IncomingCommand,
    sender: &SyncSender<IncomingCommand>,
    replies: &ReplySender,
    dropped_updates: &AtomicU64,
    stop: &AtomicBool,
) -> bool {
    if incoming.command.is_continuous() {
        let (connected, diagnostic) = admit_continuous(incoming, sender, dropped_updates);
        if let Some(error) = diagnostic {
            crate::timing::queue_full();
            report_error(replies, &error);
        }
        connected
    } else {
        let mut pending = incoming;
        loop {
            match sender.try_send(pending) {
                Ok(()) => return true,
                Err(TrySendError::Full(command)) => {
                    if stop.load(Ordering::Relaxed) {
                        return false;
                    }
                    pending = command;
                    thread::sleep(Duration::from_millis(1));
                }
                Err(TrySendError::Disconnected(_)) => return false,
            }
        }
    }
}

// Kept independent of the socket so full/disconnected queues can be tested
// deterministically without relying on host speed or kernel UDP buffering.
fn admit_continuous(
    incoming: IncomingCommand,
    sender: &SyncSender<IncomingCommand>,
    dropped_updates: &AtomicU64,
) -> (bool, Option<ErrorReply>) {
    let reply_to = incoming.command.reply_target();
    match sender.try_send(incoming) {
        Ok(()) => (true, None),
        Err(TrySendError::Full(_)) => {
            let dropped = dropped_updates.fetch_add(1, Ordering::Relaxed) + 1;
            let error = if dropped.is_power_of_two() {
                reply_to.map(|target| ErrorReply::queue_full(target, dropped))
            } else {
                None
            };
            (true, error)
        }
        Err(TrySendError::Disconnected(_)) => (false, None),
    }
}

fn timetag_to_renderer_time(
    timetag: OscTime,
    clock_started_at: Instant,
) -> Result<Option<f64>, String> {
    // OSC's special "immediately" tag is 0/1 and has no calendar-time
    // meaning. Preserve it as an immediate renderer command.
    if timetag.seconds == 0 && timetag.fractional == 1 {
        return Ok(None);
    }
    if timetag.seconds < OSC_UNIX_OFFSET_SECONDS {
        return Err("OSC bundle timetag precedes the Unix epoch".to_owned());
    }

    // OSC bundles encode a UTC/NTP timetag. At receipt, bridge it onto the
    // renderer's monotonic clock using a single wall-clock delta. The queued
    // deadline is thereafter independent of wall-clock adjustments.
    let target_wall_time: SystemTime = timetag.into();
    let now_wall_time = SystemTime::now();
    let now_renderer_time = clock_started_at.elapsed().as_secs_f64();
    let delta = match target_wall_time.duration_since(now_wall_time) {
        Ok(duration) => duration.as_secs_f64(),
        Err(error) => -error.duration().as_secs_f64(),
    };
    Ok(Some(now_renderer_time + delta))
}

fn report_error(replies: &ReplySender, error: &ErrorReply) {
    if let Err(send_error) = replies.send_error(error) {
        eprintln!("SCShader OSC reply error: {send_error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn continuous_batch() -> IncomingCommand {
        IncomingCommand {
            target_time: None,
            command: Command::Batch {
                target_time: None,
                commands: (0..2)
                    .map(|id| Command::UniformFloat {
                        reply_to: ReplyTarget::from_sender("127.0.0.1:1".parse().unwrap(), true),
                        resource_id: id + 1,
                        name: "amount".to_owned(),
                        value: 0.5,
                    })
                    .collect(),
            },
        }
    }

    #[test]
    fn full_incoming_queue_drops_whole_batches_and_recovers() {
        let (sender, receiver) = sync_channel(1);
        let dropped = AtomicU64::new(0);
        assert!(continuous_batch().command.is_continuous());
        assert!(
            admit_continuous(continuous_batch(), &sender, &dropped)
                .1
                .is_none()
        );
        for count in 1_u64..=5 {
            let (connected, diagnostic) = admit_continuous(continuous_batch(), &sender, &dropped);
            assert!(connected);
            assert_eq!(diagnostic.is_some(), count.is_power_of_two());
            assert_eq!(dropped.load(Ordering::Relaxed), count);
        }
        let Command::Batch { commands, .. } = receiver.try_recv().unwrap().command else {
            panic!("lost batch")
        };
        assert_eq!(commands.len(), 2);
        assert!(
            receiver.try_recv().is_err(),
            "a rejected batch partially entered the queue"
        );
        let (connected, diagnostic) = admit_continuous(continuous_batch(), &sender, &dropped);
        assert!(connected && diagnostic.is_none());
        assert!(receiver.try_recv().is_ok());
        assert_eq!(dropped.load(Ordering::Relaxed), 5);
    }

    #[test]
    fn disconnected_incoming_queue_is_not_reported_as_overload() {
        let (sender, receiver) = sync_channel(1);
        drop(receiver);
        let dropped = AtomicU64::new(0);
        let (connected, diagnostic) = admit_continuous(continuous_batch(), &sender, &dropped);
        assert!(!connected && diagnostic.is_none());
        assert_eq!(dropped.load(Ordering::Relaxed), 0);
    }

    fn ping() -> OscPacket {
        OscPacket::Message(OscMessage {
            addr: "/scshader/v1/ping".to_owned(),
            args: vec![1.into(), 0.0_f64.into()],
        })
    }

    fn bundle(tag: OscTime, content: Vec<OscPacket>) -> OscPacket {
        OscPacket::Bundle(rosc::OscBundle {
            timetag: tag,
            content,
        })
    }

    #[test]
    fn preflight_rejects_excessive_depth_width_arrays_and_bad_lengths() {
        let immediate = OscTime {
            seconds: 0,
            fractional: 1,
        };
        let mut nested = ping();
        for _ in 0..MAX_BUNDLE_DEPTH + 1 {
            nested = bundle(immediate, vec![nested]);
        }
        assert!(validate_packet_shape(&encoder::encode(&nested).unwrap()).is_err());
        let many = bundle(
            immediate,
            (0..MAX_PACKET_ELEMENTS).map(|_| ping()).collect(),
        );
        assert!(validate_packet_shape(&encoder::encode(&many).unwrap()).is_err());
        assert!(validate_packet_shape(b"/x\0\0,[[[]]]\0").is_err());
        let mut malformed = encoder::encode(&bundle(immediate, vec![ping()])).unwrap();
        malformed[16..20].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(validate_packet_shape(&malformed).is_err());
        assert!(validate_packet_shape(b"/x\0").is_err());
    }

    #[test]
    fn bundles_are_accepted_as_one_atomic_command() {
        let packet = bundle(
            OscTime {
                seconds: 0,
                fractional: 1,
            },
            vec![ping(), ping()],
        );
        assert!(validate_packet_shape(&encoder::encode(&packet).unwrap()).is_ok());
        let incoming = prepare_packet(
            packet,
            "127.0.0.1:1".parse().unwrap(),
            None,
            None,
            Instant::now(),
        )
        .unwrap();
        assert!(incoming.target_time.is_none());
        let Command::Batch { commands, .. } = incoming.command else {
            panic!("expected atomic bundle");
        };
        assert_eq!(commands.len(), 2);
    }

    #[test]
    fn nested_immediate_inherits_parent_and_earlier_tag_is_rejected() {
        let now = Instant::now();
        let sender = "127.0.0.1:1".parse().unwrap();
        let tag = OscTime::try_from(SystemTime::now() + Duration::from_secs(5)).unwrap();
        let packet = bundle(
            tag,
            vec![bundle(
                OscTime {
                    seconds: 0,
                    fractional: 1,
                },
                vec![ping()],
            )],
        );
        let incoming = prepare_packet(packet, sender, None, None, now).unwrap();
        let parent_time = incoming.target_time;
        let Command::Batch { commands, .. } = incoming.command else {
            panic!("expected bundle");
        };
        let Command::Batch { target_time, .. } = commands[0] else {
            panic!("expected nested bundle");
        };
        assert_eq!(target_time, parent_time);
        let earlier = OscTime {
            seconds: tag.seconds - 1,
            fractional: tag.fractional,
        };
        assert!(
            prepare_packet(
                bundle(tag, vec![bundle(earlier, vec![ping()])]),
                sender,
                None,
                None,
                now
            )
            .is_err()
        );
    }

    #[test]
    fn invalid_member_rejects_the_whole_packet_and_far_future_is_rejected() {
        let sender = "127.0.0.1:1".parse().unwrap();
        let invalid = OscPacket::Message(OscMessage {
            addr: "/scshader/v1/no-such-command".into(),
            args: vec![],
        });
        assert!(
            prepare_packet(
                bundle(
                    OscTime {
                        seconds: 0,
                        fractional: 1
                    },
                    vec![ping(), invalid]
                ),
                sender,
                None,
                None,
                Instant::now()
            )
            .is_err()
        );
        let far_future = OscTime::try_from(SystemTime::now() + Duration::from_secs(3602)).unwrap();
        assert!(
            prepare_packet(
                bundle(far_future, vec![ping()]),
                sender,
                None,
                None,
                Instant::now()
            )
            .is_err()
        );
    }

    #[test]
    fn immediate_osc_timetag_is_not_scheduled() {
        assert_eq!(
            timetag_to_renderer_time(
                OscTime {
                    seconds: 0,
                    fractional: 1,
                },
                Instant::now(),
            )
            .expect("immediate timetag is valid"),
            None,
        );
    }

    #[test]
    fn future_osc_timetag_maps_to_future_monotonic_time() {
        let timetag = OscTime::try_from(SystemTime::now() + Duration::from_millis(100))
            .expect("current time is representable as OSC time");
        let target = timetag_to_renderer_time(timetag, Instant::now())
            .expect("calendar timetag is valid")
            .expect("calendar timetag is scheduled");
        assert!(target > 0.05);
    }

    #[test]
    fn pre_unix_osc_timetag_is_rejected_without_wall_clock_conversion() {
        let error = timetag_to_renderer_time(
            OscTime {
                seconds: 1,
                fractional: 0,
            },
            Instant::now(),
        )
        .expect_err("pre-Unix timetag must be rejected");
        assert!(error.contains("precedes"));
    }
}
