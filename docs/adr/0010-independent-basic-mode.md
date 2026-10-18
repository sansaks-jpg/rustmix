# ADR 0010: independent Basic operator presentation

Date: 2026-10-18 · Task: UI-012-reference-audit · Status: implemented, hosted acceptance pending.

The complete reference audit found that RustMix's Basic button incorrectly dispatched the
Multiview action. The supplied behavioral study (`SPEC.md`, D12) and the locally inspected
Basic-mode screenshot describe a simpler operator surface with both Preview and Program,
one Quick Play control and fewer toolbars. No reference image or artwork is imported.

Basic is a UI presentation flag, independent of engine/project state and of the remembered
multiview layout. It is stored in the native UI settings as `basic-mode.v1`, defaults to
Advanced and does not change the project schema. Entering Basic while multiview is selected
shows the switcher monitors; returning to Advanced restores the previous multiview selection.

Basic retains Preview/Program, Quick Play, the input grid with overlays 1–4/GO/Cut, Add Input,
the docked audio mixer, file open/save, Settings, About and the workspace lock. It hides the
separate Cut/transition rack/FTB/T-bar, advanced production toolbar, New/Last/fullscreen/Pause
Inputs controls and the input Loop/second control row. Quick Play continues to dispatch the
validated engine Cut command. Keyboard bindings keep their existing focus/dialog/lock guards.
The remaining audio, overlay and media limitations are unchanged by presentation mode.

Widget regression covers both directions at 1100×700, 1600×900 and 1920×1080, actual Quick Play,
restoring advanced controls, multiview independence and unchanged committed project state
when toggling. The native input workflow clicks the mode control, checks the project revision
and buses, requires `input-basic.png`, then restores Advanced before save/reopen. Its bounded
deadline increases from 120 to 150 seconds for these extra capture stages; the outer process
deadline remains 270 seconds. Hosted compilation/tests and visual review are still required.
