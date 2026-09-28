param(
    [ValidateSet("x64")]
    [string]$Architecture = "x64",
    [string]$Python = "python"
)

$ErrorActionPreference = "Stop"
$ProjectDir = Split-Path -Parent $PSScriptRoot
$Target = "x86_64-pc-windows-msvc"
& $Python -c 'import sys; sys.exit(0 if sys.version_info >= (3, 11) else 1)'
if ($LASTEXITCODE -ne 0) { throw "Python 3.11+ is required. Pass -Python with its executable path." }
# Cargo discovers rust-toolchain.toml from the working directory, not --manifest-path.
Push-Location -LiteralPath $ProjectDir
try {
& $Python (Join-Path $ProjectDir "tools/package_support.py") --target $Target --check-host
if ($LASTEXITCODE -ne 0) { throw "Native target check failed" }
$Version = (Get-Content (Join-Path $ProjectDir "VERSION") -Raw).Trim()
if ($env:PROCESSOR_ARCHITECTURE -notin @("AMD64", "x86_64")) {
    throw "Requested Windows $Architecture package, but PROCESSOR_ARCHITECTURE is $env:PROCESSOR_ARCHITECTURE."
}

$Prefix = $env:SCSHADER_ARTIFACT_PREFIX
if ($Version -notmatch '^[0-9]+\.[0-9]+\.[0-9]+([.-][A-Za-z0-9.-]+)?$') { throw "Invalid package version" }
if ($Prefix -and $Prefix -notmatch '^[A-Za-z0-9][A-Za-z0-9_-]*$') { throw "Invalid artifact prefix" }
$Base = if ($Prefix) { "$Prefix-SCShader-$Version-windows-$Architecture" } else { "SCShader-$Version-windows-$Architecture" }
$StageRoot = Join-Path (Join-Path $ProjectDir "stage") $Base
$ExtensionRoot = Join-Path $StageRoot "SCShader"
$DistDir = Join-Path $ProjectDir "dist"

$Archive = Join-Path $DistDir "$Base.zip"
$Checksum = "$Archive.sha256"
foreach ($Existing in @($StageRoot, $Archive, $Checksum)) {
    if (Test-Path -LiteralPath $Existing) {
        throw "Output already exists: $Existing. Preserve/move it before rebuilding this package."
    }
}

cargo build --manifest-path (Join-Path $ProjectDir "renderer/Cargo.toml") --release --locked --target $Target --target-dir (Join-Path $ProjectDir "renderer/target")
if ($LASTEXITCODE -ne 0) { throw "Renderer build failed" }
$RendererBinary = Join-Path $ProjectDir "renderer/target/$Target/release/scshader-renderer.exe"
New-Item -ItemType Directory -Force -Path (Join-Path $ExtensionRoot "renderer"), $DistDir | Out-Null
& $Python (Join-Path $ProjectDir "tools/package_support.py") --target $Target --binary $RendererBinary --output-metadata (Join-Path $ExtensionRoot "build-info.json")
if ($LASTEXITCODE -ne 0) { throw "Binary format check failed" }
Copy-Item -Recurse (Join-Path $ProjectDir "Classes") $ExtensionRoot
Copy-Item -Recurse (Join-Path $ProjectDir "HelpSource") $ExtensionRoot
Copy-Item -Recurse (Join-Path $ProjectDir "renderer/shaders") (Join-Path $ExtensionRoot "shaders")
foreach ($Directory in @("docs", "protocol", "examples")) {
    Copy-Item -Recurse (Join-Path $ProjectDir $Directory) $ExtensionRoot
}
Copy-Item (Join-Path $ProjectDir "README.md"), (Join-Path $ProjectDir "START_HERE.md"), (Join-Path $ProjectDir "AGENTS.md"), (Join-Path $ProjectDir "LICENSE"), (Join-Path $ProjectDir "CHANGELOG.md"), (Join-Path $ProjectDir "VERSION"), (Join-Path $ProjectDir "SCShader.quark") $ExtensionRoot
Copy-Item $RendererBinary (Join-Path $ExtensionRoot "renderer/scshader-renderer.exe")
$NoticeArgs = @()
if ($env:SCSHADER_REQUIRE_NOTICE_TEXTS -eq "1") {
    $NoticeArgs += "--require-texts"
} elseif ($env:SCSHADER_REQUIRE_NOTICE_TEXTS -and $env:SCSHADER_REQUIRE_NOTICE_TEXTS -ne "0") {
    throw "SCSHADER_REQUIRE_NOTICE_TEXTS must be 0 or 1"
}
& $Python (Join-Path $ProjectDir "tools/license_inventory.py") --target x86_64-pc-windows-msvc --output-dir (Join-Path $ExtensionRoot "dependency-audit") @NoticeArgs
if ($LASTEXITCODE -ne 0) { throw "Dependency notice audit failed" }

# AppleDouble files from a macOS source transfer are metadata, not documentation.
Get-ChildItem -LiteralPath $StageRoot -Recurse -File -Force |
    Where-Object { $_.Name.StartsWith('._') -or $_.Name -eq '.DS_Store' } |
    ForEach-Object { Remove-Item -LiteralPath $_.FullName -Force }
Compress-Archive -LiteralPath $StageRoot -DestinationPath $Archive
"$((Get-FileHash $Archive -Algorithm SHA256).Hash.ToLower())  $([IO.Path]::GetFileName($Archive))" | Set-Content -NoNewline $Checksum
Write-Output "Created $Archive"
} finally {
    Pop-Location
}
