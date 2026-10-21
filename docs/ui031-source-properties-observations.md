# UI-031 source-dialog review

2026-10-21. Selected workflow accepted in56048cb/run39556368477: all274 tests
(150 UI), check/Clippy/release,15 native processes and packaging pass. Exact new
Properties/Add/Video PNG/geometry inspected, including fully opaque captures after
animation settling. Light theme is deprioritized; full UI/GPU/DPI/endurance remains
open. Earlier candidate failures and acceptance evidence stay below.

The supplied Add Input and Input Settings images were directly observed locally,
then compared with98bf2eb's actual1600 native captures. No supplied image, path,
icon or vendor resource is copied into this repository.

| Surface | Observed remaining gap | Proposed acceptance before implementation |
|---|---|---|
| Add navigation |12px glyphs repeat by category index modulo6; Video shows horizontal lines, Audio a window, Colour bars. Labels identify sources but the glyphs do not. | Original semantic vectors for the actual source types; readable proportional glyph areas, bounded labels and complete hover/accessibility text. Retain real category clicks, disabled unavailable backends and unchanged Program/Preview. |
| Properties navigation | Fixed25px rows at production sizes are visibly denser than the reference's approximately38px rows. Copy From sits just after the list. | Proportional production row targets with independent scrolling at compact sizes; reachable Copy From and footer, no content overlap. |
| General form | Name and Aspect Ratio fields are175px at1600 despite a roughly480px column; labels align left. Reference fields use most of their column and labels align toward them. | Right-aligned label gutter, expanding name/aspect/category controls, two nonoverlapping form columns at production widths and reachable compact controls. Real typing/category/mirror changes remain staged until OK; Cancel/Escape preserves the complete project. |
| Source header | Caption is centered and mixes operator identity with technical source descriptions. The observed header starts at the left. | Left-aligned clipped source identity with full hover detail, reserved Change area and no overlap for long Unicode names/paths. Use existing source-change validation only. |
| General preview | Preview currently has a fixed190..320 height cap, even when more space exists at larger viewports. | Measure exact same-client-size captures before choosing a new ratio; preserve aspect, source identity and reachability of source controls/footer. Do not infer physical output or decoder support from a preview. |

Future verification must include actual original vector paint/keyboard semantics,
real input-property draft edits/cancel/commit and exact native image/geometry review.
The common reference layout is a workflow guide; unavailable features must remain
honest. This observation does not implement capture/network/encoder backends or
qualify full UI correspondence, DPI, hardware, cadence or GPU reliability.

The initial candidate implements the General form, source-header and navigation
row corrections in ADR0031. It reuses the established reserved right-aligned label
helper, expands the three existing controls, clips a left-aligned source caption
before Change and scales production page rows. Existing draft/engine behavior is
unchanged. Source-type icon semantics and preview proportions remain future work.

New widget tests inspect actual painted glyph alignment, field/page/header/footer
bounds at734/1100/1600/1917, page-row heights, and real focused Ctrl+A/text followed
by Escape or OK. Native General geometry now rejects narrow fields, clipped fields,
bad gutters and overlapping Change/header. Existing native Properties captures and
the complete14-process workflow remain mandatory. Local formatting/diff/repository
guards pass; no local Rust compile/test/build/native launch. Hosted/native/image
acceptance is pending; no full visual equivalence or GPU fix is claimed.

a667e6e/run39552040031 passes formatting/inventory/locked workspace check but
FAILS warnings-as-errors Clippy: six new test lookups used unchecked HashMap
indexing. Replace them with explicit get/unwrap assertions under the existing
test-only allowance; keep indexing warnings enabled and all assertions unchanged.
Unit tests, release and native acceptance did not run for that candidate.

fa5e4ba/run39552489406 passes check/Clippy, all273 tests (149 UI) and release.
Native FAILED: main timed out atstage46 after698 frames/541587ms waiting for its
last dialog readback, then its shutdown drain timed out and Queue::drop panicked,
exit-1073741819. Process562426ms, peak working set389644288 bytes. No subsequent
dialog/layout/reopen/media/perf acceptance for this revision. Exact Properties
PNG/JSON inspected: Name/Aspect/Category are363.1px wide at519.9..883, labels end
511.9 with8px gutters, source caption begins400 and Change begins1315.5. Navigation
rows31.7px and footer fits. The photo also exposes flags/colour/fit starting left
of the new field column and metadata labels crowding its right edge.

