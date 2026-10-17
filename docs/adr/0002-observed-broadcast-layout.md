# ADR 0002 — Observed broadcast workspace

2026-10-17 · UI-002. The owner requested a much closer vMix-like appearance after the first original layout was implemented.

Opened the owner's installed vMix Trial 29.0.0.48 with the Computer Use plugin. Observed the main window without changing sources, recording, streaming or device settings. No cua-driver was used. No vendor screenshot, logo, icon or proprietary resource is committed or shipped.

Adopt the observed generic arrangement: compact preset/file toolbar; orange Preview and green Program headers; narrow vertical transition column; colored source categories above the lower input area; audio mixer on the right; production controls along the bottom; compact status strip. Native system Segoe UI is loaded from the installed Windows font directory when available; it is not redistributed. Bundled egui fonts remain fallback resources with their original notices.

Keep the monitor bank in a resizable top egui panel so it does not need an extra dock-tab title row. Dock Inputs and Overlay/Graphics as tabs on the lower left, Audio on the lower right. The new layout contains three dock tabs. Old four-tab layouts are rejected visibly and reset, while project engine data remains valid. eframe workspace storage key advances to workspace.v2. Program/Preview remain typed engine state.

Add an explicit atomic program.takeInput command for thumbnail Cut controls. Per-input overlay buttons populate a lower-third with the source name; they are not full-frame video input overlays. Four slots function; slots 5–8 are visibly unavailable. Quick Play cuts the continuously animated synthetic Preview. Merge, record, external, streaming, ISO recording and playlist controls are disabled with reasons.

Pause Inputs freezes procedural video presentation while the transition clock remains active. The T-Bar latches completion while the pointer is held to prevent repeated source exchange at the endpoint. No actual audio/device pause is claimed.

This is a workflow/layout adaptation implemented independently. It does not assert pixel identity, proprietary asset reproduction, complete vMix functionality or production readiness. Native screenshots of RustMix, state checks and tests must verify the resulting implementation on GitHub.
