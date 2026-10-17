# ADR 0005: Staged Overlay Settings, Procedural Dialog Icons, and Reference Workflow Alignment

- **Status:** Accepted
- **Date:** 2026-10-17
- **Milestone:** UI-006

## Context

Following the accepted UI-005 main-window control fidelity milestone (hosted Windows run `39046367904`), all eleven reference screenshots in the untracked local folder (`Interface.png`, `Add Input.png`, `Input Settings.png`, `Settings.png`, `Overlay Settings.png`, `External.png`, `Multicoder.png`, `Playlist.png`, `Recording Setup.png`, `Shortcut.png`, `Streaming Setup.png`) were reviewed read-only for generic broadcast workflow and layout structure:

1. **Overlay Settings (`Overlay Settings.png`):** Operators configure numbered overlay channels in a dedicated dialog with slot navigation (`Number`), type/effect disclosures, title/subtitle fields, position presets, a live proportional 16:9 placement preview, and staged Apply/Cancel semantics.
2. **Per-Input Audio Routing (`Interface.png`, `Input Settings.png`):** Operators can jump directly into an input's audio routing from its Audio Mixer strip gear button, from the `Input Properties → General` page (`Audio Settings…`), or from the source tile context menu, without overloading the tile's immediate `Audio` / `Muted` mute toggle.
3. **Dialog Sidebar Icons & Left Alignment (`Add Input.png`, `Input Settings.png`, `Settings.png`):** Centered button text in dialog sidebars collided with the left category color square on longer labels. Drawing left-aligned sidebar buttons with 16×16 category badges and original procedural vector glyphs (`egui::Painter`) improves visual hierarchy without adding external image assets.
4. **Shortcuts & Multiview Presets (`Shortcut.png`, `Interface.png`):** `Settings → Shortcuts` benefits from scannable keycap cards showing key, action, and scope, and `Multiview` benefits from switching between a split `Monitors + Grid` view and a `Full Source Grid`.
5. **Monitor Bank Height Stability:** `egui::Panel::top("monitor.bank")` persists `content_ui.min_rect().height()` at the end of each frame. Reserving `ui.set_min_height(available.y)` in `switcher` / `switcher_content` prevents frame-to-frame panel shrinkage when the vertical T-Bar leaves a few pixels of slack.

## Decision

- Keep `rustmix-engine` and project schema 1 unchanged; implement all UI-006 refinements in `crates/ui-egui`.
- Add `overlay_panel_rect` in `crates/ui-egui/src/lib.rs` so the main Program monitor and the `Overlay Settings` 16:9 preview share the exact normalized geometry of `rustmix-engine`'s CPU compositor (`56% × 9.5%`, `10.5%` vertical slot step).
- Stage `Overlay Settings` edits per slot in `RustMixApp::overlay_settings: Option<(usize, OverlayState)>` and commit via `Command::SetOverlay(slot, draft)` only on `Apply`.
- Draw all sidebar category icons, T-Bar graduations, and monitor category badges procedurally through `egui::Painter` so no external asset files are added.
- Keep `Record`, `External`, `Stream`, `MultiCorder`, and `PlayList` buttons and their `⚙` settings buttons explicitly disabled with clear hover reasons (`External.png`, `Multicoder.png`, `Playlist.png`, `Recording Setup.png`, `Streaming Setup.png` represent unimplemented backends and must not open fake configuration dialogs).

## Consequences

- Operators gain a complete, staged `Overlay Settings` workflow accessible from the `Overlay / Graphics` dock panel, the bottom production bar `⚙` button next to `Overlay`, the `☰` workspace menu, the `Settings ▼` toolbar caret, and source tile context menus.
- Dialog sidebars, dock tabs, T-Bar graduations, Audio Mixer strips, and Multiview presets align closely with broadcast switcher ergonomics while remaining 100% clean-room Rust code.
