$ErrorActionPreference = 'Stop'
# This script is run only on a disposable GitHub-hosted Windows runner.
# The native app injects its own opt-in test events and captures its rendered GPU surface.
New-Item -ItemType Directory -Path artifacts -Force | Out-Null
$env:APPDATA = Join-Path $env:RUNNER_TEMP 'rustmix-smoke-appdata'
New-Item -ItemType Directory -Path $env:APPDATA -Force | Out-Null
function Invoke-NativeSmoke([string[]]$SmokeArguments) {
    $stderrPath = Join-Path artifacts ($SmokeArguments[0].TrimStart('-') + '-stderr.txt')
    $process = Start-Process -FilePath (Resolve-Path 'target/release/rustmix.exe') -ArgumentList $SmokeArguments -PassThru -WindowStyle Hidden -RedirectStandardError $stderrPath
    $samples = [System.Collections.Generic.List[object]]::new()
    $timer = [System.Diagnostics.Stopwatch]::StartNew()
    while (-not $process.HasExited) {
        if ($timer.Elapsed.TotalSeconds -gt 270) {
            Stop-Process -Id $process.Id -Force
            throw 'Native smoke exceeded 270 seconds on GitHub runner'
        }
        try {
            $process.Refresh()
            $samples.Add([pscustomobject]@{ elapsedMs = $timer.ElapsedMilliseconds; workingSetBytes = $process.WorkingSet64; peakWorkingSetBytes = $process.PeakWorkingSet64; privateBytes = $process.PrivateMemorySize64; cpuMs = $process.TotalProcessorTime.TotalMilliseconds })
        } catch { if (-not $process.HasExited) { throw } }
        $null = $process.WaitForExit(500)
    }
    $process.WaitForExit()
    if ($samples.Count -eq 0) { throw 'Native memory sampler produced no evidence' }
    $measurementPath = Join-Path artifacts ($SmokeArguments[0].TrimStart('-') + '-memory.json')
    $peakBytes = ($samples | ForEach-Object { $_.peakWorkingSetBytes } | Measure-Object -Maximum).Maximum
    $privateBytes = ($samples | ForEach-Object { $_.privateBytes } | Measure-Object -Maximum).Maximum
    if ($null -eq $peakBytes -or $peakBytes -le 0 -or $null -eq $privateBytes -or $privateBytes -le 0) { throw 'Missing positive process memory measurements' }
    [ordered]@{
        arguments = $SmokeArguments; elapsedMs = $timer.ElapsedMilliseconds
        peakWorkingSetBytes = $peakBytes
        maxPrivateBytes = $privateBytes
        samples = $samples
    } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $measurementPath
    if ($process.ExitCode -ne 0) { throw "Native app exit code $($process.ExitCode)" }
    if (Test-Path artifacts/profile-error.txt) { throw (Get-Content artifacts/profile-error.txt -Raw) }
    if (Test-Path artifacts/smoke-error.txt) { throw (Get-Content artifacts/smoke-error.txt -Raw) }
}
Invoke-NativeSmoke @('--ui-smoke')
foreach ($file in @('smoke-ok.json','workspace-1100.png','workspace-1600.png','workspace-1920.png','smoke.rustmix.json','ui-elements.json','layout-stability.json','pause-transport.json','paused-program.png')) {
    if (-not (Test-Path (Join-Path artifacts $file))) { throw "Missing native evidence: $file" }
}
foreach ($dialog in @('add-input-1600','input-properties-1600','settings-1600','overlay-settings-1600','recording-setup-1600','streaming-setup-1600','multicorder-1600','playlist-1600','shortcuts-1600')) {
    foreach ($suffix in @('.png','-elements.json')) {
        $evidence = Join-Path artifacts ($dialog + $suffix)
        if (-not (Test-Path -LiteralPath $evidence)) { throw "Missing dialog evidence: $evidence" }
    }
}
Invoke-NativeSmoke @('--ui-smoke-reopen','artifacts/smoke.rustmix.json')
if (-not (Test-Path artifacts/reopen-ok.json)) { throw 'Reopen did not validate the saved production' }
Invoke-NativeSmoke @('--ui-input-smoke')
Invoke-NativeSmoke @('--ui-input-smoke-reopen','artifacts/inputs.rustmix.json')
foreach ($file in @('input-fixture.png','input-workflow.png','input-properties.png','input-playlist.png','input-keyboard.png','input-reopened.png','input-smoke-ok.json','input-reopen-ok.json','inputs.rustmix.json')) {
    if (-not (Test-Path (Join-Path artifacts $file))) { throw "Missing PNG workflow evidence: $file" }
}
Invoke-NativeSmoke @('--ui-perf')
$cadence = Get-Content artifacts/program-cadence.json -Raw | ConvertFrom-Json
if ($cadence.samples.Count -ne 3) { throw 'Missing frame-rate measurement samples' }
foreach ($sample in $cadence.samples) {
    if ($sample.frames_produced -le 0 -or -not $sample.moving_pixels) { throw 'No real moving Program frames in profile' }
    if (-not $sample.within_sample_target) { Write-Warning "Program cadence target was not met on the hosted runner: $($sample | ConvertTo-Json -Compress)" }
}
Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors | ConvertTo-Json | Out-File artifacts/runner-cpu.json
Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion | ConvertTo-Json | Out-File artifacts/runner-graphics.json
"Windows runner: $([System.Environment]::OSVersion.VersionString)" | Out-File artifacts/platform.txt
