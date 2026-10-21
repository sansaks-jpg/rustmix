# UI-030 retained dialog width review

2026-10-21. Selected width workflow accepted for0382fa1/run39550459305:
check/Clippy/all271 tests (147 UI)/release,14 native processes and packaging pass.
Exact new images/JSON inspected. Earlier98bf2eb failure is retained below.
Only formatting/diff/inventory/script checks locally; no local Rust compile/tests/
build/native launch. Full UI/DPI/hardware/endurance and GPU/cadence remain open.

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

Archive inspection complete for0382fa1/run39550459305. Exact Add1600, Add1917x1168
and validated Video1600 images were directly viewed; all14 stderr files empty and
all14 GPU drains successful. Main284 frames/175057ms, process179965ms, peak working
set136511488 bytes. Independent layout process49325ms, peak155594752 bytes. It
loads the saved production, opens/cancels at1600, captures both large workspaces,
opens the widened Add form, cancels and observes actual1600x900 restoration.
The final trace can precede restoration; restoration JSON and next-mode viewport
records provide the actual size evidence. Four logical CPUs, DX12 Microsoft Basic
Render Driver10.0.20348.5386 CPU adapter,1.0 pixels/point; no physical DPI claim.

The1917x1168 raster is exact. Window[280,130]..[1635.3,990.6] is1355.3px wide;
body[287,172]..[1628.3,929.6] occupies69.97% of the viewport. Path
[548,216]..[1526,242], Browse[1530,216]..[1620,242], list[548,245]..[1620,856];
Name/check/clear share859..885 and footer stays941.6..983.6. No clipping/overlap
observed. Real headless edge-drag/default/custom-width and project-preservation
regressions pass. This accepts the selected geometry/interaction, not full parity.

Native step685s. Short640x360 Program cadence still misses50/59.94/60 targets:
28.3181/32.2662/38.1268fps, skipped183/235/187; UI1.8031/1.7898/1.7426fps.
The prior timeout and e3b12a9 drain/panic remain recorded. No causal GPU-driver fix,
stable performance, hardware, endurance or complete workflow equivalence is proved.

Reverified unchanged width geometry in56048cb/run39556368477:274 tests (150 UI),
15 native processes and packaging pass; exact final1917x1168/1600 Add and Video
images directly viewed. All15 drains/stderr/traces, raster/restoration and original
audio/video save-reopen proofs inspected. See UI-031 for final timing/cadence and
animation-settle evidence; full correspondence and GPU reliability remain open.
