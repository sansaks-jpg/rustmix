# rustmix-platform

UI-034 candidate: Media Foundation enumerates at most 256 native formats for the
selected camera without reading frames. Exact size/rate/scan/subtype are selected
again for Check/runtime; mismatches fail without a fallback camera. Processing
still returns progressive RGB32. Hosted compilation and physical hardware tests
are pending; synchronous driver blocking remains a known limitation.

## Media Foundation local video

`media_foundation` is the audited Windows FFI module for synchronous local MP4 decoding. It opens a
file with Media Foundation, negotiates RGB32 output, copies decoded frames to a bounded CPU raster
(maximum 4096×2160 and 64 MiB per buffer), and supports sequential read and seeking. COM and Media
Foundation objects are created, used and released on the calling thread; `VideoFile` is deliberately
`!Send` and the UI owns it on one dedicated worker thread per input, never on the UI or audio thread.
The MP4 audio track is not decoded. Decode work and copies are CPU-based and are not a zero-copy,
realtime-performance or hardware-decode qualification. Only the generated 160×90 H.264 baseline
fixture is currently accepted; codec availability depends on the installed Windows Media
components. The `unsafe` code allowance is scoped to this reviewed platform module.

## Audio

`audio` uses CPAL's default Windows WASAPI host and bounded lock-free PCM rings. Devices are enumerated/opened on control threads, never automatically at
startup. The initial contract is 48 kHz f32 stereo output and mono/stereo capture; unsupported
devices return explicit errors. Device faults/underruns are counted, without callback logging,
allocation or locks. Actual device support requires physical hardware tests.

CPAL 0.18.2 is Apache-2.0; rtrb 0.4.0 is MIT OR Apache-2.0. ASIO is not enabled. No SDK/runtime
is bundled beyond the existing platform dependencies. Non-Windows calls report unsupported.
