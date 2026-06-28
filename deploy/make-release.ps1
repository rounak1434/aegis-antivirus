<#
.SYNOPSIS
  Assemble a release: collect artifacts, checksums, (optional) signature, SBOM,
  inventories, and a release manifest into ./dist.

.DESCRIPTION
  Run AFTER deploy/build-installers.ps1 (and optionally deploy/sign.ps1). Does
  not build or sign — it packages what exists. Signing the checksum file is
  optional (GPG if a key is configured, else skipped).
#>
[CmdletBinding()]
param(
  [string]$Version,
  [string]$OutDir = (Join-Path (Split-Path -Parent $PSScriptRoot) 'dist'),
  [string]$GpgKeyId = $env:AEGIS_GPG_KEY_ID
)

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

if (-not $Version) {
  $conf = Get-Content src-tauri\tauri.conf.json -Raw | ConvertFrom-Json
  $Version = $conf.version
}

Write-Host "==> Collecting artifacts for v$Version"
$bundle = Join-Path $Root 'src-tauri\target\release\bundle'
$artifacts = @()
$artifacts += Get-ChildItem -Recurse -Path $bundle -Include *.msi, *-setup.exe -ErrorAction SilentlyContinue
$portable = Join-Path $Root 'target\portable\Aegis-Antivirus-portable.zip'
if (Test-Path $portable) { $artifacts += Get-Item $portable }

foreach ($a in $artifacts) { Copy-Item $a.FullName $OutDir -Force }

# --- SHA256SUMS ---
$sumsFile = Join-Path $OutDir 'SHA256SUMS'
Remove-Item $sumsFile -ErrorAction SilentlyContinue
$entries = @()
Get-ChildItem $OutDir -File | Where-Object { $_.Name -notin @('SHA256SUMS', 'SHA256SUMS.sig', 'release-manifest.json') } | ForEach-Object {
  $h = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLower()
  Add-Content $sumsFile "$h  $($_.Name)"
  $entries += [ordered]@{ name = $_.Name; sha256 = $h; size = $_.Length }
}
Write-Host "==> SHA256SUMS written ($($entries.Count) files)"

# --- optional detached signature over SHA256SUMS ---
if ($GpgKeyId -and (Get-Command gpg -ErrorAction SilentlyContinue)) {
  gpg --batch --yes --local-user $GpgKeyId --detach-sign --armor --output "$sumsFile.sig" $sumsFile
  Write-Host '==> SHA256SUMS.sig created (GPG)'
} else {
  Write-Host '==> No GPG key configured — skipping SHA256SUMS.sig (optional)'
}

# --- SBOM + inventories ---
& (Join-Path $PSScriptRoot 'generate-sbom.ps1') -OutDir $OutDir

# --- release manifest ---
$commit = "$(git rev-parse HEAD 2>$null)".Trim()
$manifest = [ordered]@{
  product   = 'Aegis Antivirus'
  version   = $Version
  commit    = $commit
  artifacts = $entries
  sbom      = @('sbom.cargo.cdx.json', 'sbom.npm.cdx.json')
  inventories = @('license-inventory.csv', 'dependency-tree.txt')
  signed    = (Test-Path "$sumsFile.sig")
}
$manifest | ConvertTo-Json -Depth 6 | Set-Content -Encoding utf8 (Join-Path $OutDir 'release-manifest.json')

Write-Host "==> Release assembled in $OutDir"
Get-ChildItem $OutDir | Select-Object Name, Length
