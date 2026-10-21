# UI-024 through UI-027 exact-source acceptance

2026-10-21. Source ba3b209, hosted Windows 2022
[run 39540613350](https://github.com/sansaks-jpg/rustmix/actions/runs/39540613350)
passes locked workspace check, warnings-as-errors Clippy, 261 tests across 17 suites
(137 UI, none ignored/filtered), release build, all 13 native workflows and package.
Native step 03:52:53..03:54:37 UTC, 104 s. Archive inspection complete: exact main
1917x1168/compact 1100, Streaming, MultiCorder and Video-source images reviewed.
All 13 GPU drain proofs pass and all 13 stderr files are empty. Main process 25060 ms,
peak working set 154890240 bytes, max private 382238720 bytes. This is a short hosted
sample on DX12 Microsoft Basic Render Driver CPU, driver 10.0.20348.5386, 4 logical
processors and 1x scaling; it is not a workstation performance qualification.

The production UI now anchors 16:9 pictures to monitor headers, places transport
directly below pictures, and gives the initial/reset Production layout 85% Inputs
and 15% Audio. Existing saved docking and Audio workspace are retained. Add Input
remembers its category; the quick menu opens real Video/PNG/WAV/Microphone pages
and accurately named generators. Unsupported Streaming/MultiCorder Start controls
are neutral; unsupported Save and Close is disabled with a reason, while Close
works. Earlier caption/Display corrections remain included. Light theme is
deprioritized by the user's explicit instruction.

The native workflow keeps both large reference captures, then requires observed
1600x900 restoration before close. The restoration JSON confirms 1600x900 at 1x
after the reference client capture completed; the next input process stage 0 also
reports 1600x900. Client raster JSON confirms actual 1917x1168 pixels/points. This
isolates the extra comparison capture from the existing
workflow workload; no timeout, GPU drain, readback or cadence gate was
relaxed. No local compilation, Rust test, executable build or native launch ran.

Actual final monitor geometry matches the inspected UI-025 correction: header
bottom 64, picture top 67/bottom 521.5 and transport top 524.5, so both gaps are 3 px.
The 292-wide native Audio panel fully contains Master and both group rails.
Streaming Save and Close is visibly disabled; Start is neutral. MultiCorder Start
is neutral/disabled; Close fits. Video selection via the quick menu shows the real
validated 160x90, 30/1 generated MP4. Existing motion proof records 5856 changed
channels after resume, a frozen paused frame and a different resumed frame.

Preserved failures: acb00d7/run 39538665307 and e44050b/run 39539415245 pass code
checks/build and main/reopen native, then input-stage 24 readback times out at 150 s.
Their input windows inherit 1917x1198/1917x1168 respectively, shutdown drains fail,
and wgpu Queue::drop panics (exit-1073741819). See UI-024/UI-025 reviews. Later
546578b also passes all 13 workflows before the restoration change. A passing
ba3b209 run therefore proves no causal driver/GPU recovery fix.

Short Program cadence still misses 50/59.94/60 fps:42.3933/45.8314/40.4140 fps
at 640x360, skipped 61/113/157; UI samples 16.0723/15.7945/16.9644 fps. These are
diagnostic warnings, not sustained-performance or hardware qualification. Full
UI correspondence remains open: source-page structure/initial Generators choice,
remaining typography/spacing and dialog navigation must be reviewed. Missing
capture/network/encoder capabilities, physical hardware, DPI, endurance and
intermittent hosted GPU reliability remain unqualified.
