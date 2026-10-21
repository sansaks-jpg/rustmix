# ADR 0029: Escape cancellation for operator dialogs

Date: 2026-10-21
Status: candidate; exact hosted key-event/native verification required.

Standard operator windows have real Cancel/X handling but no Escape dismissal.
Use the same existing caller cancellation paths for a plain, non-repeated Escape
press in the topmost operator window. A menu/combo popup gets the first Escape;
never close a parent dialog during that same press. Consume the handled key so
it cannot reach production shortcuts. Modifier combinations and repeats do not
dismiss dialogs. Keep child Program-window keyboard handling unchanged.

Implement this once in the original caption/window wrapper, using public egui
top-layer and popup state and the existing non-repeat key consumer. Do not add
engine logic or new commit/save behavior. The caption close tooltip advertises Esc.

Acceptance before implementation: actual key events across Add, Properties,
Settings, Overlay, External, Recording, Streaming, MultiCorder, Playlist and
Shortcuts, with staged edits discarded and full project/manual-transition state
preserved; actual combo popup first-Escape behavior; only the topmost window
closes, repeat/modifier events ignored. The real native MP4 workflow dismisses
initial Video with Escape, then continues through the quick menu and existing
probe/decode/save/reopen gates. Formatting/diff/inventory only locally; all Rust
checks/tests/build/native acceptance stay on hosted Windows. No Light-theme work,
dependency/schema/backend or asset change.
