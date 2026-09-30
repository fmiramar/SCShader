# Native platform result — TEMPLATE, not an executed result

Copy to `docs/platform-results/YYYY-MM-DD-target.md`. Replace every TBD; use
**pass**, **fail**, **blocked**, or **not run**, and give a reason for the latter two.
Do not include private paths, hostnames, usernames, tokens, or unreviewed raw logs.

## Identity

- Date / tester-approved attribution: TBD
- Source ZIP checksum, manifest identity, or approved commit + dirty state: TBD
- SCShader VERSION / Cargo.lock SHA-256: TBD
- OS version / distribution / CPU and process architecture / native vs emulated: TBD
- Rust version and host target / Python / linker/SDK: TBD
- SuperCollider version, language/server architecture, audio environment: TBD
- GPU / driver / actual backend / hardware or software adapter: TBD
- Display session / compositor / Xorg vs Xwayland / scale / refresh: TBD
- Package filename / SHA-256 / installed renderer SHA-256: TBD
- Binary format / deployment or GLIBC minimum / runtime imports: TBD

## Checks

| Gate | Result | Command, exact binary, relative evidence path, observations |
| --- | --- | --- |
| Formatting, all-target/all-feature Clippy and Rust tests | not run | TBD |
| Python unit tests | not run | TBD |
| Native linked package, header/hash/dependency inspection | not run | TBD |
| Clean extension install, discovery, SC class compile | not run | TBD |
| All installed SC checks including SCDoc (currently 26) | not run | TBD |
| 18 help/README demos + cleanups (36 blocks) | not run | TBD |
| Explicit default backend, four 10-second modes | not run | TBD |
| Additional backend / display session (separate rows per path) | not run | TBD |
| Future scheduling saturation / recovery | not run | TBD |
| 80-cycle resource churn / two-second floods | not run | TBD |
| Feature-gated logical-device recovery / pixel checks | not run | TBD |
| Process crash/timeout/quit/reboot ownership cleanup | not run | TBD |
| Visible typed controls, images, graph, FFT/waveform, reload | not run | TBD |
| Fullscreen, resize, minimize, focus/input | not run | TBD |
| HiDPI / extra monitor / refresh / sleep-wake | not run | TBD |
| Missing executable/driver and install-path edge cases | not run | TBD |
| Notice gaps / unsigned package launch behavior | not run | TBD |

Record duration, offered load, observed drops/saturation, frame progress, fresh
replies, suppressed diagnostics, memory baseline/peak/growth, and remaining
processes for live tests. Link failures as well as corrected reruns. The SC runner
overwrites fixed log filenames: preserve each run before rerunning it.

## Fixes and limitations

- Reproduction, root cause, modified files, regression test: TBD
- Shared changes requiring other-target regression: TBD
- Critical failures or missing primary-platform coverage: TBD
- Optional hardware/backends not tested: TBD
- Third-party notice/signing/runtime redistribution blockers: TBD
- Physical/driver reset evidence: not run (logical injection is not equivalent).
- One-hour exact-candidate result: not run unless separately scheduled.
- Eight-hour final-release sign-off: reserved for the user; not run here.

## Handoff decision

Track complete / incomplete: TBD, with reasons and next concrete action.
Precisely tested support scope: TBD. Do not extrapolate to all drivers/OS versions.
Milestone and verification docs updated: TBD. Reviewed development sync to public
`main` is authorized; release tags, uploads, hosted workflow dispatches, and final
release publication still require separate authorization.
