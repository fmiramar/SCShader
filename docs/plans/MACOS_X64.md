# macOS Intel regression plan

Baseline: 0.0.16 local Metal short suites and installed examples passed. The
historical one-hour pass is for 0.0.14, not this executable. This track protects
existing functionality while new native platforms are brought up; follow
[COMMON.md](COMMON.md), but do not rerun unchanged long suites simply for a handoff.

## I1. Select a real Intel host

Verify `uname -m` reports x86_64 and `rustc -vV` identifies
`x86_64-apple-darwin`. Record macOS, Intel Mac GPU/driver, SC and display information.
Rosetta x64 on Apple Silicon is not an Intel GPU regression substitute. Keep an
untouched copy/hash of the last known-good installation before installing a fix.

## I2. Rebuild after shared changes

Run COMMON static checks, then:

```sh
target=x86_64-apple-darwin
export SCSHADER_ARTIFACT_PREFIX=fmiramar
bash tools/package_macos.sh x64
lipo -archs renderer/target/x86_64-apple-darwin/release/scshader-renderer
otool -L renderer/target/x86_64-apple-darwin/release/scshader-renderer
```

Install the extracted extension and set `extension`, `renderer`, and `sclang`
to installed paths. Repeat all SC/help checks after shared code/docs changes.

```sh
python3 tools/run_acceptance.py --renderer "$renderer" --backend metal --seconds 10 --modes traffic trivial reload feedback --output-dir build/platform-tests/macos-x64-regression-01
```

Run short COMMON stress/recovery if scheduling, resources, recovery, diagnostics,
or native lifecycle changed. For isolated text/tooling-only changes, use focused
checks and state why no new renderer qualification is claimed. The hook-enabled
recovery executable is under
`build/recovery-target/x86_64-apple-darwin/debug/scshader-renderer`.

## I3. Critical regression focus

- Do not regress Metal per-pass submissions, GPU feedback/graph order, bounded
  queues, diagnostic suppression accounting, or overlay isolation.
- Preserve installed discovery and all executable docs, especially fresh server
  instances, packaged assets, audio buffer synchronization and cleanup.
- Check Retina/non-Retina windows, fullscreen, lifecycle/reboot, resize/reload,
  audio/FFT streams, and retained backend/display during logical-device recovery.
- Keep exact binary/hash evidence separate from 0.0.14's one-hour pass. If a
  renderer change needs a new one-hour gate, schedule it with the user later.

## I4. Exit gate

Affected checks and the critical native regression suite pass with a result record
and hash matching the installed ordinary release binary. If a regression appears,
retain negative evidence and fix it before integrating the platform change.
Signing/notices and the user's final eight-hour run remain open. No publication.
