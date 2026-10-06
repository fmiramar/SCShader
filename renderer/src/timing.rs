//! Opt-in wall-clock tracing, including an OSC-thread snapshot during a service stall.
//! No OSC fields, queue limits, or failure policies change when this is enabled.
use std::{
    sync::{
        OnceLock,
        atomic::{AtomicU64, Ordering},
    },
    time::{Instant, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum Stage {
    EventLoop,
    Initialize,
    Commands,
    Render,
    Acquire,
    Submit,
    Present,
    Poll,
    Reload,
    WindowEvent,
    AboutToWait,
}

const NAMES: [&str; 11] = [
    "event_loop",
    "initialize",
    "commands",
    "render",
    "surface_acquire",
    "queue_submit",
    "present",
    "gpu_poll",
    "hot_reload",
    "window_event",
    "about_to_wait",
];
const SLOW_US: u64 = 50_000;
static TRACE: OnceLock<Option<Trace>> = OnceLock::new();

struct Trace {
    epoch: Instant,
    // Phase and start time share one atomic snapshot. Writers are main-thread only;
    // the OSC thread can inspect the phase even while a driver call is blocked.
    phase: AtomicU64,
    // App-side queue-consumption time, sampled after a successful try_recv.
    last_dequeue: AtomicU64,
    slow_counts: [AtomicU64; 11],
    #[cfg(target_os = "windows")]
    native_message: AtomicU64,
}

impl Trace {
    fn new() -> Self {
        Self {
            epoch: Instant::now(),
            phase: AtomicU64::new(0),
            last_dequeue: AtomicU64::new(0),
            slow_counts: std::array::from_fn(|_| AtomicU64::new(0)),
            #[cfg(target_os = "windows")]
            native_message: AtomicU64::new(0),
        }
    }

    fn now(&self) -> u64 {
        self.epoch
            .elapsed()
            .as_micros()
            .min(u128::from(u64::MAX >> 8)) as u64
    }

    fn enter(&self, stage: Stage) -> Scope<'_> {
        let started = self.now();
        let previous = self
            .phase
            .swap((started << 8) | stage as u64, Ordering::Relaxed);
        Scope {
            trace: Some(self),
            previous,
            started,
            stage,
        }
    }

    fn snapshot_at(&self, now: u64) -> (usize, u64, u64) {
        let phase = self.phase.load(Ordering::Relaxed);
        (
            (phase & 0xff) as usize,
            now.saturating_sub(phase >> 8),
            now.saturating_sub(self.last_dequeue.load(Ordering::Relaxed)),
        )
    }

    fn mark_dequeued(&self) {
        self.last_dequeue.store(self.now(), Ordering::Relaxed);
    }
}

fn trace() -> Option<&'static Trace> {
    TRACE
        .get_or_init(|| {
            (std::env::var("SCSHADER_TRACE_TIMING").as_deref() == Ok("1")).then(|| {
                let unix = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs_f64();
                eprintln!("SCShader timing enabled unix_s={unix:.6}; wall-clock phases, slow threshold=50ms");
                Trace::new()
            })
        })
        .as_ref()
}

pub struct Scope<'a> {
    trace: Option<&'a Trace>,
    previous: u64,
    started: u64,
    stage: Stage,
}

pub fn enter(stage: Stage) -> Scope<'static> {
    trace().map(|trace| trace.enter(stage)).unwrap_or(Scope {
        trace: None,
        previous: 0,
        started: 0,
        stage,
    })
}

