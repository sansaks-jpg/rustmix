# ADR 0034: New Preset and native input formats

2026-10-21. Source candidate. Hosted compilation, tests, screenshots and performance acceptance pending.

The requested scope is Add Input (Video, PNG, Photos, Browser, Camera, WAV and
Audio Input), plus a format selector on normal application startup and New.
References were viewed locally in `vmix-photos/screenshots/23*.jpg` and the
user's supplied New Preset image. No reference artwork is included in RustMix.

`engine::video_format` owns typed scan, rational frame-rate and validated canvas
configuration. `Engine::new_preset` constructs a blank session with two muted
black safety inputs required by the current independent Program/Preview model;
it opens no device, media file, browser or output. `ui::new_preset` owns only the
draft, modal interaction, explicit acknowledgement for replacing unsaved work,
and session/cache retirement. Startup without a project file uses the same blank
constructor; explicit project opening and existing offline test modes bypass
the startup selector. The new hosted preset mode exercises normal initialization.

Schema 10 adds `Preferences.scan` and an optional exact Camera format descriptor.
Schemas 1-9 migrate with progressive timing and default camera negotiation.
Old binaries reject schema 10. All new camera selections store size, reduced rate,
field order and pixel subtype. No enum ordinal or native-mode list index is saved.

New Preset and Display offer landscape/portrait canvases including 1920x1080 and
1080x1920, progressive 23.976/24/25/29.97/30/50/59.94/60, and interlaced
25/29.97/30 frames per second with top/bottom field order. Interlaced labels show
both field rate and frame rate: 50i is 25 frames/s. These are project format
settings. The current compositor/display remains progressive and intentionally
bounded to 640x360 or 360x640 (same 230400 pixels); it does not encode, weave fields,
or produce full-resolution/interlaced broadcast output. The UI states this limit.
Portrait Program, Preview, tiles, separate output and PNG snapshots preserve
orientation. Surface cache identities include raster size to reject stale layouts.

Camera format discovery runs off the UI thread, opens only the selected stable
device identity, enumerates at most 256 native types, and reads no frames. It
does not require the device's default type to decode before listing other types.
Only bounded progressive/interleaved formats with valid rates are offered. Check
and runtime startup find and set the exact native type again, request progressive
RGB32 monitoring at the selected frame rate and fail visibly if unavailable.
Interlaced source conversion uses the existing Windows video processor; no
interlaced output support or hardware acceptance is inferred.

Browser now accepts 1080x1920 as well as 1920x1080, with sides 64-1920 and a
2073600-pixel area limit. Screenshots remain capped at 5fps. A separate two-owner
permit bounds browser processes including temporary probes and retiring owners;
the existing eight-owner source limit also remains. DevTools discovery reads the
declared HTTP response length instead of waiting for EOF on a persistent socket.
HTTP headers/body, duplicate lengths and truncated/oversized responses are bounded
and rejected. Browser audio/interaction remains unavailable. MP4 remains video-only;
Audio supports the existing bounded PCM16/float32 WAV path, not MP3.

No new dependency, SDK or bundled runtime was added. Existing Windows bindings
0.62.2, image, tungstenite and base64 remain locked. Windows integration follows
[native media type selection](https://learn.microsoft.com/en-us/windows/win32/api/mfreadwrite/nf-mfreadwrite-imfsourcereader-setcurrentmediatype)
and [video processor conversion](https://learn.microsoft.com/en-us/windows/win32/medfound/mf-source-reader-enable-advanced-video-processing).

Acceptance prepared: blank format/scan/project roundtrip and migration, invalid
capture format atomicity, portrait browser bounds, actual New dropdown/Cancel/
unsaved confirmation widgets, cold startup without devices, HTTP keepalive
regression, `--ui-preset-smoke` startup/New/portrait-60p/save/load/GPU captures,
and existing real Photos/loopback Browser workflow at a portrait viewport.
All old native gates remain. Physical camera/audio, GPU reliability, 60fps
cadence, full-HD output, shutdown endurance and resource measurements remain open.

Prior exact source f6ecbdd/run 39603431352 failed Browser discovery with
`Browser page discovery timed out`, then existing wgpu queue teardown panicked.
The response-length change addresses the former. No causal GPU fix is claimed.
Local compilation/tests/native launches remain prohibited by project policy;
only formatting, repository guards, script syntax and diff review run locally.
