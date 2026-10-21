# UI-028 Video selection and recent files review

2026-10-21. Current candidate is not yet fully verified. Earlier ba3b209 is the
accepted native baseline; all failures below remain part of this change's record.

The supplied Add Input.png was inspected directly: Video is initially selected,
a dark instruction strip sits above a path/Browse row, the recent-file list fills
the page, and Number/OK/Cancel remain at the bottom. The accepted RustMix native
Video capture instead shows several technical paragraphs and narrow fields in an
otherwise empty light page; Add Input initially selects Generators.

The candidate opens Video by default, retains subsequent category choices and
uses the real file picker/probe. It shows an aligned path/Browse row, full-width
recent list, optional source name, Check file/Clear and concise status. Source
navigation scales from 25 to 38px at the reference width. Recent entries are
validated local MP4 identities, capped at32 and kept in bounded application
preferences. They are never copied from the reference. Selection probes again;
OK needs current validated metadata. Clear affects recent preferences only and
Cancel retires draft/probe results. Existing typed commits and output isolation
remain unchanged. No dependency, project schema, platform/backend or asset added.

Prepared evidence covers malformed/oversized history, MRU cap/order/dedup/round-trip,
default Video and real field/list/footer layout at734x466,1100x700,1600x900 and
1917x1168, painted operator text, disabled premature OK, selection/Clear/Cancel
without changing the project or deleting files, and bounded UTF8 filename defaults.
Generator/PNG tests now choose those real pages. Native MP4 acceptance opens and
cancels default Video, uses the quick menu, validates a generated file, reselects
its genuine recent entry, then retains decode/pause/resume/save/reopen checks.
An additional main native Input Select capture at1917x1168 records actual raster
and geometry before Cancel and observed1600x900 restoration. No deadline/drain
gate or cadence target is relaxed. Light theme remains deprioritized.

8f97ed9/run39543984216 passed workspace check and Clippy;141 UI tests passed,
one new filename test failed. It inspected inputs.last() even though default
insertion index1 adds at the front, so it saw Countdown. Fix the test to locate
the actual video descriptor, preserving the exact UTF8/name and Program/Preview
assertions. Release build/native acceptance did not run in this failed candidate.

Local formatting, diff and repository/reference inventory checks passed, as did
PowerShell parsing of the expanded native script. Context7 was unavailable; pinned
egui0.35 source and official ScrollArea documentation were checked. No local Rust
compile, Rust tests, executable build or native launch. Exact-source hosted native
images still require inspection before accepting the candidate. Full UI/workflow,
hardware/DPI/endurance and intermittent hosted GPU reliability remain open.

e3b12a9/run39544670074 passes check/Clippy, all266 tests (142 UI), and release
build. Native acceptance FAILED after approximately904s (04:37:02..04:52:06 UTC).
Downloaded evidence shows ten native processes with successful drain proofs and
empty stderr, followed by a completed video workflow/save at stage18. Video
shutdown drain timed out and wgpu Queue::drop panicked, exit-1073741819; its process
took208846ms and peaked at553615360 working-set bytes. Video reopen/perf did not
run. Main completed at333224ms (process336748ms, peak169799680 bytes) and observed
1600x900 restoration succeeded. No causal GPU fix or current cadence qualification.

Exact main Input Select1917x1168 and1600 PNG/JSON were inspected. The file row,
recent list and all controls fit. At1600: path[461,211]..[1256,237], Browse
[1260,211]..[1350,237], list[461,240]..[1350,663], footer ends783.9. At1917 the
retained dialog remains1131.2 wide instead of the fresh1355px default, while its
body grows vertically; that is a genuine remaining fidelity gap. ADR0030 adds
default-width adaptation with a real custom-resize preservation test and a native
width-proportion gate. The actual Video-source capture contains its generated
MP4 history entry; no supplied/private reference paths were imported.
