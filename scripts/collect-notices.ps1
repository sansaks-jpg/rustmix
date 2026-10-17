$ErrorActionPreference = 'Stop'
$metadataText = cargo metadata --locked --format-version 1
if ($LASTEXITCODE -ne 0) { throw 'Dependency inventory failed' }
$metadata = $metadataText | ConvertFrom-Json
$metadataText | Out-File -Encoding utf8 dist/dependency-manifest.json
$output = [System.Text.StringBuilder]::new()
[void]$output.AppendLine('RustMix dependency notices — all locked platforms, including development dependencies')
foreach ($package in $metadata.packages | Sort-Object name,version) {
    [void]$output.AppendLine("`n=== $($package.name) $($package.version) ===`nLicense expression: $($package.license)`nSource: $($package.repository)")
    $root = Split-Path $package.manifest_path
    $candidates = @(Get-ChildItem -LiteralPath $root -File | Where-Object { $_.Name -match '(?i)^(LICENSE|COPYING|NOTICE|COPYRIGHT|UNLICENSE)' })
    if ($package.license_file) {
        $declared = Join-Path $root $package.license_file
        if (Test-Path -LiteralPath $declared) { $candidates += Get-Item -LiteralPath $declared }
    }
    if ($package.name -eq 'epaint_default_fonts') { $candidates += Get-ChildItem -LiteralPath (Join-Path $root 'fonts') -Filter '*.txt' -File }
    foreach ($file in $candidates | Sort-Object FullName -Unique) {
        [void]$output.AppendLine("--- $($file.Name) ---")
        [void]$output.AppendLine([IO.File]::ReadAllText($file.FullName))
    }
    if ($candidates.Count -eq 0) { [void]$output.AppendLine('No license text file in published package. Refer to the declared license and source repository; inventory requires distribution review.') }
}
[IO.File]::WriteAllText((Join-Path (Get-Location) 'dist/THIRD_PARTY_LICENSES.txt'),$output.ToString())
