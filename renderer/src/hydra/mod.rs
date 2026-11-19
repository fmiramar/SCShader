//! Opt-in Hydra transport. Graph activation is gated on the native compiler.
pub mod camera;
pub mod capture;
pub mod clock;
pub mod compiler;
pub mod custom;
pub mod families;
pub mod gpu;
pub mod ir;
pub mod modulation;
pub mod output;
pub mod outputs;
pub mod parameters;
pub mod registry;
pub mod video;
use crate::protocol::{ErrorReply, ReplyTarget};
use rosc::OscType;
use std::{
    collections::VecDeque,
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};
const CHUNK: usize = 2048;

#[derive(Debug)]
pub struct Request {
    pub operation: String,
    pub args: Vec<OscType>,
    pub sender: SocketAddr,
}
impl Request {
    pub fn parse(operation: &str, args: Vec<OscType>, sender: SocketAddr) -> Result<Self, String> {
        fn integer(args: &[OscType], index: usize, low: i32, high: i32) -> bool {
            matches!(args.get(index), Some(OscType::Int(v)) if (low..=high).contains(v))
        }
        fn string(args: &[OscType], index: usize) -> bool {
            matches!(args.get(index), Some(OscType::String(v)) if !v.is_empty() && v.len()<=64 && v.is_ascii())
        }
        let scoped = string(&args, 0) && integer(&args, 1, 1, i32::MAX);
        let valid = match operation {
            "capabilities" => {
                args.len() == 3
                    && integer(&args, 0, 1, i32::MAX)
                    && integer(&args, 1, 1, i32::MAX)
                    && integer(&args, 2, 1, 65535)
            }
            "open" => {
                args.len() == 5
                    && integer(&args, 0, 1, i32::MAX)
                    && string(&args, 1)
                    && string(&args, 2)
                    && integer(&args, 3, 1, i32::MAX)
                    && integer(&args, 4, 1, 65535)
            }
            "begin" => {
                args.len() == 6
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && integer(&args, 3, 0, 3)
                    && integer(&args, 4, 0, i32::MAX)
                    && integer(&args, 5, 1, 65536)
            }
            "chunk" => {
                args.len() == 5
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && integer(&args, 3, 0, 31)
                    && matches!(args.get(4),Some(OscType::Blob(v)) if !v.is_empty() && v.len()<=CHUNK)
            }
            "commit" | "abort" | "close" => {
                args.len() == 3 && scoped && integer(&args, 2, 1, i32::MAX)
            }
            "function-set" => {
                args.len() == 4
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && matches!(&args[3],OscType::Blob(v) if !v.is_empty() && v.len()<=3072)
            }
            "function-remove" => {
                args.len() == 4
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && matches!(&args[3],OscType::String(v) if custom::identifier(v,24))
            }
            "capture-scan" => {
                args.len() == 4
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && matches!(&args[3],OscType::String(p) if !p.is_empty() && p.len()<=4096 && !p.contains('\0'))
            }
            "capture-devices" => args.len() == 3 && scoped && integer(&args, 2, 1, i32::MAX),
            "capture" => {
                args.len() == 6
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && integer(&args, 3, 0, 3)
                    && matches!(&args[4],OscType::String(k) if k=="camera" || k=="screen")
                    && integer(&args, 5, 0, 255)
            }
            "video" => {
                args.len() == 8
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && integer(&args, 3, 0, 3)
                    && matches!(&args[4],OscType::String(p) if !p.is_empty() && p.len()<=4096 && !p.contains('\0'))
                    && matches!(&args[5],OscType::String(p) if !p.is_empty() && p.len()<=4096 && !p.contains('\0'))
                    && integer(&args, 6, 0, 1)
                    && matches!(args[7],OscType::Float(v) if v.is_finite() && (0.1..=4.).contains(&v))
            }
            "video-control" => {
                args.len() == 6
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && integer(&args, 3, 0, 3)
                    && matches!(&args[4],OscType::String(c) if ["play","pause","loop","seek","rate"].contains(&c.as_str()))
                    && matches!(args[5],OscType::Float(v) if v.is_finite() && (0.0..=86400.).contains(&v))
            }
            "video-status" => {
                args.len() == 4
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && integer(&args, 3, 0, 3)
            }
            "image" => {
                args.len() == 5
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && integer(&args, 3, 0, 3)
                    && matches!(&args[4],OscType::String(p) if p.len()<=4096 && !p.contains('\0'))
            }
            "clock" => {
                args.len() == 7
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && matches!(args[3],OscType::Float(v) if v.is_finite() && (0.0..=1000.).contains(&v))
                    && matches!(args[4],OscType::Float(v) if v.is_finite() && v.abs()<=16.)
                    && matches!(args[5],OscType::Float(v) if v.is_finite() && v.abs()<=1e9)
                    && integer(&args, 6, 0, 1)
            }
            "parameter" => {
                args.len() == 7
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && integer(&args, 3, 0, 3)
                    && integer(&args, 4, 1, i32::MAX)
                    && integer(&args, 5, 1, 256)
                    && matches!(args[6],OscType::Float(v) if v.is_finite() && v.abs()<=1e6)
            }
            "present" => {
                args.len() == 4
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && integer(&args, 3, -1, 4)
            }
            "status" => {
                args.len() == 4
                    && scoped
                    && integer(&args, 2, 1, i32::MAX)
                    && integer(&args, 3, 0, i32::MAX)
            }
            _ => false,
        };
        if !valid {
            return Err(format!("invalid or unsupported Hydra {operation} request"));
        }
        Ok(Self {
            operation: operation.to_owned(),
            args,
            sender,
        })
    }
    pub fn heap_bytes(&self) -> usize {
        self.operation.capacity()
            + self.args.capacity() * std::mem::size_of::<OscType>()
            + self
                .args
                .iter()
                .map(|a| match a {
                    OscType::String(s) => s.capacity(),
                    OscType::Blob(b) => b.capacity(),
                    _ => 0,
                })
                .sum::<usize>()
    }
    fn int(&self, i: usize) -> i32 {
        if let OscType::Int(v) = self.args[i] {
            v
        } else {
            unreachable!()
        }
    }
    fn text(&self, i: usize) -> &str {
        if let OscType::String(v) = &self.args[i] {
            v
        } else {
            unreachable!()
        }
    }
    pub fn reply_target(&self) -> ReplyTarget {
        let mut address = self.sender;
        if self.operation == "capabilities" {
            address.set_port(self.int(2) as u16);
        }
        if self.operation == "open" {
            address.set_port(self.int(4) as u16);
        }
        ReplyTarget::from_sender(address, true)
    }
}
struct Upload {
    tx: i32,
    output: i32,
    expected: i32,
    size: usize,
    chunks: Vec<Option<Vec<u8>>>,
    started: f64,
    touched: f64,
}
struct Session {
    functions: Vec<custom::Definition>,
    function_sequence: Option<(i32, String, Vec<OscType>, String)>,
    id: i32,
    owner: SocketAddr,
    target: ReplyTarget,
    nonce: String,
    high_water: i32,
    revision: [i32; 4],
    committed_outputs: VecDeque<(i32, i32)>,
    selected: i32,
    present_sequence: i32,
    source_sequences: [Option<(i32, String)>; 4],
    scan_sequence: Option<(i32, String)>,
    clock_sequence: Option<(i32, f32, f32, f32, i32)>,
    parameter_sequences: [[Option<(i32, f32)>; 256]; 4],
    upload: Option<Upload>,
    results: VecDeque<(i32, &'static str, String)>,
}
pub enum Preparation<'a> {
    Function(&'a custom::Definition),
    CaptureScan(&'a str),
    CaptureDevices(&'a mut capture::Devices),
    Capture(usize, &'a str, i32),
    Commit(usize, &'a ir::Program),
    Present(i32),
    Parameter(usize, u32, f32),
    Clock(f32, f32, Option<f32>),
    Image(usize, &'a str),
    Video(usize, &'a str, &'a str, bool, f32),
    VideoControl(usize, &'a str, f32),
    VideoStatus(usize, &'a mut video::Snapshot),
}
pub struct Transport {
    epoch: String,
    next_session: i32,
    session: Option<Session>,
}
pub struct Reply {
    pub target: ReplyTarget,
    pub suffix: &'static str,
    pub args: Vec<OscType>,
}
impl Default for Transport {
    fn default() -> Self {
        Self {
            epoch: format!(
                "{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos()
            ),
            next_session: 1,
            session: None,
        }
    }
}
fn failure(target: ReplyTarget, code: &'static str, message: impl Into<String>) -> ErrorReply {
    ErrorReply {
        reply_to: target,
        severity: "error",
        subsystem: "hydra",
        resource_id: 0,
        code,
        message: message.into(),
    }
}
fn remember(session: &mut Session, tx: i32, code: &'static str, message: &str) {
    if session.results.len() == 32 {
        session.results.pop_front();
    }
    // Leave room for the OSC envelope in the <=4 KiB status response.
    let mut end = message.len().min(3072);
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    session
        .results
        .push_back((tx, code, message[..end].to_owned()));
}
impl Transport {
    pub fn reply_target(&self) -> Option<ReplyTarget> {
        self.session.as_ref().map(|s| s.target)
    }
    pub fn abort_pending(&mut self, reason: &str) {
        if let Some(session) = &mut self.session
            && let Some(upload) = session.upload.take()
        {
            remember(session, upload.tx, "E_HYDRA_UPLOAD", reason);
        }
    }
    pub fn expire(&mut self, now: f64) {
        if let Some(session) = &mut self.session
            && session
                .upload
                .as_ref()
                .is_some_and(|u| now - u.touched >= 10. || now - u.started >= 30.)
        {
            let tx = session.upload.take().unwrap().tx;
            remember(session, tx, "E_HYDRA_UPLOAD", "upload expired");
        }
    }
    #[cfg(test)]
    pub fn execute(&mut self, r: Request, now: f64) -> Result<Reply, ErrorReply> {
        self.execute_with(r, now, |preparation| match preparation {
            Preparation::Function(definition) => {
                compiler::compile(&definition.fixture()).map(|_| ())
            }
            Preparation::Commit(_, program) => compiler::compile(program).map(|_| ()),
            Preparation::Present(_)
            | Preparation::Clock(..)
            | Preparation::Parameter(..)
            | Preparation::Image(..)
            | Preparation::CaptureScan(..)
            | Preparation::CaptureDevices(..)
            | Preparation::Capture(..)
            | Preparation::Video(..)
            | Preparation::VideoControl(..)
            | Preparation::VideoStatus(..) => Ok(()),
        })
    }
    pub fn execute_with(
        &mut self,
        r: Request,
        now: f64,
        mut prepare: impl FnMut(Preparation<'_>) -> Result<(), String>,
    ) -> Result<Reply, ErrorReply> {
        self.expire(now);
        let target = r.reply_target();
        if r.operation == "capabilities" {
            if r.int(1) != 1 {
                return Err(failure(
                    target,
                    "E_HYDRA_VERSION",
                    "Hydra ABI major must be 1",
                ));
            }
            let limits = [65536u32, 2048, 256, 64, 4096];
            return Ok(Reply {
                target,
                suffix: "capabilities.reply",
                args: vec![
                    r.int(0).into(),
                    self.epoch.clone().into(),
                    1.into(),
                    0.into(),
                    OscType::Blob(limits.into_iter().flat_map(u32::to_be_bytes).collect()),
                    format!(
                        "staging-ir-v1;wgsl-v1:{};output-o0-v1;outputs-temporal-v1;numeric-v1;clock-v1;image-sources-v1;video-ffmpeg-v1;custom-wgsl-v1{}",
                        registry::supported()
                            .map(|s| s.name)
                            .collect::<Vec<_>>()
                            .join(","),
                        if cfg!(target_os="macos") {";capture-avfoundation-v1"} else {""}
                    )
                    .into(),
                ],
            });
        }
        if r.operation == "open" {
            if r.text(1) != self.epoch || r.int(3) != 1 {
                return Err(failure(
                    target,
                    "E_HYDRA_VERSION",
                    "stale epoch or unsupported ABI",
                ));
            }
            if let Some(s) = &self.session {
                if s.owner != r.sender || s.nonce != r.text(2) || s.target.address != target.address
                {
                    return Err(failure(target, "E_HYDRA_SESSION", "session already owned"));
                }
            } else {
                let id = self.next_session;
                self.next_session = self
                    .next_session
                    .checked_add(1)
                    .ok_or_else(|| failure(target, "E_HYDRA_LIMIT", "session IDs exhausted"))?;
                self.session = Some(Session {
                    functions: vec![],
                    function_sequence: None,
                    id,
                    owner: r.sender,
                    target,
                    nonce: r.text(2).into(),
                    high_water: 0,
                    revision: [0; 4],
                    committed_outputs: VecDeque::new(),
                    selected: -1,
                    present_sequence: 0,
                    parameter_sequences: [[None; 256]; 4],
                    clock_sequence: None,
                    scan_sequence: None,
                    source_sequences: std::array::from_fn(|_| None),
                    upload: None,
                    results: VecDeque::new(),
                });
            }
            return Ok(Reply {
                target,
                suffix: "opened",
                args: vec![
                    r.int(0).into(),
                    self.epoch.clone().into(),
                    self.session.as_ref().unwrap().id.into(),
                ],
            });
        }
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| failure(target, "E_HYDRA_SESSION", "no Hydra session"))?;
        if r.text(0) != self.epoch || r.int(1) != session.id || r.sender != session.owner {
            return Err(failure(
                target,
                "E_HYDRA_SESSION",
                "stale or foreign session",
            ));
        }
        let target = session.target;
        let tx = r.int(2);
        let mut reply = Reply {
            target,
            suffix: "ack",
            args: vec![self.epoch.clone().into(), session.id.into(), tx.into()],
        };
        let source_signature =
            if ["image", "video", "video-control", "capture"].contains(&r.operation.as_str()) {
                let slot = r.int(3) as usize;
                let sig = format!("{}:{:?}", r.operation, &r.args[4..]);
                if let Some((last, old)) = &session.source_sequences[slot] {
                    if tx == *last && *old == sig {
                        return Ok(reply);
                    }
                    if tx <= *last {
                        return Err(failure(target, "E_HYDRA_STALE", "stale source sequence"));
                    }
                }
                Some((slot, sig))
            } else {
                None
            };
        match r.operation.as_str() {
            "function-set" | "function-remove" => {
                if let Some((last, operation, args, wire)) = &session.function_sequence {
                    if tx == *last && operation == &r.operation && *args == r.args[3..] {
                        if r.operation == "function-set" {
                            reply.suffix = "function-set";
                            reply.args.push(wire.clone().into());
                        }
                        return Ok(reply);
                    }
                    if tx <= *last {
                        return Err(failure(
                            target,
                            "E_HYDRA_STALE",
                            "stale function registry sequence",
                        ));
                    }
                }
                let wire = if r.operation == "function-set" {
                    let OscType::Blob(bytes) = &r.args[3] else {
                        unreachable!()
                    };
                    let definition = custom::Definition::decode(bytes, tx)
                        .map_err(|e| failure(target, "E_HYDRA_FUNCTION", e))?;
                    let existing = session
                        .functions
                        .iter()
                        .position(|d| d.name == definition.name);
                    if existing.is_none() && session.functions.len() >= custom::MAX_FUNCTIONS {
                        return Err(failure(
                            target,
                            "E_HYDRA_LIMIT",
                            "custom function registry limit is 16",
                        ));
                    }
                    prepare(Preparation::Function(&definition))
                        .map_err(|e| failure(target, "E_HYDRA_FUNCTION", e))?;
                    let wire = definition.wire_name();
                    if let Some(index) = existing {
                        session.functions[index] = definition;
                    } else {
                        session.functions.push(definition);
                    }
                    reply.suffix = "function-set";
                    reply.args.push(wire.clone().into());
                    wire
                } else {
                    session.functions.retain(|d| d.name != r.text(3));
                    String::new()
                };
                session.function_sequence =
                    Some((tx, r.operation.clone(), r.args[3..].to_vec(), wire));
            }
            "begin" => {
                if let Some(u) = &session.upload {
                    if u.tx == tx
                        && u.output == r.int(3)
                        && u.expected == r.int(4)
                        && u.size == r.int(5) as usize
                    {
                        reply.args.push((-1).into());
                        return Ok(reply);
                    }
                    return Err(failure(
                        target,
                        "E_HYDRA_UPLOAD",
                        "another upload is active",
                    ));
                }
                if tx <= session.high_water {
                    return Err(failure(
                        target,
                        "E_HYDRA_STALE",
                        "transaction ID is stale; query status",
                    ));
                }
                if r.int(4) != session.revision[r.int(3) as usize] {
                    return Err(failure(target, "E_HYDRA_STALE", "output revision mismatch"));
                }
                session.high_water = tx;
                let size = r.int(5) as usize;
                session.upload = Some(Upload {
                    tx,
                    output: r.int(3),
                    expected: r.int(4),
                    size,
                    chunks: vec![None; size.div_ceil(CHUNK)],
                    started: now,
                    touched: now,
                });
                reply.args.push((-1).into());
            }
            "chunk" => {
                let u = session
                    .upload
                    .as_mut()
                    .filter(|u| u.tx == tx)
                    .ok_or_else(|| failure(target, "E_HYDRA_UPLOAD", "no matching upload"))?;
                let index = r.int(3) as usize;
                let OscType::Blob(data) = &r.args[4] else {
                    unreachable!()
                };
                if index >= u.chunks.len() || data.len() != (u.size - index * CHUNK).min(CHUNK) {
                    return Err(failure(
                        target,
                        "E_HYDRA_UPLOAD",
                        "wrong chunk index or length",
                    ));
                }
                if u.chunks[index].as_ref().is_some_and(|old| old != data) {
                    return Err(failure(
                        target,
                        "E_HYDRA_UPLOAD",
                        "conflicting duplicate chunk",
                    ));
                }
                u.chunks[index] = Some(data.clone());
                u.touched = now;
                reply.args.push((index as i32).into());
            }
            "commit" => {
                if let Some((_, code, message)) = session.results.iter().find(|entry| entry.0 == tx)
                {
                    if code.is_empty() {
                        reply.suffix = "committed";
                        reply.args.extend([
                            session
                                .committed_outputs
                                .iter()
                                .find(|entry| entry.0 == tx)
                                .unwrap()
                                .1
                                .into(),
                            message.parse::<i32>().unwrap().into(),
                        ]);
                        return Ok(reply);
                    }
                    return Err(failure(target, code, message.clone()));
                }
                let u = session
                    .upload
                    .as_ref()
                    .filter(|u| u.tx == tx)
                    .ok_or_else(|| failure(target, "E_HYDRA_UPLOAD", "no matching upload"))?;
                if u.chunks.iter().any(Option::is_none) {
                    return Err(failure(target, "E_HYDRA_UPLOAD", "upload incomplete"));
                }
                if u.expected != session.revision[u.output as usize] {
                    return Err(failure(
                        target,
                        "E_HYDRA_STALE",
                        "output changed before commit",
                    ));
                }
                let output = u.output;
                let next_revision = session.revision[output as usize]
                    .checked_add(1)
                    .ok_or_else(|| failure(target, "E_HYDRA_LIMIT", "revision exhausted"))?;
                let bytes: Vec<u8> = u
                    .chunks
                    .iter()
                    .flat_map(|chunk| chunk.as_ref().unwrap().iter().copied())
                    .collect();
                session.upload = None;
                let (code, message) =
                    match ir::Program::decode_with_custom(&bytes, &session.functions) {
                        Ok(program) => {
                            match prepare(Preparation::Commit(output as usize, &program)) {
                                Ok(()) => {
                                    session.revision[output as usize] = next_revision;
                                    session.parameter_sequences[output as usize] = [None; 256];
                                    if session.committed_outputs.len() == 32 {
                                        session.committed_outputs.pop_front();
                                    }
                                    session.committed_outputs.push_back((tx, output));
                                    remember(session, tx, "", &next_revision.to_string());
                                    reply.suffix = "committed";
                                    reply.args.extend([output.into(), next_revision.into()]);
                                    return Ok(reply);
                                }
                                Err(message) => ("E_HYDRA_COMPILE", message),
                            }
                        }
                        Err(message) => ("E_HYDRA_IR", message),
                    };
                remember(session, tx, code, &message);
                return Err(failure(target, code, message));
            }
            "abort" => {
                if session.upload.as_ref().is_some_and(|u| u.tx == tx) {
                    session.upload = None;
                    remember(session, tx, "E_HYDRA_UPLOAD", "aborted");
                }
                reply.suffix = "aborted";
            }
            "capture-scan" => {
                if let Some((last, decoder)) = &session.scan_sequence {
                    if tx == *last && decoder == r.text(3) {
                        return Ok(reply);
                    }
                    if tx <= *last {
                        return Err(failure(
                            target,
                            "E_HYDRA_STALE",
                            "stale capture discovery sequence",
                        ));
                    }
                }
                prepare(Preparation::CaptureScan(r.text(3)))
                    .map_err(|e| failure(target, "E_HYDRA_CAPTURE", e))?;
                session.scan_sequence = Some((tx, r.text(3).into()));
            }
            "capture-devices" => {
                let mut devices = capture::Devices::default();
                prepare(Preparation::CaptureDevices(&mut devices))
                    .map_err(|e| failure(target, "E_HYDRA_CAPTURE", e))?;
                reply.suffix = "capture-devices.reply";
                reply.args.extend([
                    devices.state.into(),
                    devices.error.into(),
                    (devices.devices.len() as i32).into(),
                ]);
                for d in devices.devices {
                    let mut name = d.name;
                    name.truncate(name.floor_char_boundary(96));
                    reply
                        .args
                        .extend([d.kind.into(), d.index.into(), name.into()]);
                }
            }
            "capture" => {
                prepare(Preparation::Capture(r.int(3) as usize, r.text(4), r.int(5)))
                    .map_err(|e| failure(target, "E_HYDRA_CAPTURE", e))?;
            }
            "video" => {
                let OscType::Float(rate) = r.args[7] else {
                    unreachable!()
                };
                prepare(Preparation::Video(
                    r.int(3) as usize,
                    r.text(4),
                    r.text(5),
                    r.int(6) == 1,
                    rate,
                ))
                .map_err(|e| failure(target, "E_HYDRA_VIDEO", e))?;
            }
            "video-control" => {
                let OscType::Float(value) = r.args[5] else {
                    unreachable!()
                };
                prepare(Preparation::VideoControl(
                    r.int(3) as usize,
                    r.text(4),
                    value,
                ))
                .map_err(|e| failure(target, "E_HYDRA_VIDEO", e))?;
            }
            "video-status" => {
                let mut status = video::Snapshot::default();
                prepare(Preparation::VideoStatus(r.int(3) as usize, &mut status))
                    .map_err(|e| failure(target, "E_HYDRA_VIDEO", e))?;
                reply.suffix = "video-status.reply";
                reply.args.extend([
                    r.int(3).into(),
                    status.state.into(),
                    OscType::Float(status.position),
                    status.decoded.into(),
                    status.dropped.into(),
                    status.error.into(),
                    status.kind.into(),
                    status.input_width.into(),
                    status.input_height.into(),
                    (video::WIDTH as i32).into(),
                    (video::HEIGHT as i32).into(),
                    OscType::Float(30.),
                ]);
            }
            "image" => {
                let slot = r.int(3) as usize;
                let OscType::String(path) = &r.args[4] else {
                    unreachable!()
                };
                prepare(Preparation::Image(slot, path))
                    .map_err(|m| failure(target, "E_HYDRA_IMAGE", m))?;
            }
            "clock" => {
                let (OscType::Float(bpm), OscType::Float(speed), OscType::Float(phase)) =
                    (&r.args[3], &r.args[4], &r.args[5])
                else {
                    unreachable!()
                };
                let entry = (tx, *bpm, *speed, *phase, r.int(6));
                if let Some(last) = session.clock_sequence {
                    if entry == last {
                        return Ok(reply);
                    }
                    if tx <= last.0 {
                        return Err(failure(target, "E_HYDRA_STALE", "stale clock sequence"));
                    }
                }
                prepare(Preparation::Clock(
                    *bpm,
                    *speed,
                    if r.int(6) == 1 { Some(*phase) } else { None },
                ))
                .map_err(|m| failure(target, "E_HYDRA_CAPABILITY", m))?;
                session.clock_sequence = Some(entry);
            }
            "parameter" => {
                let slot = r.int(3) as usize;
                let id = r.int(5);
                let OscType::Float(value) = r.args[6] else {
                    unreachable!()
                };
                if session.revision[slot] != r.int(4) {
                    return Err(failure(target, "E_HYDRA_STALE", "stale parameter revision"));
                }
                let entry = (tx, value);
                if let Some(last) = session.parameter_sequences[slot][(id - 1) as usize] {
                    if entry == last {
                        return Ok(reply);
                    }
                    if tx <= last.0 {
                        return Err(failure(target, "E_HYDRA_STALE", "stale parameter sequence"));
                    }
                }
                prepare(Preparation::Parameter(slot, id as u32, value))
                    .map_err(|message| failure(target, "E_HYDRA_CAPABILITY", message))?;
                session.parameter_sequences[slot][(id - 1) as usize] = Some(entry);
            }
            "present" => {
                if tx == session.present_sequence && r.int(3) == session.selected {
                    reply.suffix = "presented";
                    reply.args.push(session.selected.into());
                    return Ok(reply);
                }
                if tx <= session.present_sequence {
                    return Err(failure(
                        target,
                        "E_HYDRA_STALE",
                        "stale presentation sequence",
                    ));
                }
                prepare(Preparation::Present(r.int(3)))
                    .map_err(|message| failure(target, "E_HYDRA_LIMIT", message))?;
                session.present_sequence = tx;
                session.selected = r.int(3);
                reply.suffix = "presented";
                reply.args.push(session.selected.into());
            }
            "status" => {
                let queried = r.int(3);
                let result = session.results.iter().find(|entry| entry.0 == queried);
                let state = if session.upload.as_ref().is_some_and(|u| u.tx == queried) {
                    "uploading"
                } else if result.is_some_and(|v| v.1.is_empty()) {
                    "committed"
                } else if result.is_some() {
                    "failed"
                } else {
                    "unknown"
                };
                reply.suffix = "status.reply";
                reply.args.extend([
                    session.selected.into(),
                    OscType::Blob(
                        session
                            .revision
                            .into_iter()
                            .flat_map(i32::to_be_bytes)
                            .collect(),
                    ),
                    state.into(),
                    result.map_or("", |v| v.1).into(),
                    result.map_or("", |v| v.2.as_str()).into(),
                ]);
            }
            "close" => {
                self.session = None;
                reply.suffix = "closed";
                return Ok(reply);
            }
            _ => unreachable!(),
        }
        if let Some((slot, signature)) = source_signature {
            session.source_sequences[slot] = Some((tx, signature));
        }
        Ok(reply)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sender() -> SocketAddr {
        "127.0.0.1:57120".parse().unwrap()
    }
    fn run(t: &mut Transport, op: &str, args: Vec<OscType>, now: f64) -> Result<Reply, ErrorReply> {
        t.execute(Request::parse(op, args, sender()).unwrap(), now)
    }
    fn open(t: &mut Transport) {
        run(
            t,
            "open",
            vec![
                1.into(),
                t.epoch.clone().into(),
                "test".into(),
                1.into(),
                57120.into(),
            ],
            0.,
        )
        .unwrap();
    }
    fn scoped(t: &Transport, tail: Vec<OscType>) -> Vec<OscType> {
        let mut args = vec![t.epoch.clone().into(), 1.into()];
        args.extend(tail);
        args
    }
    #[test]
    fn clock_retry_stale_and_failed_prepare() {
        let mut t = Transport::default();
        open(&mut t);
        let args = scoped(
            &t,
            vec![5.into(), 120f32.into(), 1f32.into(), 0f32.into(), 1.into()],
        );
        let request =
            Request::parse("clock", args.clone(), "127.0.0.1:57120".parse().unwrap()).unwrap();
        assert!(
            t.execute_with(request, 0., |_| Err("not open".into()))
                .is_err()
        );
        assert!(t.session.as_ref().unwrap().clock_sequence.is_none());
        run(&mut t, "clock", args.clone(), 0.).unwrap();
        run(&mut t, "clock", args, 0.).unwrap();
        let args = scoped(
            &t,
            vec![4.into(), 90f32.into(), 1f32.into(), 0f32.into(), 0.into()],
        );
        assert!(run(&mut t, "clock", args, 0.).is_err());
    }
    #[test]
    fn image_retry_failure_and_independent_slots() {
        let mut t = Transport::default();
        open(&mut t);
        let args = scoped(&t, vec![10.into(), 0.into(), "fixture.ppm".into()]);
        let request =
            Request::parse("image", args.clone(), "127.0.0.1:57120".parse().unwrap()).unwrap();
        assert!(
            t.execute_with(request, 0., |_| Err("decode failed".into()))
                .is_err()
        );
        assert!(t.session.as_ref().unwrap().source_sequences[0].is_none());
        run(&mut t, "image", args.clone(), 0.).unwrap();
        run(&mut t, "image", args, 0.).unwrap();
        let args = scoped(&t, vec![9.into(), 1.into(), "other.ppm".into()]);
        run(&mut t, "image", args, 0.).unwrap();
        let args = scoped(&t, vec![8.into(), 0.into(), "".into()]);
        assert!(run(&mut t, "image", args, 0.).is_err());
    }
    #[test]
    fn video_image_and_controls_share_slot_ordering() {
        let mut t = Transport::default();
        open(&mut t);
        let args = scoped(
            &t,
            vec![
                10.into(),
                0.into(),
                "clip.mkv".into(),
                "/decoder".into(),
                1.into(),
                OscType::Float(1.),
            ],
        );
        let request = Request::parse("video", args.clone(), sender()).unwrap();
        assert!(
            t.execute_with(request, 0., |_| Err("decoder unavailable".into()))
                .is_err()
        );
        assert!(t.session.as_ref().unwrap().source_sequences[0].is_none());
        run(&mut t, "video", args.clone(), 0.).unwrap();
        let request = Request::parse("video", args, sender()).unwrap();
        t.execute_with(request, 0., |_| panic!("duplicate restarted decoder"))
            .unwrap();
        let args = scoped(
            &t,
            vec![11.into(), 0.into(), "pause".into(), OscType::Float(0.)],
        );
        run(&mut t, "video-control", args, 0.).unwrap();
        let args = scoped(&t, vec![10.into(), 0.into(), "old.ppm".into()]);
        assert!(run(&mut t, "image", args, 0.).is_err());
        let args = scoped(&t, vec![10.into(), 1.into(), "independent.ppm".into()]);
        run(&mut t, "image", args, 0.).unwrap();
        let args = scoped(&t, vec![12.into(), 0.into()]);
        let request = Request::parse("video-status", args, sender()).unwrap();
        let reply = t
            .execute_with(request, 0., |p| {
                if let Preparation::VideoStatus(_, status) = p {
                    status.state = "playing".into();
                    status.decoded = 7;
                }
                Ok(())
            })
            .unwrap();
        assert_eq!(reply.suffix, "video-status.reply");
        assert_eq!(reply.args[4], OscType::String("playing".into()));
        assert_eq!(reply.args[6], OscType::Int(7));
    }
    #[test]
    fn capture_discovery_retry_and_source_ordering() {
        let mut t = Transport::default();
        open(&mut t);
        let args = scoped(&t, vec![10.into(), "/decoder".into()]);
        let request = Request::parse("capture-scan", args.clone(), sender()).unwrap();
        assert!(
            t.execute_with(request, 0., |_| Err("not available".into()))
                .is_err()
        );
        assert!(t.session.as_ref().unwrap().scan_sequence.is_none());
        run(&mut t, "capture-scan", args.clone(), 0.).unwrap();
        let request = Request::parse("capture-scan", args, sender()).unwrap();
        t.execute_with(request, 0., |_| panic!("retry rescanned devices"))
            .unwrap();
        let args = scoped(&t, vec![11.into(), 0.into(), "camera".into(), 0.into()]);
        run(&mut t, "capture", args, 0.).unwrap();
        let args = scoped(&t, vec![10.into(), 0.into(), "old.ppm".into()]);
        assert!(run(&mut t, "image", args, 0.).is_err());
        let args = scoped(&t, vec![12.into()]);
        let request = Request::parse("capture-devices", args, sender()).unwrap();
        let reply = t
            .execute_with(request, 0., |p| {
                if let Preparation::CaptureDevices(result) = p {
                    result.state = "ready".into();
                    result.devices = (0..16)
                        .map(|index| capture::Device {
                            kind: "camera",
                            index,
                            name: "é".repeat(256),
                        })
                        .collect();
                }
                Ok(())
            })
            .unwrap();
        let bytes = rosc::encoder::encode(&rosc::OscPacket::Message(rosc::OscMessage {
            addr: format!("/scshader/v1/hydra/{}", reply.suffix),
            args: reply.args,
        }))
        .unwrap();
        assert!(
            bytes.len() < 4096,
            "device labels exceeded the bounded reply"
        );
    }
    #[test]
    fn custom_registry_retry_rollback_and_generation_lifetime() {
        let mut t = Transport::default();
        open(&mut t);
        let d = custom::tests::definition(0, "return vec4<f32>(gain);");
        let args = scoped(
            &t,
            vec![10.into(), OscType::Blob(custom::tests::payload(&d))],
        );
        let reply = run(&mut t, "function-set", args.clone(), 0.).unwrap();
        assert_eq!(reply.args[3], OscType::String("u_testFn_10".into()));
        let request = Request::parse("function-set", args, sender()).unwrap();
        t.execute_with(request, 0., |_| panic!("retry compiled twice"))
            .unwrap();
        let snapshot = t.session.as_ref().unwrap().functions[0].fixture();
        let args = scoped(
            &t,
            vec![11.into(), OscType::Blob(custom::tests::payload(&d))],
        );
        let request = Request::parse("function-set", args, sender()).unwrap();
        assert!(
            t.execute_with(request, 0., |_| Err("GPU validation failed".into()))
                .is_err()
        );
        assert_eq!(t.session.as_ref().unwrap().functions[0].generation, 10);
        let args = scoped(&t, vec![9.into(), "testFn".into()]);
        assert!(run(&mut t, "function-remove", args, 0.).is_err());
        let args = scoped(&t, vec![12.into(), "testFn".into()]);
        run(&mut t, "function-remove", args, 0.).unwrap();
        assert!(t.session.as_ref().unwrap().functions.is_empty());
        compiler::compile(&snapshot).unwrap(); // accepted programs outlive registry removal
    }
    #[test]
    fn concurrent_parameters_keep_independent_sequence_order() {
        let mut t = Transport::default();
        open(&mut t);
        t.session.as_mut().unwrap().revision[0] = 1;
        for (sequence, id, value) in [(10, 1, 0.2), (9, 2, 0.7), (11, 1, 0.3)] {
            let args = scoped(
                &t,
                vec![
                    sequence.into(),
                    0.into(),
                    1.into(),
                    id.into(),
                    OscType::Float(value),
                ],
            );
            run(&mut t, "parameter", args, 0.).unwrap();
        }
        let args = scoped(
            &t,
            vec![8.into(), 0.into(), 1.into(), 2.into(), OscType::Float(0.4)],
        );
        assert!(run(&mut t, "parameter", args, 0.).is_err());
    }
    #[test]
    fn numeric_updates_guard_revision_sequence_and_failure() {
        let mut t = Transport::default();
        open(&mut t);
        t.session.as_mut().unwrap().revision[0] = 1;
        let args = scoped(
            &t,
            vec![10.into(), 0.into(), 1.into(), 1.into(), OscType::Float(0.5)],
        );
        let request = Request::parse(
            "parameter",
            args.clone(),
            "127.0.0.1:57120".parse().unwrap(),
        )
        .unwrap();
        assert!(
            t.execute_with(request, 0., |_| Err("unknown parameter".into()))
                .is_err()
        );
        assert_eq!(t.session.as_ref().unwrap().parameter_sequences[0][0], None);
        run(&mut t, "parameter", args.clone(), 0.).unwrap();
        run(&mut t, "parameter", args, 0.).unwrap();
        let args = scoped(
            &t,
            vec![9.into(), 0.into(), 1.into(), 1.into(), OscType::Float(0.7)],
        );
        assert!(run(&mut t, "parameter", args, 0.).is_err());
        let args = scoped(
            &t,
            vec![11.into(), 0.into(), 2.into(), 1.into(), OscType::Float(0.7)],
        );
        assert!(run(&mut t, "parameter", args, 0.).is_err());
    }
    #[test]
    fn upload_bounds_duplicates_and_expiry() {
        let mut t = Transport::default();
        open(&mut t);
        let begin = scoped(&t, vec![1.into(), 0.into(), 0.into(), 2050.into()]);
        run(&mut t, "begin", begin.clone(), 0.).unwrap();
        run(&mut t, "begin", begin, 1.).unwrap();
        let chunk = scoped(&t, vec![1.into(), 1.into(), OscType::Blob(vec![2; 2])]);
        run(&mut t, "chunk", chunk.clone(), 1.).unwrap();
        run(&mut t, "chunk", chunk, 1.).unwrap();
        let conflict = scoped(&t, vec![1.into(), 1.into(), OscType::Blob(vec![3; 2])]);
        assert!(run(&mut t, "chunk", conflict, 2.).is_err());
        let commit = scoped(&t, vec![1.into()]);
        assert!(run(&mut t, "commit", commit, 2.).is_err());
        t.expire(12.);
        assert!(t.session.as_ref().unwrap().upload.is_none());
        let begin = scoped(&t, vec![1.into(), 0.into(), 0.into(), 1.into()]);
        assert!(run(&mut t, "begin", begin, 13.).is_err());
    }
    #[test]
    fn commit_failure_is_retained_without_activation() {
        let mut t = Transport::default();
        open(&mut t);
        let begin = scoped(&t, vec![1.into(), 0.into(), 0.into(), 1.into()]);
        run(&mut t, "begin", begin, 0.).unwrap();
        let chunk = scoped(&t, vec![1.into(), 0.into(), OscType::Blob(vec![0])]);
        run(&mut t, "chunk", chunk, 0.).unwrap();
        let commit = scoped(&t, vec![1.into()]);
        assert_eq!(
            run(&mut t, "commit", commit.clone(), 0.)
                .err()
                .unwrap()
                .code,
            "E_HYDRA_IR"
        );
        assert_eq!(
            run(&mut t, "commit", commit, 0.).err().unwrap().code,
            "E_HYDRA_IR"
        );
        assert!(t.session.as_ref().unwrap().upload.is_none());
        assert_eq!(t.session.as_ref().unwrap().results.len(), 1);
    }
    #[test]
    fn retained_compiler_diagnostics_fit_status_without_splitting_utf8() {
        let mut transport = Transport::default();
        open(&mut transport);
        let session = transport.session.as_mut().unwrap();
        let message = format!("x{}", "🔥".repeat(2048));
        remember(session, 1, "E_HYDRA_COMPILE", &message);
        let args = scoped(&transport, vec![2.into(), 1.into()]);
        let reply = run(&mut transport, "status", args, 0.).unwrap();
        let OscType::String(reason) = &reply.args[7] else {
            panic!("expected status reason")
        };
        assert!(reason.len() <= 3072);
        assert!(message.starts_with(reason));
    }
    #[test]
    fn committed_revisions_retry_and_present_ordering() {
        let mut t = Transport::default();
        open(&mut t);
        let mut bytes = b"HYIR\0\x01\0\0".to_vec();
        for n in [1u32, 0, 1, 1] {
            bytes.extend(n.to_be_bytes());
        }
        bytes.extend([0, 3, 0, 3]);
        bytes.extend(b"osc");
        bytes.extend(0u32.to_be_bytes());
        for value in [10f32, 0.1, 0.] {
            bytes.push(0);
            bytes.extend(value.to_be_bytes());
        }
        let args = scoped(
            &t,
            vec![1.into(), 0.into(), 0.into(), (bytes.len() as i32).into()],
        );
        run(&mut t, "begin", args, 0.).unwrap();
        let args = scoped(&t, vec![1.into(), 0.into(), OscType::Blob(bytes)]);
        run(&mut t, "chunk", args, 0.).unwrap();
        let args = scoped(&t, vec![1.into()]);
        assert_eq!(
            run(&mut t, "commit", args.clone(), 0.).unwrap().suffix,
            "committed"
        );
        assert_eq!(run(&mut t, "commit", args, 0.).unwrap().suffix, "committed");
        assert_eq!(t.session.as_ref().unwrap().revision[0], 1);
        let args = scoped(&t, vec![2.into(), 0.into(), 0.into(), 1.into()]);
        assert_eq!(
            run(&mut t, "begin", args, 0.).err().unwrap().code,
            "E_HYDRA_STALE"
        );
        let args = scoped(&t, vec![2.into(), 0.into(), 1.into(), 1.into()]);
        run(&mut t, "begin", args, 0.).unwrap();
        let args = scoped(&t, vec![2.into(), 0.into(), OscType::Blob(vec![0])]);
        run(&mut t, "chunk", args, 0.).unwrap();
        let args = scoped(&t, vec![2.into()]);
        assert!(run(&mut t, "commit", args, 0.).is_err());
        assert_eq!(t.session.as_ref().unwrap().revision[0], 1);
        let args = scoped(&t, vec![10.into(), 0.into()]);
        run(&mut t, "present", args.clone(), 0.).unwrap();
        run(&mut t, "present", args, 0.).unwrap();
        let args = scoped(&t, vec![9.into(), (-1).into()]);
        assert!(run(&mut t, "present", args, 0.).is_err());
        assert_eq!(t.session.as_ref().unwrap().selected, 0);
    }
    #[test]
    fn four_revisions_and_duplicate_commit_destination() {
        let mut t = Transport::default();
        open(&mut t);
        let mut bytes = b"HYIR\0\x01\0\0".to_vec();
        for word in [1u32, 0, 1, 1] {
            bytes.extend(word.to_be_bytes());
        }
        bytes.extend([0, 4, 0, 5]);
        bytes.extend(b"solid");
        bytes.extend(0u32.to_be_bytes());
        for value in [0f32, 0., 0., 1.] {
            bytes.push(0);
            bytes.extend(value.to_be_bytes());
        }
        for slot in 0..4 {
            let tx = slot + 1;
            let args = scoped(
                &t,
                vec![
                    tx.into(),
                    slot.into(),
                    0.into(),
                    (bytes.len() as i32).into(),
                ],
            );
            run(&mut t, "begin", args, 0.).unwrap();
            let args = scoped(&t, vec![tx.into(), 0.into(), OscType::Blob(bytes.clone())]);
            run(&mut t, "chunk", args, 0.).unwrap();
            let args = scoped(&t, vec![tx.into()]);
            let reply = run(&mut t, "commit", args.clone(), 0.).unwrap();
            assert_eq!(reply.args[3], OscType::Int(slot));
            assert_eq!(run(&mut t, "commit", args, 0.).unwrap().args, reply.args);
        }
        assert_eq!(t.session.as_ref().unwrap().revision, [1; 4]);
        let args = scoped(&t, vec![20.into(), 4.into()]);
        run(&mut t, "present", args, 0.).unwrap();
        let args = scoped(&t, vec![21.into(), 0.into()]);
        let reply = run(&mut t, "status", args, 0.).unwrap();
        assert_eq!(
            reply.args[4],
            OscType::Blob([1i32; 4].into_iter().flat_map(i32::to_be_bytes).collect())
        );
    }
    #[test]
    fn rejected_display_preparation_keeps_selection_and_sequence() {
        let mut t = Transport::default();
        open(&mut t);
        let args = scoped(&t, vec![5.into(), 4.into()]);
        let request =
            Request::parse("present", args.clone(), "127.0.0.1:57120".parse().unwrap()).unwrap();
        assert_eq!(
            t.execute_with(request, 0., |_| Err("dimension limit".into()))
                .err()
                .unwrap()
                .code,
            "E_HYDRA_LIMIT"
        );
        assert_eq!(t.session.as_ref().unwrap().selected, -1);
        assert_eq!(t.session.as_ref().unwrap().present_sequence, 0);
        run(&mut t, "present", args, 0.).unwrap();
        assert_eq!(t.session.as_ref().unwrap().selected, 4);
    }
    #[test]
    fn ownership_and_parser_limits() {
        let mut t = Transport::default();
        open(&mut t);
        let args = scoped(&t, vec![1.into(), 0.into()]);
        let r = Request::parse("status", args, "127.0.0.1:57121".parse().unwrap()).unwrap();
        assert_eq!(t.execute(r, 0.).err().unwrap().code, "E_HYDRA_SESSION");
        let args = scoped(&t, vec![1.into(), 0.into(), 0.into(), 65537.into()]);
        assert!(Request::parse("begin", args, sender()).is_err());
        let args = scoped(&t, vec![1.into(), 0.into(), OscType::Blob(vec![0; 2049])]);
        assert!(Request::parse("chunk", args, sender()).is_err());
    }
}
