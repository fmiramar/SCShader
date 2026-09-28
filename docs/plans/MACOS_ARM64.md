# macOS Apple Silicon agentic plan

Status: native arm64 package path exists; an Apple Silicon build/runtime result
has **not** been established. Intel Metal passes do not qualify Apple GPUs.
Follow [COMMON.md](COMMON.md).

## A1. Prove native execution, not Rosetta

Use an Apple Silicon Mac in an interactive desktop with Xcode command-line tools,
rustup, Python 3.11+, and native arm64/universal SuperCollider running as arm64.
Inspect `uname -m`, `arch`, `rustc -vV`, `xcode-select -p`, and
`sysctl -in sysctl.proc_translated` when available. A result of `1` indicates a
translated process: reopen a native terminal/toolchain. An absent translation key
alone is not sufficient evidence; also verify CPU/Rust host and built Mach-O.

Required host/target: `aarch64-apple-darwin`. Record macOS version, Apple GPU family,
SC executable/process architecture, display scale/refresh, Python and Rust versions.
Do not copy the Intel executable and rename it arm64 or use Rosetta as a native pass.

## A2. Native build and package

Run COMMON static checks, then:

```sh
target=aarch64-apple-darwin
export SCSHADER_ARTIFACT_PREFIX=fmiramar
python3 tools/package_support.py --target "$target" --check-host
bash tools/package_macos.sh arm64
lipo -archs renderer/target/aarch64-apple-darwin/release/scshader-renderer
otool -L renderer/target/aarch64-apple-darwin/release/scshader-renderer
```

Require an arm64 Mach-O and no accidental developer-local dylib dependencies.
Inspect deployment target load commands, package `build-info.json`, checksum, and
renderer `--version`. Record the actual tested minimum macOS, not an inferred
deployment compatibility promise. Keep the inherited Metal split-submission
workaround until experiments justify a change without regressing Intel.

Install the extracted package in this user's normal Extensions folder, backing
up the previous extension outside the scanned tree. Set `extension`, `renderer`,
and `sclang` to absolute installed paths. Run all COMMON SC and help checks,
including source examples from an unsaved document and renderer auto-discovery.

## A3. Apple GPU behavior

```sh
python3 tools/run_acceptance.py --renderer "$renderer" --backend metal --seconds 10 --modes traffic trivial reload feedback --output-dir build/platform-tests/macos-arm64-metal-01
```

Run COMMON scheduling/resource stress and the isolated `gpu-test-hooks` build.
Recovery binary: `build/recovery-target/aarch64-apple-darwin/debug/scshader-renderer`.
Record logical-device recovery separately from natural physical/driver failures.

Investigate in order:

1. Native AppKit event-loop/lifecycle, discovery/launch/reboot, .app sclang paths,
   Unicode/spaces, missing executable behavior, and no leftover owned processes.
2. WGSL and constrained GLSL/Shadertoy compilation, all reflected matrix/vector
   layouts, Float32/FFT texture streaming and GPU pixel/readback behavior.
3. Feedback/graph pass order, stats overlay exclusion, shader reload and resource
   churn, and memory behavior on unified-memory hardware. Short RSS stability
   does not prove a one-hour or eight-hour result.
4. Retina logical/framebuffer mapping, internal/external display moves, fullscreen,
   minimize/restore, sleep/wake, refresh-rate changes, cursor/input/focus.
5. Download quarantine/Gatekeeper behavior on a transferred development package.
   Record unsigned/not-notarized status honestly. Do not strip quarantine,
   disable protections, or claim distribution signing was completed automatically.

## A4. Exit gate

Native arm64 installed package, Metal critical suites, SC/help examples, and
documented Apple display/lifecycle checks pass with exact hashes. Record unavailable
external displays or additional chip generations as not run. Preserve separate
Intel support and request regression for shared fixes. Universal bundling, signing,
notice clearance, final duration testing, and publication are later decisions.
