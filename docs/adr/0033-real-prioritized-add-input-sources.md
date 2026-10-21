# ADR 0033: Real prioritized Add Input sources

Date: 2026-10-21
Status: implemented source candidate; exact hosted checks and native source evidence pending.
Scope: Photos, Browser and Camera; preserve existing MP4/PNG/WAV/microphone flow.

UI-032 was accepted on source4850b0f/run39587449408 attempt3. Earlier attempts
failed GPU queue destruction after successful workflow assertions; those failures
remain open and are not considered fixed by adding inputs.

Schema9 adds typed VideoSource variants Photos(paths,interval), Browser(URL,viewport)
and Camera(stable symbolic link). Existing schema1–8 projects migrate through the
validated loader; old binaries reject schema9. AddInputAt remains the sole commit
route. No old PNG path is reused and Program/Preview do not change until Take.

All source owners publish immutable timestamped VideoFeed frames on the existing
bounded owner-worker system (eight permits, including workers still retiring).
UI/main thread performs no directory scan, decode, COM/device opening or browser
control. Sources are rendered by the existing CPU reference compositor into
Preview/Program. On a source error retain its last valid raster with a visible
warning. This is an experimental CPU path, not hardware/latency/zero-copy parity.

Photos scans one folder nonrecursively, at most1024 entries and256 PNG/JPEG/BMP
files, sorted alphabetically. All selected files decode before OK. Each decode
limits encoded bytes32MiB, raster4096px, allocation64MiB and caches a640x360
raster. Playback reads the current image only, observes pause/restart/loop and
uses a monotonic clock. Source file changes/missing files yield visible failure;
there is no background directory rescan or silent substitution.

Camera reuses the audited Media Foundation RGB32 source reader with a new
selected-device adapter. Enumeration does not activate a device. Check explicitly
opens/closes the selected source and requires a real frame; OK starts that camera.
A stable symbolic link is retained. Permission/driver/disconnect errors do not
fall back to another camera. SourceReader reads may block a driver worker;
retiring workers retain permits. Automatic reconnect and hardware/capture-card
qualification remain open. Camera audio is a separate Audio Input.

Browser launches installed Edge/Chrome with a fresh disposable profile, loopback
CDP control and video-only PNG screenshots capped at5fps. A persistent page keeps
its JS/CSS state. Viewport is bounded64–1920 by64–1080; final raster at most640x360.
Only HTTP(S) URLs without embedded credentials are accepted. No user signed-in
profile, SDK, Chromium binary or licensed runtime is bundled. Installed runtime
licence/update/compatibility remains the operator's environment. Sandbox stays
enabled; there are no bypasses for HTTPS certificates or site authentication.
Browser process stdout/stderr is suppressed so URLs/secrets are not logged.
Control I/O has timeouts, WebSocket/PNG limits and finite event/startup loops.
A source owner kills only its spawned process tree and removes its profile on
normal stop/failure. No site mouse/keyboard interaction, browser audio, local HTML
file navigation or 30/60fps claim is included in this slice.

Camera and Browser require explicit Start source after project reopen. Properties
provide Start/Stop, and adding one explicitly arms only the new source. Photos
and local files retain existing playback semantics. Audio monitoring/capture,
recording and streaming are never started by checking/adding these video sources.

Dependencies reviewed from actual locked metadata: tungstenite0.30.0
(MIT OR Apache-2.0, MSRV1.85), base640.22.1(MIT OR Apache-2.0), existing image0.25.10
(MIT OR Apache-2.0) enables JPEG/BMP; existing tempfile becomes a runtime edge.
The16 newly locked packages all declare MIT and/or Apache-2.0. TLS remains in the
installed browser, not the CDP WebSocket. Windows bindings remain0.62.2; no new
FFmpeg/GStreamer/codec/ASIO/NDI/VST runtime is introduced. Existing package notices
collect the actual full locked dependency set. Platform FFI is restricted to the
existing audited media_foundation module with documented ownership/thread/buffer
preconditions. Browser CPU/process overhead and dependency security need normal
release review; feature presence does not imply production readiness.

Verification: new descriptor validation/atomic persistence and UI stale-result,
readiness/retirement/reopen tests. New mandatory --ui-source-smoke runs first for
fast diagnosis, with all16 old processes still required afterwards. It generates
PNG/JPEG/BMP fixtures plus a loopback HTML page with changing colours, clicks
Check/OK, requires real photo pixel changes, Browser Take and changing Program
pixels, PNG/element captures, schema9 roundtrip and no recording/stream/audio.
Camera evidence is enumeration only; physical camera capture cannot be certified
by a generic hosted runner. Every native process still requires stage/adapter,
screenshot readback, finite deadline and successful GPU drain. No local Rust
compile/test/build/native launch is permitted; hosted validation is mandatory.
