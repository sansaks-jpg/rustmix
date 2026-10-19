# rustmix-platform

Safe Windows device adapters. `audio` uses CPAL's default Windows WASAPI host and bounded
lock-free PCM rings. Devices are enumerated/opened on control threads, never automatically at
startup. The initial contract is 48 kHz f32 stereo output and mono/stereo capture; unsupported
devices return explicit errors. Device faults/underruns are counted, without callback logging,
allocation or locks. Actual device support requires physical hardware tests.

CPAL 0.18.2 is Apache-2.0; rtrb 0.4.0 is MIT OR Apache-2.0. ASIO is not enabled. No SDK/runtime
is bundled beyond the existing platform dependencies. Non-Windows calls report unsupported.
