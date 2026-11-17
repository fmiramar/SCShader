# Pattern controls — Step 18 / Checkpoint C

The existing numeric-message scheduler is sufficient for Pattern integration.
There is no global hydra Event type and no replacement of the existing shader
Event behavior. A control Pbind uses rest events and a Pfunc to send numeric
values while its stream advances:

```supercollider
Pbind(\type, \rest, \dur, 1, \value, Pseq([5, 8, 12, 16], 2),
    \send, Pfunc { |event|
        h.setAt(\frequency, event[\value], 0.3, \o0);
        0
    }
).play(clock);
```

Commit the named parameter before starting the Pattern. Pfunc runs while the
event is built; the rest event creates no Synth. The example's second control
Pbind drives color, while a normal audio Pbind runs on the same TempoClock. It
captures a shared starting beat and uses a future logical deadline with the
same late-wake guard as the clock example. This is scheduled command timing,
not sample-accurate display timing. No graph uploads or shader compilations are
performed by the per-note functions.

Sequence ordering now applies independently to each parameter within an output
revision. This allows two control streams to arrive in either order when they
target different parameters. Updates to the same parameter must retain intended
application order; the newest accepted sequence wins. The fixed four-by-256
sequence table is cleared for a successfully replaced output; old revisions and
old sessions remain invalid. Wire fields and ABI are unchanged. This also
prevents unrelated audio-bus controls from invalidating one another's updates.

`08_patterns.scd` creates the graph first, then starts a finite eight-note
performance with two control Pbinds and one audio Pbind. It rejects overlap and
waits for all sixteen marker replies before declaring completion. Immediate
validation rejects unknown named parameters. Marker replies indicate dispatch;
renderer diagnostics still report rejected controls. Cleanup stops all players
before disposing the controller and visual server. Audio Synths free themselves;
the user's audio server stays available. Stopping a player alone prevents new
messages but cannot recall messages already queued with lookahead; disposing the
session makes those messages stale. The shared scheduler is not globally cleared.

Native checks cover independent parameter sequences, stale same-parameter
updates, the actual installed concurrent Pbind example, invalid target handling,
unchanged shader Event identity, and the existing legacy Pattern/audio/clock
checks. Test audio is muted. Manual listening/visual observation and full browser
parity remain separate. Checkpoint C is available within this native scope;
image/media sources remain later steps. See VERIFICATION.md for exact results.
