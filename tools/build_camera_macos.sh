#!/usr/bin/env bash
set -euo pipefail
project_dir=$(cd "$(dirname "$0")/.." && pwd)
output=${1:?usage: build_camera_macos.sh <new SCShaderCamera.app path>}
target=${2:-aarch64-apple-macos11.0}
[[ ! -e "$output" ]] || { echo "Camera bundle already exists: $output" >&2; exit 2; }
mkdir -p "$output/Contents/MacOS"
cp "$project_dir/renderer/mac-camera/Info.plist" "$output/Contents/Info.plist"
/usr/bin/swiftc -swift-version 5 -O -target "$target" \
    -framework AppKit -framework AVFoundation \
    "$project_dir/renderer/mac-camera/main.swift" -o "$output/Contents/MacOS/SCShaderCamera"
/usr/bin/codesign --force --sign - "$output"
"$output/Contents/MacOS/SCShaderCamera" --pixel-test
