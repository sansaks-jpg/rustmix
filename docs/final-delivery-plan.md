# RustMix complete-production delivery

2026-10-19. Owner requested a complete native live-production application, including audio,
video, PNG, overlays, browser input, streaming, external output and fullscreen on an extended
display. This expands the earlier UI-only scope. All 30 areas in reference-audit.json remain
tracked; a completed UI control never substitutes for a working backend or acceptance evidence.

The owner explicitly authorized commits/pushes to a codex branch and hosted Windows CI. Keep
local builds disabled, perform native runtime checks on the runner, and retrieve actual evidence.
No public deployment, internet broadcast, personal capture or restrictive SDK approval is implied.

Delivery order and required evidence:

1. Validate the existing schema-6 output configuration slice, including actual external editor
   clicks, rejection/Cancel and project reopen. Preserve the first failing CI and its correction.
2. Independent Program window: move to an extended display, fullscreen/F11 and Escape, unchanged
   operator window and Preview, actual GPU screenshot pixels after overlay/fade/FTB, safe closed
   startup. Hosted single-monitor evidence does not qualify physical multi-monitor/hotplug.
3. Actual audio: bounded WAV PCM ingest/resampling, selected microphone/device capture, 48 kHz
   sample mixer with Master/A/B, gain/mute/solo/pan/sends, real meters and two-guest mix-minus.
   Explicit monitor Start/Stop; malformed file and no-device/fault handling. Verify impulses and
   callbacks without hardware, then physical sound and AV timestamps/clock compensation.
4. Local video and cameras: typed source lifecycle, OS Media Foundation decoding, bounded live
   frame store independent of UI, immutable PTS/sequence/color metadata, Preview/Program ingest,
   transport, EOS/loop/seek and offline placeholders. Generate original test media; compare against
   external developer-only decoding oracles. UVC acceptance is separate from capture-card SDKs.
5. Composition: real PNG alpha/source overlays/keying/PIP, correct deterministic ordering and
   transforms, stinger frame playback at the exact cut point, CPU vectors and GPU parity. Preserve
   current authored title panels while extending source-based overlays.
6. Recording and outputs: independent output consumers, Windows encoding worker, explicit format
   capabilities, recoverable segments/manifest, real path/state/elapsed/disk/queue/drop metrics,
   stop/finalize, disk-full/fault recovery, independent playback with a decoder. SDI/virtual-camera
   backends require their own driver/capability evidence; configuration slots are not devices.
7. Streaming: real encoded audio/video packets to a local test server, RTMP/RTMPS connection,
   timestamp/keyframe/sequence-header correctness, bounded backpressure/reconnect and stop.
   Keep stream keys outside project/logs. Internet streaming remains an explicit operator action.
8. Browser and network/guests: review browser runtime terms before bundling/installation,
   operator URL/configuration and real composited pixels/audio, isolation/crash recovery, network
   input jitter/reconnect and guest mix-minus/consent. Snapshot APIs alone are not live browser
   playback. NDI/ASIO/VST3/proprietary capture runtimes require documented licence decisions.
9. Complete remaining reference workflows: advanced graphics/data/templates, clocks, multiview,
   secondary/clean feeds, API/CLI/controllers/tally/PTZ, replay/ISO and project/autosave recovery.
   Every item needs an independent task and scope-specific tests; never alias unsupported controls.
10. Final qualification: original polished operator UX at multiple DPI/viewports, physical device
    matrix, camera/audio disconnects, GPU loss, disk/network faults, 30-minute production workflow,
    60-minute resource stability, eight-hour soak, AV skew/cadence targets and measured percentiles,
    accessibility, licence/SBOM, package install/upgrade/recovery and release review.

Current implementation and run-by-run evidence belong in ui-progress.md, gaps.md and the feature
matrix. No part of this plan asserts full reference parity or production readiness. Remain explicit
when a system-runtime licence, actual hardware or an operator endpoint prevents final acceptance.
