<#
.SYNOPSIS
  Build the Aegis Windows installers (MSI + NSIS) and a portable ZIP.

.DESCRIPTION
  1. Builds the AegisService binary (release) and stages it as the Tauri sidecar.
  2. Builds the frontend + Tauri app and bundles MSI + NSIS (per tauri.conf.json).
  3. Produces a portable ZIP from the built executables.

  Requires: Rust (MSVC), Node.js, and the Tauri prerequisites (WebView2; the
  WiX/NSIS toolchains are fetched by the Tauri bundler on first run).

.PARAMETER SkipBundle
  Build the app but skip the MSI/NSIS bundling step (faster smoke build).
#>
[CmdletBinding()]
param([switch]$SkipBundle)

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
$Triple = 'x86_64-pc-windows-msvc'
Set-Location $Root

Write-Host '==> Building AegisService (release)…'
cargo build -p aegis-service --release

$svc = Join-Path $Root "target\release\aegis-service.exe"
if (-not (Test-Path $svc)) { throw "aegis-service.exe not produced at $svc" }

$binDir = Join-Path $Root 'src-tauri\binaries'
New-Item -ItemType Directory -Force -Path $binDir | Out-Null
$sidecar = Join-Path $binDir "aegis-service-$Triple.exe"
Copy-Item $svc $sidecar -Force
Write-Host "==> Staged sidecar: $sidecar"

Write-Host '==> Installing frontend deps + building UI…'
npm ci
npm run build

if ($SkipBundle) {
  Write-Host '==> Building app (no bundle)…'
  npm run tauri -- build --no-bundle
  Write-Host 'Done (no-bundle smoke build).'
  return
}

Write-Host '==> Building app + bundling MSI + NSIS…'
npm run tauri -- build

$bundleDir = Join-Path $Root 'src-tauri\target\release\bundle'
Write-Host "==> Installers under: $bundleDir"
Get-ChildItem -Recurse -Path $bundleDir -Include *.msi, *-setup.exe -ErrorAction SilentlyContinue |
  Select-Object FullName, Length

# Portable ZIP from the release exe + sidecar.
$portableDir = Join-Path $Root 'target\portable\Aegis-Antivirus'
New-Item -ItemType Directory -Force -Path $portableDir | Out-Null
Copy-Item (Join-Path $Root 'src-tauri\target\release\aegis-tauri.exe') (Join-Path $portableDir 'Aegis Antivirus.exe') -Force -ErrorAction SilentlyContinue
Copy-Item $svc (Join-Path $portableDir 'aegis-service.exe') -Force
$zip = Join-Path $Root 'target\portable\Aegis-Antivirus-portable.zip'
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path "$portableDir\*" -DestinationPath $zip
Write-Host "==> Portable ZIP: $zip"
