#!/usr/bin/env bash
set -euo pipefail

project_dir=$(cd "$(dirname "$0")/.." && pwd)
cd "$project_dir"
version=$(tr -d '[:space:]' < "$project_dir/VERSION")
requested_arch=${1:?usage: tools/package_macos.sh <x64|arm64>}
machine_arch=$(uname -m)

case "$requested_arch:$machine_arch" in
  x64:x86_64) notice_target=x86_64-apple-darwin ;;
  arm64:arm64) notice_target=aarch64-apple-darwin ;;
  *)
    printf 'Requested macOS %s package, but this machine is %s.\n' "$requested_arch" "$machine_arch" >&2
    exit 2
    ;;
esac

python3 "$project_dir/tools/package_support.py" --target "$notice_target" --check-host

prefix=${SCSHADER_ARTIFACT_PREFIX:-}
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([.-][A-Za-z0-9.-]+)?$ ]] || { echo "Invalid package version" >&2; exit 2; }
[[ -z "$prefix" || "$prefix" =~ ^[A-Za-z0-9][A-Za-z0-9_-]*$ ]] || { echo "Invalid artifact prefix" >&2; exit 2; }
base=${prefix:+$prefix-}SCShader-${version}-macos-${requested_arch}
stage_root="$project_dir/stage/$base"
extension_root="$stage_root/SCShader"
dist_dir="$project_dir/dist"

for existing in "$stage_root" "$dist_dir/$base.zip" "$dist_dir/$base.zip.sha256" \
  "$dist_dir/$base-corresponding-source.zip" "$dist_dir/$base-corresponding-source.zip.sha256"; do
  [[ ! -e "$existing" ]] || { printf 'Output already exists: %s\n' "$existing" >&2; exit 2; }
done
mkdir -p "$extension_root/renderer" "$dist_dir"

cargo build --manifest-path "$project_dir/renderer/Cargo.toml" --release --locked --target "$notice_target" --target-dir "$project_dir/renderer/target"
renderer_binary="$project_dir/renderer/target/$notice_target/release/scshader-renderer"
python3 "$project_dir/tools/package_support.py" --target "$notice_target" --binary "$renderer_binary" --output-metadata "$extension_root/build-info.json"
actual_arch=$(lipo -archs "$renderer_binary")
[[ "$actual_arch" == "$machine_arch" ]] || { echo "Built executable architecture does not match requested package" >&2; exit 2; }
cp -R "$project_dir/Classes" "$extension_root/"
cp -R "$project_dir/HelpSource" "$extension_root/"
cp -R "$project_dir/renderer/shaders" "$extension_root/shaders"
cp -R "$project_dir/docs" "$project_dir/protocol" "$project_dir/examples" "$extension_root/"
cp "$project_dir/README.md" "$project_dir/START_HERE.md" "$project_dir/AGENTS.md" "$project_dir/LICENSE" "$project_dir/COPYING" "$project_dir/CHANGELOG.md" "$project_dir/VERSION" "$project_dir/SCShader.quark" "$extension_root/"
cp "$renderer_binary" "$extension_root/renderer/"
notice_flag=
case "${SCSHADER_REQUIRE_NOTICE_TEXTS:-0}" in
  0) ;;
  1) notice_flag=--require-texts ;;
  *) echo "SCSHADER_REQUIRE_NOTICE_TEXTS must be 0 or 1" >&2; exit 2 ;;
esac
python3 "$project_dir/tools/license_inventory.py" --target "$notice_target" \
  --output-dir "$extension_root/dependency-audit" --distribution ${notice_flag:+"$notice_flag"}
cargo fetch --manifest-path "$project_dir/renderer/Cargo.toml" --locked
python3 "$project_dir/tools/package_corresponding_source.py" --target "$notice_target" \
  --binary "$renderer_binary" --extension "$extension_root" \
  --output "$dist_dir/$base-corresponding-source.zip"

# Finder/AppleDouble files may exist in a working source directory. Remove only
# staged metadata; preserve the user's original files and extended attributes.
python3 - "$stage_root" <<'PY'
import os
from pathlib import Path
import shutil
import sys

for directory, dirs, files in os.walk(sys.argv[1], topdown=False):
    for name in files:
        if name == ".DS_Store" or name.startswith("._"):
            (Path(directory) / name).unlink()
    for name in dirs:
        if name == "__MACOSX":
            shutil.rmtree(Path(directory) / name)
PY

archive="$dist_dir/$base.zip"
checksum="$archive.sha256"
ditto -c -k --norsrc --noextattr --noacl --keepParent "$stage_root" "$archive"
(cd "$dist_dir" && shasum -a 256 "$base.zip") > "$checksum"
printf 'Created %s\n' "$archive"
