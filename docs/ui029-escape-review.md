# UI-029 operator dialog keyboard review

2026-10-21. Selected Escape workflow verified by82b6cec/run39546775770 and
0382fa1/run39550459305; the latter passes271 tests (147 UI),14 native processes
and packaging. Full physical keyboard/DPI, UI equivalence and GPU reliability
qualification remain open.

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

82b6cec/run39546775770 passes check/Clippy,269 tests across17 suites (145 UI,
none ignored/filtered), release build, all13 native processes and packaging.
This includes focused path editing, real held-repeat/release input, popup-first
handling, topmost-only closure and the actual native Escape/video workflow.
Native step05:00:46..05:11:39 UTC, approximately653s. Its short Program samples
still miss50/59.94/60fps:34.8641/36.1553/37.2998fps, skipped122/196/185; UI samples
1.8434/1.8089/1.8390fps at640x360. Do not claim a causal GPU fix or production
performance qualification from this green run. UI-030 separately addresses the
retained default-width gap observed in the UI-028 native comparison.
