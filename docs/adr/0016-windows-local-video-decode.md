# ADR 0016: Windows Media Foundation local video decoding

2026-10-19 · source implementation; hosted codec acceptance pending.

Use Windows' installed Media Foundation decoders through `windows = 0.62.2` (MIT OR Apache-2.0,
MSRV 1.82). Reuse this compatible locked version and its `windows::core` exports; do not select
the incompatible windows-core 0.100 generation. No FFmpeg/GStreamer runtime, codec DLL, licensed
capture SDK, browser runtime or vendor implementation is bundled.

`rustmix-platform::media_foundation` is the narrowly scoped exception anticipated by AGENTS.md:
crate-level unsafe_code is denied, with an allow only on this one audited FFI module. Core/UI
keep unsafe forbidden. Each unsafe call documents thread ownership, interface lifetimes and
pointer/buffer preconditions. Public methods expose owned safe frames and fallible errors; COM/MF
resources are RAII and the reader cannot move across threads.

Initial scope: synchronous local MP4 video, RGB32 negotiation with a CPU reference-raster resize
request, bounded native-size fallback, verified dimensions/subtype/stride/color attributes,
timestamp/sequence/discontinuity/format changes, EOF and seek. Reads/open/seek can block the owning
worker; this is not a bounded-cancellation camera or network source and cannot enter a realtime
callback/UI path. The source adapter must back off on NoSample rather than busy-spin.

Reject network/UNC/mapped-remote paths, alternate streams, unsupported formats and excessive
dimensions/buffer lengths. Copy only BGR bytes within validated row spans; do not interpret the
undefined RGB32 padding byte as alpha. Unlock buffers and release the reader before MF shutdown
and COM uninitialization on the owner thread. Do not panic across an FFI boundary.

Acceptance requires malformed-buffer/path tests plus an original generated MP4, actual decoded
pixels, monotonic PTS, seek discontinuity, bounded EOF and corrupt/unsupported-file failure on
hosted Windows. Source type integration, live frame fan-out, source lifecycle, AV synchronization,
hardware acceleration, UVC/card capture, cancellation and sustained performance remain separate
uncompleted requirements. Windows codec/patent and runtime availability require release review;
the OS integration does not establish universal MP4 codec support.
