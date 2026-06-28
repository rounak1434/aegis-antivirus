<#
.SYNOPSIS
  Authenticode-sign Aegis artifacts (exe / msi / nsis) with a timestamp.

.DESCRIPTION
  Signing is OPTIONAL. If no certificate is provided, the script logs a notice
  and exits 0 — unsigned development builds are fully supported. Provide a cert
  via -PfxPath/-PfxPassword, or a store thumbprint via -Thumbprint, or the
  AEGIS_SIGN_PFX_BASE64 / AEGIS_SIGN_PFX_PASSWORD env vars (used by CI secrets).

.PARAMETER Paths
  Files to sign (exe, msi, setup.exe). Defaults to the built release artifacts.

.EXAMPLE
  ./sign.ps1 -PfxPath cert.pfx -PfxPassword (Read-Host -AsSecureString)
  ./sign.ps1 -Thumbprint A1B2C3...   # cert already in the user/machine store
#>
[CmdletBinding()]
param(
  [string[]]$Paths,
  [string]$PfxPath,
  [string]$PfxPassword,
  [string]$Thumbprint,
  [string]$TimestampUrl = 'http://timestamp.digicert.com',
  [string]$Digest = 'sha256'
)

$ErrorActionPreference = 'Stop'
$Root = Split-Path -Parent $PSScriptRoot

function Find-SignTool {
  $cmd = Get-Command signtool.exe -ErrorAction SilentlyContinue
  if ($cmd) { return $cmd.Source }
  $found = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match 'x64' } | Select-Object -First 1
  if ($found) { return $found.FullName }
  throw 'signtool.exe not found (install the Windows SDK).'
}

# Resolve cert source: explicit args > env (CI secrets) > none.
if (-not $PfxPath -and -not $Thumbprint -and $env:AEGIS_SIGN_PFX_BASE64) {
  $tmp = Join-Path $env:TEMP 'aegis-sign.pfx'
  [IO.File]::WriteAllBytes($tmp, [Convert]::FromBase64String($env:AEGIS_SIGN_PFX_BASE64))
  $PfxPath = $tmp
  if (-not $PfxPassword) { $PfxPassword = $env:AEGIS_SIGN_PFX_PASSWORD }
}

if (-not $PfxPath -and -not $Thumbprint) {
  Write-Host 'No signing certificate supplied — skipping signing (unsigned dev build).'
  exit 0
}

if (-not $Paths) {
  $bundle = Join-Path $Root 'src-tauri\target\release\bundle'
  $Paths = @()
  $Paths += (Join-Path $Root 'src-tauri\target\release\aegis-tauri.exe')
  $Paths += (Join-Path $Root 'target\release\aegis-service.exe')
  $Paths += (Get-ChildItem -Recurse -Path $bundle -Include *.msi, *-setup.exe -ErrorAction SilentlyContinue | ForEach-Object FullName)
}

$signtool = Find-SignTool
foreach ($p in ($Paths | Where-Object { $_ -and (Test-Path $_) })) {
  Write-Host "Signing $p"
  if ($Thumbprint) {
    & $signtool sign /sha1 $Thumbprint /fd $Digest /tr $TimestampUrl /td $Digest "$p"
  } else {
    & $signtool sign /f "$PfxPath" /p "$PfxPassword" /fd $Digest /tr $TimestampUrl /td $Digest "$p"
  }
  if ($LASTEXITCODE -ne 0) { throw "signing failed for $p" }
  & $signtool verify /pa /v "$p"
}
Write-Host 'Signing complete (timestamped).'
