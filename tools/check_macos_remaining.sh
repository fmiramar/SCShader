#!/usr/bin/env bash
# Remaining short Apple Silicon checks; invoke from an interactive local session.
set -euo pipefail

project_dir=$(cd "$(dirname "$0")/.." && pwd)
cd "$project_dir"
renderer=${1:-"$HOME/Library/Application Support/SuperCollider/Extensions/SCShader/renderer/scshader-renderer"}
recovery="$project_dir/build/recovery-target/aarch64-apple-darwin/debug/scshader-renderer"
mkdir -p build/platform-tests
evidence=$(mktemp -d "$project_dir/build/platform-tests/macos-arm64-remaining-XXXXXX")
printf 'Evidence: %s\n' "$evidence"

# Fail before attempting GPU checks if this execution context cannot use OSC.
python3 - "$renderer" "$recovery" "$evidence" <<'PY'
import json
from pathlib import Path
import platform
import socket
import sys

sys.path.insert(0, str(Path.cwd() / "tools"))
from package_support import check_binary

renderer, recovery, evidence = map(Path, sys.argv[1:])
record = dict(passed=False, platform=platform.system(), architecture=platform.machine(), stage="native_binaries")
try:
    if (platform.system(), platform.machine()) != ("Darwin", "arm64"):
        raise RuntimeError("these checks require a native Apple Silicon desktop")
    for name, path in [("renderer", renderer), ("recovery_renderer", recovery)]:
        record[name] = dict(path=str(path.resolve()), **check_binary(path, "aarch64-apple-darwin"))
    record["stage"] = "loopback_udp"
    with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
        sock.bind(("127.0.0.1", 0))
    record["passed"] = True
except (OSError, RuntimeError, ValueError) as error:
    record["reason"] = str(error)
finally:
    (evidence / "preflight.json").write_text(json.dumps(record, indent=2) + "\n")
if not record["passed"]:
    sys.exit("Preflight blocked at " + record["stage"] + ": " + record["reason"])
print("PASS native binaries and local UDP preflight")
PY

# These are serial, bounded checks. Every pipeline propagates test failure.
python3 tools/stress_schedule.py --renderer "$renderer" \
    2>&1 | tee "$evidence/schedule-console.log"
python3 tools/stress_runtime.py --renderer "$renderer" --cycles 80 --seconds 2 \
    --output-dir "$evidence/stress" 2>&1 | tee "$evidence/stress-console.log"
python3 tools/run_acceptance.py --renderer "$renderer" --backend metal \
    --seconds 10 --modes feedback --output-dir "$evidence/feedback-resize" \
    2>&1 | tee "$evidence/feedback-console.log"
python3 tools/check_gpu_recovery.py --renderer "$recovery" \
    --log "$evidence/recovery-renderer.log" 2>&1 | tee "$evidence/recovery-console.log"

# Confirm the same executables were used throughout; do not combine candidates.
python3 - "$evidence" <<'PY'
import hashlib
import json
from pathlib import Path
import sys

evidence = Path(sys.argv[1])
record = json.loads((evidence / "preflight.json").read_text())
for name in ("renderer", "recovery_renderer"):
    binary = record[name]
    with Path(binary["path"]).open("rb") as source:
        actual = hashlib.file_digest(source, "sha256").hexdigest()
    if actual != binary["renderer_sha256"]:
        sys.exit("FAIL executable changed during checks: " + name)
(evidence / "complete.json").write_text(json.dumps(dict(
    passed=True, renderer_sha256=record["renderer"]["renderer_sha256"],
    recovery_renderer_sha256=record["recovery_renderer"]["renderer_sha256"],
    checks=["schedule", "resource_stress", "feedback_resize", "logical_recovery"],
    long_duration_validation=False), indent=2) + "\n")
print("PASS remaining short checks; results saved to", evidence)
PY