Follow-up aligns flags/colour/fit with Name and reserves right-aligned metadata
labels, retaining real disabled/fixed semantics. Extend actual paint/bounds tests.
Move the existing nine dialog captures plus dirty-overlay cancellation into their
own mandatory native process loading the saved core workflow project. Keep the
main switch/transport/pause/stability/small/large picture checks and all screenshot,
geometry/state/drain gates. Both retain540s deadlines and600s external limits.
This is bounded verification scheduling; GPU reliability remains unproven.

Additional exact-image finding: fa5e4ba's Add1600 screenshot captures a partially
transparent opening animation despite its four-pass wait. egui can produce several
UI passes before sufficient animation time elapses. Pinned egui0.35 Area source
uses InputState.time and global_style.animation_time for fade-in. The opt-in native
harness now records stage start in that same UI clock and waits animation_time+
100ms before dialog/dirty-overlay/large-Add captures. Normal UI animation, media
clocks, deadlines, geometry and readback/drain acceptance remain unchanged. A pure
regression rejects many early passes, clock rewind and invalid times; valid elapsed
time is mandatory. This improves evidence timing, not GPU execution or reliability.

Archive inspection complete for a72c146/run39554890819:136 original ZIP members
retrieved selectively with CRC32 verification. All15 drain proofs pass,15 stderr
files empty,14 stage traces/194 samples bounded and monotonic with no failed
sample; audio/video save-reopen bytes match and actual1600 restoration and1917x1168
Add raster proofs pass. The three independent processes take103503ms/86518ms/
51487ms (core/dialog/layout), peak working sets132972544/110510080/159137792 bytes.
No causal GPU fix is established by this green run.

Exact current1600 Properties image directly viewed: Name/Aspect/Category are363.1px
wide, field starts519.9, flags start520.0, colour and Image fit start519.9. Metadata
labels end1046.1 and values start1054.1 (8px gutter); Mouse label is visibly clear
of the left field column. Source caption starts400, Change1315.5; footer748.8..783.9
fits. Navigation31.7px; preview287.2x161.5. The selected alignment/typing/cancellation
workflow is accepted, while source-icon semantics and preview proportions remain
open. All references were observed locally; no supplied artwork or screenshots
are committed.

Native step696s. Short640x360 Program samples still miss50/59.94/60 targets:
27.7511/35.9409/33.5186fps, skipped186/203/222; UI1.7785/1.7418/1.8070fps.
Full reference correspondence, physical input/DPI/devices, stable performance,
GPU recovery and endurance remain unqualified. Earlier failures stay above.

Archive inspection complete for56048cb/run39556368477. All274 tests across17
suites pass (150 UI, none ignored/filtered), locked check/Clippy/release and all15
native processes/package pass.136 original ZIP members selectively retrieved with
CRC32 checks;15 successful drains,15 empty stderr,14 bounded monotonic traces/
193 samples without failure. Audio/video save-reopen bytes match, exact1917x1168
raster and observed1600x900 restoration pass. Core/dialog/layout processes take
97476/89829/44816ms, peak working sets136499200/112009216/155287552 bytes.

Exact final Properties1600, Add1600, Add1917x1168 and validated Video1600 images
directly viewed. Dialogs are fully opaque with no observed clipping/overlap; the
new fields/metadata/flags retain the measured geometry above. Real generated MP4
history selection and validation are visible. Native capture now waits for actual
UI animation time; normal app animation and engine timing are unchanged. No local
Rust compile/test/build/native launch, no new dependency/asset or Light-theme work.

Native step approximately683s. Short640x360 Program samples still miss targets:
32.9226/35.1584/35.2870fps at50/59.94/60, skipped139/203/201; UI1.8642/1.8418/
1.8470fps. Earlier GPU failures remain unresolved causally. Full visual/workflow
correspondence, source-icon semantics, preview proportions, physical/DPI/devices
and endurance remain open. This accepts the selected operator UI changes only.
