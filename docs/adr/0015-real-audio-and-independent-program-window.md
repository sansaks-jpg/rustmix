# ADR 0015: real sample audio and an independent Program window

2026-10-19 · implementation in progress, not runtime acceptance.

The operator needs actual audio samples and a Program-only output on an extended display.
Simulated meters and fullscreen of the control interface cannot satisfy these tasks.

Decision: add a portable 48 kHz stereo block mixer and strict WAV PCM decoder to the headless
engine. Route Master/A/B and destination-aware guest returns over validated source IDs, apply
click-safe gain ramps and expose actual peak/RMS/clipping/underruns. Preallocate render buffers.
Keep existing simulated meters explicitly distinguished until a real source/monitor is active.

Use a small `rustmix-platform` crate for Windows WASAPI through safe CPAL 0.18.2 APIs. Its
Apache-2.0 licence and Rust 1.85 MSRV were verified from registry metadata and the downloaded
manifest; default features are empty and ASIO remains disabled. Use rtrb 0.4.0 (MIT OR Apache-2.0,
MSRV 1.38) for fixed-capacity SPSC PCM transfer. Callback paths only copy samples, sanitize invalid
values and update atomics. Enumeration, stream construction, file I/O, configuration and mixing
or resampling happen on control/worker threads. Start/stop are explicit operator actions; there
is no automatic microphone access. Device faults produce silence and visible status, preserving
video and routing; reopening requires operator action in this first backend.

Use egui/eframe's pinned native secondary viewport for a separate Program-only window. It follows
the committed Program frame, including transitions, overlays and FTB, and does not alter Preview.
Move it to the desired extended display and explicitly enter fullscreen; no invented monitor
enumeration or automatic placement. Escape closes only the output window. Keep the existing
application-fullscreen action separate and never auto-start outputs after project load.

Acceptance: pure impulse/routing/mute/mix-minus/WAV-failure tests, deterministic PCM callback
tests without opening hardware, widget tests for operator controls, hosted native child-viewport
pixel/capture tests, and subsequent physical extended-monitor/audio-device qualification. A hosted
runner without audio devices must not count as hardware acceptance. AV clock synchronization,
sample-rate compensation, compressed audio/video, full recording/stream encoders and physical
device/reconnect/endurance tests remain separate open work.
