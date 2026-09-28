param(
    [Parameter(Mandatory = $true)]
    [string]$PackageDir,
    [string]$ExtensionsDir = (Join-Path $env:LOCALAPPDATA 'SuperCollider/Extensions')
)

$ErrorActionPreference = 'Stop'
$Source = (Resolve-Path -LiteralPath $PackageDir).Path
$Extensions = [IO.Path]::GetFullPath($ExtensionsDir)
$Destination = Join-Path $Extensions 'SCShader'
if ($Source.Equals($Destination, [StringComparison]::OrdinalIgnoreCase) -or
    $Source.StartsWith($Destination + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
    throw 'PackageDir must be an extracted package outside the installed SCShader directory.'
}
foreach ($Required in @('Classes/ShaderServer.sc', 'HelpSource/Guides/SCShader.schelp',
    'shaders/fullscreen.wgsl', 'renderer/scshader-renderer.exe', 'build-info.json', 'VERSION')) {
    if (-not (Test-Path -LiteralPath (Join-Path $Source $Required) -PathType Leaf)) {
        throw "Incomplete package: $Required is missing. Pass the inner SCShader folder from the Windows ZIP."
    }
}
$Metadata = Get-Content -LiteralPath (Join-Path $Source 'build-info.json') -Raw | ConvertFrom-Json
$Binary = Join-Path $Source 'renderer/scshader-renderer.exe'
$BinaryHash = (Get-FileHash -LiteralPath $Binary -Algorithm SHA256).Hash.ToLowerInvariant()
if ($Metadata.target -ne 'x86_64-pc-windows-msvc' -or $Metadata.binary_format -ne 'PE' -or
    $Metadata.renderer_sha256 -ne $BinaryHash -or
    $Metadata.version -ne (Get-Content -LiteralPath (Join-Path $Source 'VERSION') -Raw).Trim()) {
    throw 'Package target, version or renderer checksum does not match build-info.json.'
}
# Keep old classes and the incoming copy outside the directory SC scans.
$BackupRoot = Join-Path (Split-Path -Parent $Extensions) 'SCShader-backups'
$InstallId = (Get-Date -Format 'yyyyMMdd-HHmmss') + '-' + [Guid]::NewGuid().ToString('N').Substring(0, 8)
$Incoming = Join-Path $BackupRoot "incoming-$InstallId"
$Backup = Join-Path $BackupRoot "SCShader-$InstallId"
foreach ($Directory in @($Source, $Extensions, $Destination, $BackupRoot)) {
    if ((Test-Path -LiteralPath $Directory) -and
        ((Get-Item -LiteralPath $Directory -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "Refusing to replace or copy a linked directory: $Directory"
    }
}
if (Get-ChildItem -LiteralPath $Source -Recurse -Force |
    Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint } |
    Select-Object -First 1) {
    throw 'Package contains a linked file or directory.'
}
New-Item -ItemType Directory -Force -Path $Extensions, $BackupRoot | Out-Null
Copy-Item -LiteralPath $Source -Destination $Incoming -Recurse
$CopiedHash = (Get-FileHash -LiteralPath (Join-Path $Incoming 'renderer/scshader-renderer.exe') -Algorithm SHA256).Hash.ToLowerInvariant()
if ($CopiedHash -ne $BinaryHash) { throw "Copied renderer checksum mismatch; retained $Incoming" }
$BackedUp = $false
try {
    if (Test-Path -LiteralPath $Destination) {
        # Both exact paths are children of the resolved extension/backup roots.
        Move-Item -LiteralPath $Destination -Destination $Backup
        $BackedUp = $true
    }
    Move-Item -LiteralPath $Incoming -Destination $Destination
} catch {
    if ($BackedUp -and -not (Test-Path -LiteralPath $Destination)) {
        Move-Item -LiteralPath $Backup -Destination $Destination
    }
    throw
}
Write-Output "Installed SCShader $($Metadata.version): $Destination"
Write-Output "Renderer SHA-256: $BinaryHash"
if ($BackedUp) { Write-Output "Previous extension preserved: $Backup" }
Write-Output 'Recompile the SuperCollider class library before booting ShaderServer.'
