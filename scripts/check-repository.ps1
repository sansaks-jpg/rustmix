$ErrorActionPreference = 'Stop'
$metadata = cargo metadata --no-deps --locked --format-version 1 | ConvertFrom-Json
if ($LASTEXITCODE -ne 0) { throw 'Cargo metadata failed' }
$allowed = @{
    'rustmix-engine' = @('serde','serde_json','thiserror','tempfile')
    'rustmix-ui' = @('rustmix-engine','eframe','egui_dock','serde','serde_json','rfd','image','tempfile')
    'rustmix' = @('rustmix-ui','eframe')
}
foreach ($package in $metadata.packages) {
    if (-not $allowed.ContainsKey($package.name)) { throw "Unreviewed workspace package: $($package.name)" }
    foreach ($dependency in $package.dependencies) {
        if ($dependency.name -notin $allowed[$package.name]) { throw "Unreviewed layer/dependency edge: $($package.name) -> $($dependency.name)" }
    }
}
if (Test-Path assets) {
    foreach ($asset in Get-ChildItem assets -File -Recurse | Where-Object { $_.Extension -ne '.attribution' }) {
        if (-not (Test-Path ($asset.FullName + '.attribution'))) { throw "Missing asset sidecar: $($asset.Name)" }
        if (-not (Select-String -LiteralPath ATTRIBUTION.md -SimpleMatch $asset.Name -Quiet)) { throw "Missing asset index row: $($asset.Name)" }
    }
}
& (Join-Path $PSScriptRoot 'check-reference-coverage.ps1')
Write-Output 'Repository layer, asset and reference inventory checks passed (not full feature acceptance)'
