# rustmix-platform

UI-034 candidate: Media Foundation enumerates at most 256 native formats for the
selected camera without reading frames. Exact size/rate/scan/subtype are selected
again for Check/runtime; mismatches fail without a fallback camera. Processing
still returns progressive RGB32. Hosted compilation and physical hardware tests
are pending; synchronous driver blocking remains a known limitation.

Safe Windows device adapters. `audio` uses CPAL's default Windows WASAPI host and bounded
lock-free PCM rings. Devices are enumerated/opened on control threads, never automatically at
startup. The initial contract is 48 kHz f32 stereo output and mono/stereo capture; unsupported
devices return explicit errors. Device faults/underruns are counted, without callback logging,
allocation or locks. Actual device support requires physical hardware tests.

CPAL 0.18.2 is Apache-2.0; rtrb 0.4.0 is MIT OR Apache-2.0. ASIO is not enabled. No SDK/runtime
is bundled beyond the existing platform dependencies. Non-Windows calls report unsupported.
