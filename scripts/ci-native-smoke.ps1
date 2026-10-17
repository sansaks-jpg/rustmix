$ErrorActionPreference = 'Stop'
# This script is run only on a disposable GitHub-hosted Windows runner.
# The native app injects its own opt-in test events and captures its rendered GPU surface.
New-Item -ItemType Directory -Path artifacts -Force | Out-Null
$env:APPDATA = Join-Path $env:RUNNER_TEMP 'rustmix-smoke-appdata'
New-Item -ItemType Directory -Path $env:APPDATA -Force | Out-Null
function Invoke-NativeSmoke([string[]]$SmokeArguments) {
    $process = Start-Process -FilePath (Resolve-Path 'target/release/rustmix.exe') -ArgumentList $SmokeArguments -PassThru -WindowStyle Hidden -RedirectStandardError artifacts/native-stderr.txt
    if (-not $process.WaitForExit(210000)) {
        Stop-Process -Id $process.Id -Force
        throw 'Native smoke exceeded 210 seconds on GitHub runner'
    }
    if ($process.ExitCode -ne 0) { throw "Native app exit code $($process.ExitCode)" }
    if (Test-Path artifacts/smoke-error.txt) { throw (Get-Content artifacts/smoke-error.txt -Raw) }
}
Invoke-NativeSmoke @('--ui-smoke')
foreach ($file in @('smoke-ok.json','workspace-1600.png','workspace-1920.png','smoke.rustmix.json','ui-elements.json')) {
    if (-not (Test-Path (Join-Path artifacts $file))) { throw "Missing native evidence: $file" }
}
foreach ($size in @('1100','1600','1920')) {
    foreach ($dialog in @('add-input','input-properties','settings')) {
        foreach ($suffix in @('.png','-elements.json')) {
            $evidence = Join-Path artifacts ($dialog + '-' + $size + $suffix)
            if (-not (Test-Path -LiteralPath $evidence)) { throw "Missing native dialog evidence: $evidence" }
        }
    }
}
foreach ($suffix in @('.png','-elements.json')) {
    $overlayEvidence = Join-Path artifacts ('overlay-settings-1600' + $suffix)
    if (-not (Test-Path -LiteralPath $overlayEvidence)) { throw "Missing overlay dialog evidence: $overlayEvidence" }
}
Invoke-NativeSmoke @('--ui-smoke-reopen','artifacts/smoke.rustmix.json')
if (-not (Test-Path artifacts/reopen-ok.json)) { throw 'Reopen did not validate the saved production' }
Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion | ConvertTo-Json | Out-File artifacts/runner-graphics.json
"Windows runner: $([System.Environment]::OSVersion.VersionString)" | Out-File artifacts/platform.txt
