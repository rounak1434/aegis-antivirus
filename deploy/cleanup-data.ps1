<#
.SYNOPSIS
  Full cleanup of Aegis user data under %ProgramData%\Aegis (DESTRUCTIVE).

.DESCRIPTION
  The installer/uninstaller intentionally PRESERVES user data (settings,
  database, quarantine vault, signatures, YARA rules) across upgrade and
  uninstall. Run this only when you want a complete wipe.

.PARAMETER Force
  Skip the confirmation prompt (for unattended cleanup).
#>
[CmdletBinding()]
param([switch]$Force)

$ErrorActionPreference = 'Stop'
$DataDir = Join-Path $env:ProgramData 'Aegis'

if (-not (Test-Path $DataDir)) {
  Write-Host "Nothing to remove: $DataDir does not exist."
  return
}

if (-not $Force) {
  Write-Warning "This permanently deletes ALL Aegis data: $DataDir"
  Write-Warning '(settings, database, quarantine vault, signatures, YARA rules)'
  $answer = Read-Host 'Type DELETE to confirm'
  if ($answer -ne 'DELETE') { Write-Host 'Aborted.'; return }
}

Remove-Item -Recurse -Force $DataDir
Write-Host "Removed $DataDir"
