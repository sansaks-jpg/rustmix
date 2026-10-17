# Dependency and asset review

2026-10-17 · UI-001. Original RustMix code: MIT. No proprietary application source or artwork is used.

Direct dependencies verified through registry metadata / downloaded manifests: eframe 0.35.0 (MIT OR Apache-2.0; maintained egui project; Rust 1.92), egui_dock 0.20.1 (MIT; egui 0.35 compatible), rfd 0.17.2 (MIT; native platform file dialogs), serde/serde_json (MIT OR Apache-2.0), thiserror 2 (MIT OR Apache-2.0), tempfile 3.27 (MIT OR Apache-2.0), image 0.25.10 (MIT OR Apache-2.0; PNG-only feature used by opt-in smoke captures).

eframe includes egui font assets through epaint_default_fonts. Preserve their individual notices: Hack (MIT and Bitstream Vera terms), Noto Emoji (OFL-1.1), Ubuntu Light (Ubuntu Font Licence), emoji-icon-font (MIT). These allow font redistribution/modification under their terms and are not taken from vMix. The fonts are unmodified dependency resources. No font file is copied into RustMix assets. Do not sell font files separately or remove copyright/reserved-name notices.

No FFmpeg, GStreamer, GPL/LGPL runtime, x264/x265, NDI, ASIO, VST or capture SDK is added. Codecs, patents and device SDK licensing remain future integration reviews. Native dependencies include Windows/wgpu/winit/AccessKit FFI; crate unsafe restrictions apply to RustMix code, not audited claims about dependencies.

The portable CI artifact includes a locked dependency manifest and collected upstream license/notice texts, including font notices. This is a prototype inventory, not a formal legal audit or a signed distribution SBOM. Security advisories must be checked using cargo-audit if installed on the runner; do not claim a vulnerability scan without a result.
