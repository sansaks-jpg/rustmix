# Dependency and asset review

2026-10-19 overlay caption review: the Program caption font in
`crates/engine/src/overlay_text.rs` is an original 5x8 bitmap table authored for this repository
(printable ASCII requesters/metadata only), so it adds no third-party font file, no font licence
obligation and no new crate, lockfile entry or asset-attribution row. A scalable font rasterizer
(such as a permissively licensed MIT/Apache font crate plus an OFL/Ubuntu-licensed font asset) is a
possible future upgrade, but it would need its own dependency, licence and lockfile review before
adoption; it is not present in this change.

2026-10-19 video review: local MP4 decode uses the operating system's Windows Media Foundation
components through the `windows` crate. RustMix does not add FFmpeg, GStreamer, libavcodec, x264/x265
or any third-party codec SDK as a runtime dependency, and no codec binary is bundled by RustMix. The
decoder is the existing audited FFI module in `crates/platform/src/media_foundation.rs`; the unsafe
allowance remains scoped to the platform crate. FFmpeg is used only as an external developer fixture
generator/oracle by `scripts/ci-video-fixture.ps1`; the pinned Gyan build is GPLv3, is never linked
or invoked by the RustMix application, and the tool and its libraries are excluded from packages and
evidence uploads. Only the generated original fixture and its provenance manifest are uploaded as evidence.
The generated media is original project test content, not third-party footage. Codec/patent and
distribution questions for the shipped Windows Media components (including which decoders a given
Windows installation provides) still require release review; this paragraph is not a legal audit.

2026-10-19 audio adapter review: CPAL 0.18.2 (Apache-2.0, Rust 1.85) and rtrb 0.4.0
(MIT OR Apache-2.0, Rust 1.38) were checked through `cargo info` and downloaded registry sources.
They fit workspace Rust 1.94. CPAL has empty default features and is Windows-targeted here;
ASIO/JACK/PipeWire/realtime elevation features are not enabled. The default Windows host is
WASAPI. No private SDK, codec runtime or driver is bundled. rtrb fixed rings carry plain stereo
frames so callbacks do not allocate or lock. This review is not a vulnerability scan or device
qualification. Preserve licence notices for these locked dependencies in CI packaging.

2026-10-17 · UI-001. Original RustMix code: MIT. No proprietary application source or artwork is used.

Direct dependencies verified through registry metadata / downloaded manifests: eframe 0.35.0 (MIT OR Apache-2.0; maintained egui project; Rust 1.92), egui_dock 0.20.1 (MIT; egui 0.35 compatible), rfd 0.17.2 (MIT; native platform file dialogs), serde/serde_json (MIT OR Apache-2.0), thiserror 2 (MIT OR Apache-2.0), tempfile 3.27 (MIT OR Apache-2.0), image 0.25.10 (MIT OR Apache-2.0; PNG-only feature used by opt-in smoke captures).

eframe includes egui font assets through epaint_default_fonts. Preserve their individual notices: Hack (MIT and Bitstream Vera terms), Noto Emoji (OFL-1.1), Ubuntu Light (Ubuntu Font Licence), emoji-icon-font (MIT). These allow font redistribution/modification under their terms and are not taken from vMix. The fonts are unmodified dependency resources. No font file is copied into RustMix assets. Do not sell font files separately or remove copyright/reserved-name notices.

No FFmpeg, GStreamer, GPL/LGPL runtime, x264/x265, NDI, ASIO, VST or capture SDK is added. Codecs, patents and device SDK licensing remain future integration reviews. Native dependencies include Windows/wgpu/winit/AccessKit FFI; crate unsafe restrictions apply to RustMix code, not audited claims about dependencies.

The portable CI artifact includes a locked dependency manifest and collected upstream license/notice texts, including font notices. This is a prototype inventory, not a formal legal audit or a signed distribution SBOM. Security advisories must be checked using cargo-audit if installed on the runner; do not claim a vulnerability scan without a result.

UI-002 optionally uses the existing Windows Segoe UI font through a runtime file read from the installed OS fonts folder. No Segoe font is copied or redistributed. On systems without it, the unmodified egui fallback font resources and notices apply.
