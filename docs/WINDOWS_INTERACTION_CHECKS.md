# Windows interaction checks

Use the installed 0.0.17 extension. The renderer runs independently of scsynth;
the window check below does not need an audio server. Automated results and exact
binary hashes are in the [Windows record](platform-results/2026-09-28-windows-x64.md).

## Automated native window check

From the source directory, with `$Renderer` set to the installed executable:

```powershell
python tools/check_windows_interactions.py --renderer "$Renderer" --adapter NVIDIA --output-dir build/platform-tests/interactions-nvidia-01
if ($LASTEXITCODE -ne 0) { throw 'NVIDIA window check failed' }
python tools/check_windows_interactions.py --renderer "$Renderer" --adapter Intel --output-dir build/platform-tests/interactions-intel-01
if ($LASTEXITCODE -ne 0) { throw 'Intel window check failed' }
```

Choose new output directories on each run. This opens and briefly fullscreens
only its owned renderer. It verifies four resizes during watched shader reload,
fullscreen entry/exit, border toggles, actual OS minimize/restore, live OSC replies
while minimized, frame progress after restore, and exit 0 on window close.
Generated shader edits are confined to an owned fixture, not installed assets.

Window ownership uses the child's live process handle/PID and exact test title.
This avoids touching graphics-driver helper windows or other renderer sessions.
The implementation uses Microsoft's [window enumeration](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumwindows),
[owner lookup](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowthreadprocessid),
and [asynchronous show-state API](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindowasync).
Successful state/OSC checks do not prove physical input or visible correctness.

## IDE and physical-input check

Open [examples/09_window_interaction.scd](../examples/09_window_interaction.scd)
in the SuperCollider IDE. Evaluate its first parenthesized block, then interact
with the renderer window. Evaluate the final block to clean up this example.

| Action | Expected observation |
| --- | --- |
| Move mouse across the window | Animated pattern remains visible; horizontal position changes brightness |
| Click near corners | Post pane reports normalized coordinates approaching 0 or 1 |
| F, then Escape | Enters borderless fullscreen, then returns to a usable window |
| R | Recompiles unchanged source, so appearance should stay the same; the example's acknowledgement block should print reloads above zero and error nil |
| V twice | Statistics appear and disappear |
| Drag-resize; minimize/restore | Image returns at the new size and continues animating |
| Alt-Tab away and back | Window remains usable; Post pane reports focus changes |

Also run the installed `ShaderAnalysis` and `ShaderFFTTexture` help examples from
the IDE, one at a time, followed by each cleanup block. Those examples use a
looping bundled sample; an audio server and working audio device are required.
Automated audio checks already pass, but listening/viewing confirms the user's
actual IDE audio configuration and visible response.

The user confirmed that `ShaderFFTTexture` worked perfectly in the installed IDE:
audio played and the spectrum visualization behaved as expected. This is a user
observation on the tested Windows audio setup; it does not qualify other audio
devices or another operating system. `ShaderAnalysis` listening results remain
unreported.

If used in the intended setup, test a monitor move with different display scale,
and a normal sleep/wake cycle when convenient. Record unavailable/not-run cases
instead of treating a laptop-only check as multi-monitor qualification. The
automated script does not sleep the computer or change display settings.

Record version, GPU, display scale, actions, observed errors and whether cleanup
closed the owned renderer. The user reports the window/input actions worked and
confirmed `ShaderFFTTexture` worked perfectly,
but noticed no visual difference for R. That is expected for unchanged source;
its reload-count acknowledgement was requested separately. Automated reload
acknowledgements already pass. Listening/viewing the audio examples, sleep/wake
and mixed-monitor observations remain unreported.

The keyboard log's `autoRepeat, false` means an initial key press, not a failure.
Only the letter R triggers reload; numeric 4 has no assigned action. The example
now labels the repeat flag and prints an explicit reload-request message to make
these observations unambiguous.
