<#
.SYNOPSIS
  Verify a downloaded Aegis release: checksums, signature file, and Authenticode
  signatures on the executables/installers.

.PARAMETER Dir
  Directory containing the release artifacts + SHA256SUMS (default ./dist).
#>
[CmdletBinding()]
param([string]$Dir = (Join-Path (Split-Path -Parent $PSScriptRoot) 'dist'))

$ErrorActionPreference = 'Stop'
$fail = 0
$sums = Join-Path $Dir 'SHA256SUMS'
if (-not (Test-Path $sums)) { throw "SHA256SUMS not found in $Dir" }

Write-Host '== Checksums =='
foreach ($line in Get-Content $sums) {
  if (-not $line.Trim()) { continue }
  $hash, $name = $line -split '\s+', 2
  $name = $name.Trim()
  $path = Join-Path $Dir $name
  if (-not (Test-Path $path)) { Write-Warning "MISSING  $name"; $fail++; continue }
  $actual = (Get-FileHash $path -Algorithm SHA256).Hash.ToLower()
  if ($actual -eq $hash.ToLower()) { Write-Host "OK       $name" }
  else { Write-Warning "MISMATCH $name"; $fail++ }
}

Write-Host '== Checksum signature =='
if (Test-Path "$sums.sig") {
  if (Get-Command gpg -ErrorAction SilentlyContinue) {
    gpg --verify "$sums.sig" $sums; if ($LASTEXITCODE -ne 0) { $fail++ }
  } else { Write-Warning 'gpg not available to verify SHA256SUMS.sig' }
} else { Write-Host 'No SHA256SUMS.sig (unsigned checksums).' }

Write-Host '== Authenticode signatures =='
Get-ChildItem $Dir -Include *.exe, *.msi -Recurse | ForEach-Object {
  $s = Get-AuthenticodeSignature $_.FullName
  $tag = if ($s.Status -eq 'Valid') { 'SIGNED  ' } elseif ($s.Status -eq 'NotSigned') { 'UNSIGNED' } else { "BAD($($s.Status))"; $script:fail++ }
  Write-Host "$tag $($_.Name)"
}

if ($fail -gt 0) { Write-Error "$fail verification failure(s)"; exit 1 }
Write-Host 'Release integrity OK.'
