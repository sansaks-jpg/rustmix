# Input parity review

Review of source at baseline `9db25fc` plus this change. The proprietary vMix
implementation is not in this checkout; this is a comparison against the
repository's recorded observations ([reference audit](reference-audit.md),
[reference notes](reference-notes.md)) and the publicly documented
[vMix feature list](https://www.vmix.com/software/features.aspx), not a source
code comparison or a claim of exact behavior. Historical inventory rows may
predate current implementations. A category or descriptor is not evidence of
a qualified input backend.

| Input | RustMix source today | Remaining work / qualification |
| --- | --- | --- |
| Local video | Media Foundation video worker, transport and bounded feed (`video_player.rs`, `media_foundation.rs`); MP4 was previously fixture-tested. This change permits M4V, MOV, AVI and WMV as unverified source candidates. The audio monitor separately attempts media audio or a same-stem WAV; this is not a synchronized pipeline. | Run real container/codec fixtures on Windows; add further formats, bounded streaming audio, A/V clock and hardware/endurance tests. Existing documented video acceptance uses only a small MP4 fixture. |
| Still images / photos | PNG add/drop path, bounded decode and image processing; Photos descriptors/workers exist. This change extends the standalone still-image path to JPEG/BMP as an unverified source candidate. | Run the new decoder, Add Input, drop, project reopen and relink tests; qualify native workflow and broader image formats. |
| Audio | WAV and microphone descriptors, PCM mixer and explicit Windows monitoring. WAV decoder is bounded and supports PCM16/float32 mono/stereo. | Compressed audio, long-file streaming, device compatibility, resampling quality and sync; physical capture/playback qualification. |
| Cameras | Media Foundation device/mode discovery and camera frame worker exist. | Driver blocking/cancellation, capture cards, embedded audio, format and device-loss handling, physical hardware matrix. |
| Browser | Browser source/worker code and descriptors exist. | Interaction, audio, frame rate, external runtime and process/device recovery; verify normal UI reachability and native behavior. |
| Generators | Procedural colours, bars, countdown and other sources exist. | Complete reference behaviours, clock triggers and visual/runtime acceptance. |
| Other | Several entries are explicitly unavailable in Add Input. | Video/audio lists, desktop capture, NDI/OMT/SRT and other streams, guest calls/Zoom, replay/delay, image sequences, PowerPoint, titles/templates and virtual sets require independent backends and, where applicable, SDK/licence review. |

## Additional local video container candidate

The existing Media Foundation file worker, engine descriptor, Add Input picker,
recent files and drop intake now accept MP4, M4V, MOV, AVI and WMV. The probe
must successfully open each file before the operator can commit it; supported
extensions do not guarantee a particular codec or installed Windows component.
This selection follows the [Microsoft Media Foundation format list](https://learn.microsoft.com/en-us/windows/win32/medfound/supported-media-formats-in-media-foundation)
and covers some of the formats on the [vMix feature list](https://www.vmix.com/software/features.aspx),
not all of them. No new codec dependency or SDK was added. The audio monitor,
clock and 4 GiB file limit are unchanged. Earlier RustMix versions may reject
projects referring to new extensions.

`scripts/ci-video-fixture.ps1` now generates original MOV/M4V H.264, AVI MJPEG
and WMV2 fixtures alongside the MP4 and records their hashes in the manifest.
New platform and drop tests require those actual fixtures. They and the source
changes have **not** been executed in this Linux environment; regenerate all
fixtures and run the Windows gate before claiming support. A Linux FFmpeg
check independently reproduced the four encoding recipes and decoded their
outputs; it did not execute the PowerShell script or exercise Media Foundation.
Other containers, codecs, variable frame rates, audio, long files and device
configurations are not qualified.

## Still-image input candidate

The Add Input page, file picker, Properties relink and drag-and-drop now share
bounded PNG/JPEG/BMP decoding and matching engine path validation. UI labels
identify all three formats; unsupported image formats remain rejected. Existing
PNG automation IDs and the project `media_path` field are retained. No new
runtime dependency was added. JPEG/BMP files are external references, not
embedded in projects; older RustMix builds that only accept PNG may reject
projects containing the new formats. Prepared tests cover encoding and decoding,
corrupt/mismatched files, Add Input, drop and project persistence. These are
source changes awaiting Rust and Windows verification, not accepted parity.

## Initial correctness fixes

- Photo slideshows, cameras and browser sources no longer claim to have an audio
  backend or acquire mixer strips. Local video files retain an audio control
  because the monitor attempts a file-audio path, but a file may lack a usable
  track. Capability and mixer widget regressions are prepared, not yet run.
- The video worker previously read `seek_ms` repeatedly from a persistent
  control snapshot, seeking on every iteration; a subsequent state update could
  also overwrite an unconsumed seek. Seek requests now coalesce to the latest
  target and are consumed once by the native owner. Overflow and decoder seek
  errors are reported instead of silently proceeding with the old position.
- The engine's shared procedural playback clock still clamps seeks to 60 seconds
  (`crates/engine/src/playback.rs`), while the video owner receives the requested
  media position separately. This can produce incorrect audio positions for long
  media; the fix above does not resolve that architectural gap.
- Video audio previously fell back to `audio.wav` or `audio-fixture.wav` in the
  video directory (and sometimes a workspace fixture), potentially selecting
  unrelated audio. Only a same-stem WAV is considered; otherwise the video
  file's own audio path is attempted as before. Same-stem automatic association
  is a RustMix-specific convenience, not a vMix-parity claim; it still needs an
  explicit operator-controlled design and A/V synchronization.

Regression tests were added alongside the source. This checkout has no Rust
compiler or Cargo, and no Windows hardware, so these tests and native behavior
have **not** been run here. `git diff --check` passed. A Tree-sitter Rust grammar
check found no syntax errors in the changed Rust files; it is not compilation,
type checking, Clippy, or a test run.

## Next implementation gates

1. On Windows, run formatting, strict Clippy, workspace tests with the generated
   MP4 fixture, and a native seek test covering paused/playing playback, repeated
   scrubs, seek failures and restart. Test audio with real videos containing audio
   and without audio, with and without sidecars, and record exactly what is heard.
2. Replace whole-file audio loading and implicit sidecar substitution with a
   bounded, timestamped and explicitly selected media-audio design; use a shared
   A/V clock. Test long clips, seek/loop, missing streams and decode errors.
3. Qualify cameras and microphones on physical devices, including disconnect,
   reconnect, mode mismatch, concurrent inputs and blocked driver calls.
4. Add input families incrementally with real backends, documented licensing,
   operator-visible errors and source-specific integration tests; do not enable
   unavailable entries merely to make the catalogue look complete.

No changes in this review establish full vMix parity or broadcast readiness.
