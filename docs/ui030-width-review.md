# UI-030 retained dialog width review

2026-10-21. Source98bf2eb, run39548053929. Hosted check/Clippy/all270 tests
(146 UI)/build pass; full native/image acceptance FAILED. Formatting, diff and inventory
checks passed locally; no local Rust compile/tests/build/native launch.

The exact UI-028 e3b12a9 Input Select images demonstrate the gap: at1600 the
window is[234,130]..[1365.2,790.9]. After enlarging the parent to1917x1168, it
remains1131.2 wide and grows to990.6 bottom; a fresh default is approximately1355
wide. The recent-file form itself fits, but this retained width defeats the
requested wider reference proportions. The supplied image was observed locally
only; no vendor screenshot or artwork is committed.

Use a small per-window context preference containing finite bounded viewport,
preferred and actual widths. On a changed parent width, adapt a retained default
only when it still matches the prior preferred width (4px rounding tolerance).
Constrain width for that paint using public Window APIs, then resume normal
resizing. Scale horizontal placement within existing viewport constraints.
Preserve a real custom width. This auxiliary preference leaves project/dock
formats and engine behavior untouched. No dependency, asset or Light-theme work.

A single live dialog traverses1600/1917/1100/1917 sizes, then a real border drag
changes its width before resizing the parent to2100. Tests require default
adaptation, custom-width preservation, reachable footer/chrome and unchanged
project/Program/Preview. Existing native1917x1168 Input Select capture now also
requires its body width to be66..73% of the viewport. No original timeout,
readback/drain/cadence gate is relaxed. Exact final PNG/JSON still must be reviewed.

98bf2eb native archive inspected: main timed out after645 frames/565522ms at
stage56. It reached1917x1168 but had not yet requested that client capture or the
new Input Select capture. Only the earlier1917x1198 PNG/raster exists. Stage53
resize delivery spanned35.675s; stage56 delivery spanned36.455s; multiple earlier
dialog readbacks waited tens of seconds. These observations describe delivery,
not GPU execution time or a demonstrated driver cause. One successful GPU drain,
empty stderr, process567890ms, peak working set285442048 bytes; no later workflows
or cadence measurements ran. The1600 source form still fits, as directly viewed.

Next source keeps the complete main switcher/pause/stability/draft/dialog workflow
and moves all three mandatory large reference captures into a dedicated native
process loading its saved project. It first opens/cancels the1600 form, then checks
the1917 captures, enlarged width, Cancel and1600 restoration. State/revision checks
and all image/raster/drain/environment gates stay mandatory; no timeout is raised.
A regression keeps delayed readback stages waiting without accepting a missing
capture or an unknown stage. Exact source and full14-process acceptance pending.
