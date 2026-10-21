# UI-029 operator dialog keyboard review

2026-10-21. Candidate; full exact-source hosted/native qualification still required.

The shared original operator-window wrapper now uses the existing non-repeated
key consumer for plain Escape in the topmost window. It preserves the caller's
existing X/Cancel paths. An already open popup consumes the first Escape; repeats
and modifier combinations do not discard the parent. The close tooltip shows Esc.
No engine logic, schema, dependency, backend or Light-theme change.

New real-key tests cover ten standard dialogs, staged Properties/Settings/Overlay
drafts, complete project preservation and an unchanged manual transition. Add is
also exercised with a focused, edited file-path field. Other checks open a real
frame-rate combo and distinguish popup dismissal, held repeat, modifier and fresh
Escape; only the topmost of two overlapping dialogs may close. Native MP4 smoke
uses Escape on default Video then continues through the existing quick-menu,
recent-file, probe/decode and save/reopen checks. Program-child keyboard handling
is unchanged. No test deadline, readback/drain gate or frame target is relaxed.

86391d7/run39546000855 passed check/Clippy and144 UI tests, then one new repeat
test failed. Its helper released Escape in each frame, so egui correctly treated
the next key-down as a fresh press: pinned egui0.35 InputState derives repeat from
the held-key set. Keep the first key down through the repeat frame and release it
after the assertion. The production key filter stays unchanged; keep the same
popup/repeat/modifier/project assertions. Build/native acceptance did not run.

Formatting, diff and repository/reference guards passed locally. No local Rust
compile/tests/build/native launch. Full reference UI/workflow, physical keyboard,
DPI, hardware/endurance and intermittent hosted GPU reliability remain open.