impl Drop for Scope<'_> {
    fn drop(&mut self) {
        if let Some(trace) = self.trace {
            let now = trace.now();
            // Restart the idle/event-loop clock, but retain an enclosing render's
            // original start when returning from nested acquire/submit/present.
            let previous = if self.previous & 0xff == Stage::EventLoop as u64 {
                now << 8
            } else {
                self.previous
            };
            trace.phase.store(previous, Ordering::Relaxed);
            let elapsed = now.saturating_sub(self.started);
            if elapsed >= SLOW_US {
                let count =
                    trace.slow_counts[self.stage as usize].fetch_add(1, Ordering::Relaxed) + 1;
                if count <= 4 || count.is_power_of_two() {
                    eprintln!(
                        "SCShader timing t={:.6}s slow={} elapsed_ms={:.3} count={count}",
                        now as f64 / 1_000_000.0,
                        NAMES[self.stage as usize],
                        elapsed as f64 / 1000.0
                    );
                }
            }
        }
    }
}

pub fn dequeued() {
    if let Some(trace) = trace() {
        trace.mark_dequeued();
    }
}

pub fn queue_full() {
    if let Some(trace) = trace() {
        let now = trace.now();
        let (phase, phase_us, dequeue_us) = trace.snapshot_at(now);
        eprintln!(
            "SCShader timing t={:.6}s queue_full phase={} phase_ms={:.3} since_dequeue_ms={:.3}",
            now as f64 / 1_000_000.0,
            NAMES[phase],
            phase_us as f64 / 1000.0,
            dequeue_us as f64 / 1000.0
        );
        #[cfg(target_os = "windows")]
        {
            let message = trace.native_message.load(Ordering::Relaxed);
            eprintln!(
                "SCShader timing last_native_message=0x{:04x} system_command=0x{:04x}",
                message >> 32,
                message as u32
            );
        }
    }
}

#[cfg(target_os = "windows")]
pub fn install_windows_hook(builder: &mut winit::event_loop::EventLoopBuilder<()>) {
    use winit::platform::windows::EventLoopBuilderExtWindows;
    if let Some(trace) = trace() {
        // Only the documented MSG prefix is needed; never record input/key data.
        #[repr(C)]
        struct MessagePrefix {
            hwnd: *mut std::ffi::c_void,
            message: u32,
            wparam: usize,
        }
        builder.with_msg_hook(move |message| {
            // SAFETY: winit supplies a valid Windows MSG for this callback.
            let message = unsafe { &*message.cast::<MessagePrefix>() };
            let command = if message.message == 0x0112 {
                message.wparam as u32
            } else {
                0
            };
            trace.native_message.store(
                (u64::from(message.message) << 32) | u64::from(command),
                Ordering::Relaxed,
            );
            false
        });
    }
}

pub fn event(name: &str) {
    if let Some(trace) = trace() {
        eprintln!(
            "SCShader timing t={:.6}s event={name}",
            trace.now() as f64 / 1_000_000.0
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receiver_can_observe_nested_driver_call_and_its_enclosing_phase() {
        let trace = Trace::new();
        let render = trace.enter(Stage::Render);
        let original = trace.phase.load(Ordering::Relaxed);
        {
            let _acquire = trace.enter(Stage::Acquire);
            let (phase, _, _) = trace.snapshot_at(trace.now());
            assert_eq!(phase, Stage::Acquire as usize);
        }
        assert_eq!(trace.phase.load(Ordering::Relaxed), original);
        drop(render);
        assert_eq!(trace.snapshot_at(trace.now()).0, Stage::EventLoop as usize);
    }

    #[test]
    fn stalled_snapshot_reports_driver_wait_and_time_since_last_dequeue() {
        let trace = Trace::new();
        trace
            .phase
            .store((10_000 << 8) | Stage::Acquire as u64, Ordering::Relaxed);
        trace.last_dequeue.store(8_000, Ordering::Relaxed);
        assert_eq!(
            trace.snapshot_at(300_000),
            (Stage::Acquire as usize, 290_000, 292_000)
        );
    }

    #[test]
    fn dequeue_resets_the_queue_service_age_snapshot() {
        let trace = Trace::new();
        trace.last_dequeue.store(1, Ordering::Relaxed);
        trace.mark_dequeued();
        let dequeued_at = trace.last_dequeue.load(Ordering::Relaxed);
        assert_eq!(trace.snapshot_at(dequeued_at + 5_000).2, 5_000);
    }
}
