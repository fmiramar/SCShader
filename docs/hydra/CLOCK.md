# Session clock and scheduled controls — Step 16

`h.clock(bpm=30, speed=1, phase=nil)` changes the session's visual clock. With
nil phase, time is continuous at command application. An explicit phase sets
seconds immediately. `h.reset(seconds=0, bpm=30, speed=1)` resets those controls
explicitly. Reset retains programs, numeric handles and history images; it does
not clear the scene. Speed zero pauses time while frames/feedback still render.
Negative speed reverses time. BPM accepts 0..1000, speed −16..16 and an explicit
phase must be finite within ±1e9 seconds. These are native bounded controls.

| Value | Unit / behavior |
| --- | --- |
| `h.time(rate,bias)` | Hydra seconds × rate + bias |
| `h.beat(rate,bias)` | Unwrapped beats = Hydra seconds × BPM / 60, then rate/bias |
| Beat phase | Fractional part of that unwrapped beat count; sequences wrap it as needed |
| `h.delta(rate,bias)` | Wall interval between renderer frames × current speed, then rate/bias |
| `h.frame(rate,bias)` | Low 32 bits of renderer frame counter, then rate/bias; not scaled by speed |
| Sequence phase | Hydra seconds × sequence speed × BPM / 60 + sequence offset |
| Shader clock uniform | seconds, speed, BPM, scaled frame delta |

All assigned outputs receive one common frame context. Numeric/shader values are
f32; the CPU clock uses f64 seconds and Instant anchors. Frame values converted
to f32 eventually lose individual integer precision. Delta excludes explicit
phase jumps and uses the current speed for the whole frame interval. It is not
an exact difference of clock seconds when speed changes within that interval.
Legacy selection freezes Hydra images but the session clock continues. Closing
releases the clock; duplicate open does not reset it. Resize retains the clock.
The CPU clock state, including anchor/phase/speed/BPM, is retained by existing
logical recovery; specific non-default clock recovery injection is unqualified.

Pinned Hydra [advances time by dt × speed](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/hydra-synth.js)
with initial BPM 30 and speed 1. Its [array evaluator](https://github.com/hydra-synth/hydra-synth/blob/9d29a9f4fd8f9081b9759943f38db36f05b9a88f/src/lib/array-utils.js)
uses time × speed × BPM/60. Native time advances piecewise on the renderer's
monotonic clock, including hidden periods, rather than accumulating browser ticks.
Changing BPM retains seconds but can jump beat/sequence phase, matching that
formula; it is not a beat accumulator that automatically preserves phase. Set
phase explicitly to align a new musical section. Renderer frame/delta expressions
are explicit native additions, not browser callback parity claims.

## Scheduled controls

`h.parameterMessage(name, value, output)` and `h.clockMessage(bpm, speed, phase)`
construct session-scoped numeric messages for the existing `ShaderServer.sendBundle`
or `scheduleMarker` path. `h.setAt(name, value, delay, output, action)` sends a
numeric message plus a marker using that same path. Immediate `h.set` retains its
acknowledged update behavior; `h.clock` sends immediately. The message constructors
reserve sequence numbers at construction, so construct/send messages in intended
application order for each output/control stream. Stale sequences are rejected.
Queued updates also retain output revision/session guards. Replacing a graph,
closing the session or applying a newer control can invalidate an older queued
update. A scheduled update does not rewrite the SC handle's default for future
snapshots; immediate successful set does.

Only Hydra `parameter` and `clock` accept inherited future timetags. Structural
operations remain immediate-only; an invalid structural member rejects the whole
bundle. These commands use the existing bounded queue, scheduler, NTP mapping and
payload accounting. There is no second clock estimator or per-frame SC producer.
Late commands apply when dispatched, without retroactive phase correction.

Clock wire arguments are `epoch:s, session:i, sequence:i, bpm:f, speed:f,
phaseSeconds:f, setPhase:i` (0 keeps time, 1 resets); the ordinary Hydra ack is
returned. `clock-v1` advertises this support. HYIR 1.1 numeric source kinds 4,5,6
add beat, renderer frame and delta with `[rate,bias]` payloads. Older declarations
remain compatible. Clock retries are idempotent; conflicting/older sequences and
invalid values retain accepted state.

A schedule-marker reply measures **renderer command application**, not GPU
presentation or audio output. Its callback runs even if a preceding control
reports an error: observe normal renderer diagnostics as well. Physical display
refresh, scan-out, audio device latency and OS scheduling remain separate.

## Example and verification

`examples/hydra/07_clock.scd` provides visual startup, an opt-in eight-beat audio
performance, tempo/speed change, explicit reset and cleanup. A dedicated
TempoClock/Routine consumes a Pseq at 120 BPM and sends both SC synth creation
and Hydra controls with a shared 0.1-second logical delay. Tick Synths free
themselves; cleanup stops this example's visual server/clock and keeps the user's
audio server available. Re-run the performance block after its prior run finishes.

The installed example check uses private audio/video server ports and muted
ticks. It verifies SynthDef boot/sync, eight audio bundles and eight renderer
markers without claiming listening or desktop/audio synchronization qualification.
Separate checks cover speed continuity/pause/reverse/reset, retry/failure guards,
timed-control versus structural admission, actual beat/BPM/frame/delta pixels and
TempoClock/Routine/Pseq control. See VERIFICATION.md for actual error samples,
hashes, passed checks and remaining manual observations.
