param(
    [string]$OutputDirectory = (Join-Path (Split-Path $PSScriptRoot -Parent) 'artifacts')
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

# This is an external developer test oracle, never a RustMix runtime dependency.
# ffmpeg.org links Gyan's Windows builds; this exact vendor archive/hash is pinned.
$downloadUrl = 'https://www.gyan.dev/ffmpeg/builds/packages/ffmpeg-9.0.2-essentials_build.7z'
$checksumUrl = $downloadUrl + '.sha256'
$expectedArchiveHash = '4705843ccaaf54257c16ad90f3e952ece33c17df964ecf7bfdbb0f49c7171077'
$sourcePage = 'https://ffmpeg.org/download.html'
$runnerImagePage = 'https://github.com/actions/runner-images/blob/main/images/windows/Windows2022-Readme.md'

function Invoke-FixtureTool {
    param([string]$Executable, [string[]]$Arguments)
    $start = [System.Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $Executable
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    foreach ($argument in $Arguments) { $start.ArgumentList.Add($argument) }
    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $start
    try {
        if (-not $process.Start()) { throw "Could not start fixture tool: $Executable" }
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit(60000)) {
            $process.Kill($true)
            if (-not $process.WaitForExit(5000)) { throw 'Fixture tool did not terminate after timeout' }
            throw 'Fixture tool exceeded the 60-second limit'
        }
        $output = $stdout.GetAwaiter().GetResult()
        $errorOutput = $stderr.GetAwaiter().GetResult()
        if ($process.ExitCode -ne 0) {
            throw "Fixture tool exited $($process.ExitCode): $errorOutput"
        }
        return [PSCustomObject]@{ stdout = $output; stderr = $errorOutput }
    } finally {
        $process.Dispose()
    }
}

$artifacts = [System.IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Path $artifacts -Force | Out-Null
$fixturePath = Join-Path $artifacts 'video-fixture.mp4'
$manifestPath = Join-Path $artifacts 'video-fixture.json'
if ((Test-Path -LiteralPath $fixturePath) -or (Test-Path -LiteralPath $manifestPath)) {
    throw 'Video fixture outputs already exist; choose an empty output directory'
}

$temporaryRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [System.IO.Path]::GetTempPath() }
$taskDirectory = Join-Path $temporaryRoot ('rustmix-video-fixture-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $taskDirectory | Out-Null
$discovered = Get-Command ffmpeg -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
$origin = 'existing PATH executable; exact version and hash recorded below'
$archiveHash = $null
if ($null -ne $discovered) {
    $ffmpegPath = $discovered.Source
} else {
    $extractor = Get-Command 7z -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($null -eq $extractor) { throw 'Pinned fixture download requires the runner image 7-Zip executable' }
    $archive = Join-Path $taskDirectory 'ffmpeg-9.0.2-essentials_build.7z'
    Invoke-WebRequest -Uri $downloadUrl -OutFile $archive -TimeoutSec 120
    $archiveHash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($archiveHash -ne $expectedArchiveHash) { throw 'Pinned FFmpeg archive SHA256 mismatch; refusing to execute' }
    $extracted = Join-Path $taskDirectory 'developer-tool'
    $null = Invoke-FixtureTool -Executable $extractor.Source -Arguments @('x', $archive, ('-o' + $extracted), '-y', '-bso0', '-bsp0')
    $candidates = @(Get-ChildItem -LiteralPath $extracted -Filter ffmpeg.exe -File -Recurse)
    if ($candidates.Count -ne 1) { throw 'Pinned archive must contain exactly one ffmpeg.exe' }
    $ffmpegPath = $candidates[0].FullName
    $origin = 'Gyan 9.0.2 essentials Windows build, listed by ffmpeg.org; pinned archive SHA256 verified'
}

$version = Invoke-FixtureTool -Executable $ffmpegPath -Arguments @('-version')
$toolHash = (Get-FileHash -LiteralPath $ffmpegPath -Algorithm SHA256).Hash.ToLowerInvariant()

# Original RGB source: 60 frames of red/green/blue/white quadrants and a black
# 12x10 square moving one pixel per frame across the lower-right quadrant.
# No stock footage, proprietary assets, camera input or external media is used.
$width = 160
$height = 90
$frames = 60
$rawPath = Join-Path $taskDirectory 'original-quadrants.rgb24'
$pixels = [byte[]]::new($width * $height * 3)
$raw = [System.IO.File]::Open($rawPath, [System.IO.FileMode]::CreateNew, [System.IO.FileAccess]::Write)
try {
    for ($frame = 0; $frame -lt $frames; $frame++) {
        for ($y = 0; $y -lt $height; $y++) {
            for ($x = 0; $x -lt $width; $x++) {
                $offset = (($y * $width) + $x) * 3
                if ($y -lt 45) {
                    if ($x -lt 80) { $red = 255; $green = 0; $blue = 0 }
                    else { $red = 0; $green = 255; $blue = 0 }
                } elseif ($x -lt 80) {
                    $red = 0; $green = 0; $blue = 255
                } else {
                    $red = 255; $green = 255; $blue = 255
                }
                if ($y -ge 74 -and $y -lt 84 -and $x -ge (84 + $frame) -and $x -lt (96 + $frame)) {
                    $red = 0; $green = 0; $blue = 0
                }
                $pixels[$offset] = [byte]$red
                $pixels[$offset + 1] = [byte]$green
                $pixels[$offset + 2] = [byte]$blue
            }
        }
        $raw.Write($pixels, 0, $pixels.Length)
    }
    $raw.Flush($true)
} finally {
    $raw.Dispose()
}
$rawHash = (Get-FileHash -LiteralPath $rawPath -Algorithm SHA256).Hash.ToLowerInvariant()
$arguments = @(
    '-hide_banner', '-nostdin', '-loglevel', 'error', '-n',
    '-f', 'rawvideo', '-pixel_format', 'rgb24', '-video_size', '160x90', '-framerate', '30', '-i', $rawPath,
    '-frames:v', '60', '-an',
    '-vf', 'scale=out_color_matrix=bt709:out_range=limited,format=yuv420p',
    '-c:v', 'libx264', '-preset', 'veryfast', '-crf', '10', '-profile:v', 'baseline',
    '-bf', '0', '-g', '30', '-keyint_min', '30', '-sc_threshold', '0',
    '-colorspace', 'bt709', '-color_primaries', 'bt709', '-color_trc', 'bt709', '-color_range', 'tv',
    '-movflags', '+faststart', '-f', 'mp4', $fixturePath
)
$encoding = Invoke-FixtureTool -Executable $ffmpegPath -Arguments $arguments
if (-not (Test-Path -LiteralPath $fixturePath -PathType Leaf)) { throw 'FFmpeg did not create the expected MP4 fixture' }
$fixture = Get-Item -LiteralPath $fixturePath
if ($fixture.Length -lt 1000 -or $fixture.Length -gt 1048576) { throw 'Generated MP4 fixture has an unexpected size' }
$manifest = [ordered]@{
    schema = 1
    generated_utc = [DateTime]::UtcNow.ToString('o')
    purpose = 'External developer fixture generator/oracle only; FFmpeg is never linked, bundled or used by RustMix at runtime'
    source = 'Original RustMix procedural RGB quadrants with moving black square, generated by scripts/ci-video-fixture.ps1'
    source_ownership = 'Original project test vectors; no third-party footage or media assets'
    generator_script_sha256 = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash.ToLowerInvariant()
    width = $width
    height = $height
    frame_rate = '30/1'
    frames = $frames
    duration_seconds = 2
    raw_pixel_format = 'rgb24'
    encoded_video = 'H.264 Baseline, yuv420p, limited BT.709, GOP 30, no B frames, no audio'
    raw_source_sha256 = $rawHash
    fixture_path = $fixture.FullName
    fixture_sha256 = (Get-FileHash -LiteralPath $fixturePath -Algorithm SHA256).Hash.ToLowerInvariant()
    fixture_bytes = $fixture.Length
    ffmpeg = [ordered]@{
        origin = $origin
        path = $ffmpegPath
        executable_sha256 = $toolHash
        version_output = $version.stdout
        source_page = $sourcePage
        vendor_archive_url = $downloadUrl
        vendor_checksum_url = $checksumUrl
        pinned_archive_sha256 = $expectedArchiveHash
        downloaded_archive_sha256 = $archiveHash
        runner_inventory = $runnerImagePage
        licence_boundary = 'Gyan builds are GPLv3 developer tools. They and their libraries are excluded from RustMix packages and evidence uploads.'
    }
    command = [ordered]@{ executable = $ffmpegPath; arguments = $arguments }
    encoder_stderr = $encoding.stderr
}
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifestPath -Encoding utf8
$env:RUSTMIX_TEST_VIDEO = $fixture.FullName
if ($env:GITHUB_ENV) {
    Add-Content -LiteralPath $env:GITHUB_ENV -Value ('RUSTMIX_TEST_VIDEO=' + $fixture.FullName) -Encoding utf8
}
Write-Output ('Generated original video fixture: ' + $fixture.FullName)
Write-Output ('Recorded provenance: ' + $manifestPath)
