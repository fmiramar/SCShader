# macOS window and input checks

Use the installed ordinary release renderer on a native macOS desktop. The
[Apple Silicon record](platform-results/2026-09-30-macos-arm64.md) preserves exact
hashes, successful runs and negative evidence. Intel hardware needs its own run.

## Automated native window check

From the source directory:

```sh
renderer="$HOME/Library/Application Support/SuperCollider/Extensions/SCShader/renderer/scshader-renderer"
python3 tools/check_macos_interactions.py --renderer "$renderer" --adapter Apple \
  --output-dir build/platform-tests/macos-arm64-interactions-new
```

Use a new output directory each time. The short runner requires Python 3.11+
and existing macOS Accessibility access for its launching application. It checks
permission without requesting it or changing system settings. Missing access
produces a blocked result before starting the renderer. An Intel Mac must select
its actual adapter with `--adapter`; Rosetta does not qualify native Intel.

The checker owns one renderer child, requires explicit Metal and the requested
adapter, and uses the child's PID and exact verification-window title. Before
each native mutation it rechecks the live child and retained window identity.
It never searches other applications' windows or sends global keystrokes.
Generated reload fixtures stay in the ignored evidence directory. A context
manager cleans up only the owned child even when a check fails.

The automated checks cover:

- Four actual resizes during watched shader reload, matching native AppKit frame
  dimensions and renderer content dimensions. Framebuffer dimensions must match
  the reported Retina scale; owned GPU texture allocation must follow pixel area.
- Fullscreen entry/exit confirmed through both native Accessibility state and
  renderer metrics, followed by frame progress and GPU allocation checks.
- Decoration changes confirmed by native frame/content insets, plus cursor and
  statistics flags. Visible cursor and overlay appearance still need observation.
- Synthetic F and Escape key down/up delivery in normal and fullscreen windows,
  using events addressed only to the owned child PID.
- Resize input events and focus loss during native minimize. OSC must answer
  while minimized; restoration followed by the explicit window-front command
  must report focus gain and advance frames. Restoration alone need not activate
  the application.
- Native close-button action followed by exit 0, and unchanged executable hash.

AppKit can temporarily omit a window from `AXWindows` while moving it into a
fullscreen Space. The checker waits for the same retained window to reappear;
an identity change fails. On the tested built-in display, fullscreen content is
1470x923 logical / 2940x1846 pixels, below the 33-point top display area. The
initial decorated window has a 32-point title-bar inset. These are observations
of this display configuration, not hard-coded requirements for other Macs.

The implementation uses Apple's [Accessibility application/window hierarchy](https://developer.apple.com/library/archive/documentation/Accessibility/Conceptual/AccessibilityMacOSX/OSXAXmodel.html)
and [minimized attribute](https://developer.apple.com/documentation/applicationservices/kaxminimizedattribute)
plus [PID-targeted keyboard events](https://developer.apple.com/documentation/coregraphics/cgevent/posttopid(_:))
through the system ApplicationServices and CoreFoundation libraries. No Rust
dependency or renderer protocol change is needed.

## Physical interaction and audio observations

Open [examples/09_window_interaction.scd](../examples/09_window_interaction.scd)
in SuperCollider. Run its first parenthesized block, then check this table.
Finish with its cleanup block, including after manually closing the window.

| Action | Expected observation |
| --- | --- |
| Move and click the mouse | Brightness changes horizontally; clicks print normalized coordinates |
| F once, wait for fullscreen, then Escape | Returns to a normal window; Escape already in a normal window has no visible effect |
| R, then the acknowledgement block | Reload count above zero, error nil; unchanged source looks the same |
| V twice | Statistics appear and disappear |
| Drag-resize, minimize/restore | Image resizes and resumes animating |
| Cmd-Tab away and back | Focus events print and mouse/keyboard remain usable |
| Cleanup block | Only this example's renderer/resources close |

Next open the installed `ShaderAnalysis` help page in the IDE. Run its example,
observe the sample audio and brightness response for about ten seconds, and run
its cleanup. Repeat with `ShaderFFTTexture`, observing the spectrum bars and
sample audio. Use the existing looping sample and fade-in; keep the audio server
running if it serves other work. These observations supplement the already-passed
automated analysis/FFT and help-example checks.

## Display lifecycle

When no automated suite is running, leave the interaction example active and
perform a normal Apple-menu Sleep and wake. Check animation, mouse/keyboard,
resize/fullscreen and cleanup after waking, recording any diagnostics. The agent
does not force system sleep or a hardware GPU reset.

External-monitor moves, mixed display scales and refresh-rate changes need that
hardware. The user confirmed this session has only the built-in display; these
checks are **not run**, not passed. Intel regression is blocked here by the
Apple Silicon host. One-hour validation requires separate scheduling, and the
eight-hour final-release test belongs to the user.
