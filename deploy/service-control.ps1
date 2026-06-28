<#
.SYNOPSIS
  Install, start, stop, restart, remove, or query the AegisService Windows
  service. Run from an elevated (Administrator) PowerShell.

.PARAMETER Action
  install | start | stop | restart | remove | status

.PARAMETER BinPath
  Path to aegis-service.exe (required for 'install'). Defaults to the installed
  location under Program Files.

.EXAMPLE
  ./service-control.ps1 -Action install -BinPath "C:\Program Files\Aegis Antivirus\aegis-service.exe"
  ./service-control.ps1 -Action restart
  ./service-control.ps1 -Action remove
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory = $true)]
  [ValidateSet('install', 'start', 'stop', 'restart', 'remove', 'status')]
  [string]$Action,

  [string]$BinPath = "$env:ProgramFiles\Aegis Antivirus\aegis-service.exe"
)

$ErrorActionPreference = 'Stop'
$ServiceName = 'AegisService'
$DisplayName = 'Aegis Security Service'
$DataDir = Join-Path $env:ProgramData 'Aegis'
$LogDir = Join-Path $DataDir 'Logs'

function Assert-Admin {
  $id = [Security.Principal.WindowsIdentity]::GetCurrent()
  $p = New-Object Security.Principal.WindowsPrincipal($id)
  if (-not $p.IsInRole([Security.Principal.WindowsBuiltinRole]::Administrator)) {
    throw 'This action requires an elevated (Administrator) PowerShell.'
  }
}

function Write-Log([string]$msg) {
  New-Item -ItemType Directory -Force -Path $LogDir | Out-Null
  $line = "[{0}] {1}" -f (Get-Date -Format o), $msg
  Add-Content -Path (Join-Path $LogDir 'service.log') -Value $line
  Write-Host $line
}

function New-DataDirs {
  foreach ($d in @('Updates', 'Quarantine', 'Logs', 'Database')) {
    New-Item -ItemType Directory -Force -Path (Join-Path $DataDir $d) | Out-Null
  }
}

switch ($Action) {
  'install' {
    Assert-Admin
    if (-not (Test-Path $BinPath)) { throw "Service binary not found: $BinPath" }
    New-DataDirs
    sc.exe stop $ServiceName | Out-Null
    sc.exe delete $ServiceName | Out-Null
    sc.exe create $ServiceName binPath= "`"$BinPath`"" start= auto DisplayName= "$DisplayName"
    sc.exe description $ServiceName "Aegis Antivirus background protection (scanning, real-time, quarantine, updates)."
    # Crash recovery: restart on failure, reset the count daily.
    sc.exe failure $ServiceName reset= 86400 actions= restart/60000/restart/60000/restart/120000
    sc.exe failureflag $ServiceName 1
    sc.exe start $ServiceName
    Write-Log "installed + started ($BinPath)"
  }
  'start' { Assert-Admin; sc.exe start $ServiceName; Write-Log 'start' }
  'stop' { Assert-Admin; sc.exe stop $ServiceName; Write-Log 'stop' }
  'restart' {
    Assert-Admin
    sc.exe stop $ServiceName | Out-Null
    Start-Sleep -Seconds 2
    sc.exe start $ServiceName
    Write-Log 'restart'
  }
  'remove' {
    Assert-Admin
    sc.exe stop $ServiceName | Out-Null
    sc.exe delete $ServiceName
    Write-Log 'removed (user data preserved)'
  }
  'status' {
    sc.exe query $ServiceName
    sc.exe qfailure $ServiceName
  }
}
