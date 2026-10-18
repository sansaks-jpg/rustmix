# Engine audit — 2026-10-18

Source baseline: `52d3610`; repair branch: `fix/ui009-layout-transport`.
This audit distinguishes working prototype paths from missing media backends.

| Area | Actual implementation | Verification / remaining work |
|---|---|---|
| Switcher | Headless typed Preview/Program, Cut, Fade, Wipe, manual blend and FTB | Engine/widget tests and native workflow; no independent broadcast output scheduler |
| Source transport | Independent monotonic procedural clocks, pause/restart/loop and global source pause | Picture, progress and timecode share one source position; transition time continues while sources pause |
| Video processing | CPU-generated RGBA8 at 384×216 for input previews and 640×360 for Program; CPU transforms, grading, key approximation, sharpening and PIP | Source effects precede composition; PNG size conversion and offline-black behavior repaired; not a full color-managed/keying pipeline |
| GPU | eframe/wgpu presents generated textures | No compute/media shader compositor, hardware decoder or encoder; project 1080p setting does not change the actual demo raster |
| Input media | Seven procedural source kinds; PNG loading through the PNG-only image dependency | No MP4/MKV/video decoder, capture driver, NDI/SRT/network ingest or browser engine; unsupported source categories are disabled; generator samples do not impersonate MP4 files |
| Audio | Deterministic meter simulation and control state | No generated/decoded sample blocks, device callback, sound output, A/V sync or real DSP/mix-minus |
| Recording/stream/external | Configuration dialogs; backend absent | Removed counter-only live simulation; start paths cannot assert successful recording/transmission; no encoder/device/network evidence |
| Projects | Validated versioned atomic saves, backups and checkpoint recovery | UI-008 source image/effect settings remain ephemeral; new/load/remove clears stale image/effect IDs to prevent the wrong image appearing in another source/project |
| Performance | Measured CPU generation time and UI callback FPS | Removed fabricated minimum FPS; paused sources can reuse unchanged surfaces; no production latency/FPS/VRAM claims |
| Reliability | Input validation and controlled error states in the prototype | Real device loss, full disk while recording, hotplug, output isolation, 30-minute/8-hour endurance and signed packaging remain unqualified |

The owner's laptop reports AMD Radeon(TM) Graphics, driver `32.0.21041.1000`.
This was a read-only device inventory, **not execution/performance testing**.
Compilation and executable testing are restricted to GitHub-hosted Windows by
the project's existing environment policy. Hosted graphics screenshots do not
qualify the owner's AMD device.

No vMix binary/code/artwork was extracted or shipped. The supplied RustMix
screenshots and existing local vMix main-window reference informed generic
operator placement. There is no 100% feature or production-readiness claim.

Hosted outcomes and artifact links are recorded in `ui-progress.md` after
execution; unrun or failed checks must remain visible.

Unsupported files are rejected without changing the existing input image or creating a generator under a video filename. A PNG decode failure retains the previous image; Browse cancellation retains the previous selection. Unsupported transition types cannot silently run as a different effect.
