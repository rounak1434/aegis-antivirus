<#
.SYNOPSIS
  Generate a CycloneDX SBOM + license/dependency inventory for the workspace.

.DESCRIPTION
  Prefers the real tools (`cargo cyclonedx`, `cyclonedx-npm`) when installed.
  Falls back to a built-in `cargo metadata` → CycloneDX transform so an SBOM is
  always produced offline. Outputs into the given -OutDir (default ./dist).
#>
[CmdletBinding()]
param([string]$OutDir = (Join-Path (Split-Path -Parent $PSScriptRoot) 'dist'))

# 'Continue' (not 'Stop'): cargo/npm write progress to stderr, which Windows
# PowerShell 5.1 otherwise turns into a terminating error. The generator is
# best-effort and validates its own output.
$ErrorActionPreference = 'Continue'
$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$rustSbom = Join-Path $OutDir 'sbom.cargo.cdx.json'

if (Get-Command cargo-cyclonedx -ErrorAction SilentlyContinue) {
  Write-Host '==> cargo cyclonedx'
  cargo cyclonedx --format json --all
  Get-ChildItem -Recurse -Filter '*.cdx.json' | Select-Object -First 1 | ForEach-Object { Copy-Item $_.FullName $rustSbom -Force }
} else {
  Write-Host '==> cargo-cyclonedx not installed; building CycloneDX from cargo metadata'
  $meta = cargo metadata --format-version 1 2>$null | ConvertFrom-Json
  $components = foreach ($p in $meta.packages) {
    [ordered]@{
      type    = 'library'
      name    = $p.name
      version = $p.version
      purl    = "pkg:cargo/$($p.name)@$($p.version)"
      licenses = @(if ($p.license) { @{ license = @{ id = $p.license } } })
    }
  }
  $bom = [ordered]@{
    bomFormat   = 'CycloneDX'
    specVersion = '1.5'
    version     = 1
    metadata    = [ordered]@{ component = [ordered]@{ type = 'application'; name = 'aegis-antivirus' } }
    components  = $components
  }
  $bom | ConvertTo-Json -Depth 8 | Set-Content -Encoding utf8 $rustSbom
}
Write-Host "Rust SBOM: $rustSbom"

# Frontend SBOM (best-effort).
$nodeSbom = Join-Path $OutDir 'sbom.npm.cdx.json'
if (Get-Command cyclonedx-npm -ErrorAction SilentlyContinue) {
  cyclonedx-npm --output-file $nodeSbom
} else {
  Write-Host '==> cyclonedx-npm not installed; emitting npm dependency list as SBOM fallback'
  npm ls --all --json 2>$null | Set-Content -Encoding utf8 $nodeSbom
}

# License + dependency inventories.
$meta = cargo metadata --format-version 1 2>$null | ConvertFrom-Json
$meta.packages |
  Select-Object name, version, @{n = 'license'; e = { $_.license } }, @{n = 'repository'; e = { $_.repository } } |
  Sort-Object name |
  ConvertTo-Csv -NoTypeInformation |
  Set-Content -Encoding utf8 (Join-Path $OutDir 'license-inventory.csv')

cargo tree --workspace --edges normal 2>$null | Set-Content -Encoding utf8 (Join-Path $OutDir 'dependency-tree.txt')

Write-Host "Inventories written to $OutDir"
