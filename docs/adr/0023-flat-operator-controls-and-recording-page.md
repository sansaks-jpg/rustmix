# ADR 0023: Flat operator controls and selected recording page

Date: 2026-10-20
Status: experimental; selected visual corrections and exact hosted workflow accepted.

The supplied Interface screenshot places input number/name at the left monitor edge,
centers take labels and uses flat rectangular controls on a charcoal workspace.
RustMix's exact UI-022 screenshot centers monitor names, left-aligns the take labels
and uses rounded controls. Recording Setup also paints MP4 and AVI/MKV sections
simultaneously and hardcodes 29.97p although the default Program rate is 30/1.

Keep RustMix branding, existing configurable Preview/Program colors, original
procedural icons and the current command/state architecture. Use a flat native
operator treatment and explicitly align actual labels. Preserve toolbar targets,
monitor allocation, independent source clocks, Program/Preview and output safety.
No dependency, schema, encoder or hardware capability is introduced.

Review and integrate only the isolated UI-023 recording helpers, selected-format
page and tests described in ui023-recording-candidate.md. Its format/slot navigation
changes presentation only. Show one disabled page with actual rational Program rate
and 640x360 reference raster, a vertical Filename/Browse arrangement and a fixed
footer outside the page scroller. Configuration remains unavailable and unsaved;
Start cannot arm recording. Generic containers do not assert vendor codec support.

Acceptance: inspect actual painted monitor/take text at 1100x700, 1600x900 and
1920x1200. Drive recording navigation, disabled fields, Start and OK/Cancel;
require unchanged engine/project/output state. Require rail/page/footer containment
and a regression that rejects overlapping footer or horizontal Browse placement.
The existing hosted native Recording screenshot gate now checks that geometry
before accepting its PNG/JSON. Review exact main-workspace and Recording captures
against the local observation-only references; never bundle those references.

Local checks are formatting, diff review and repository/reference inventory only,
per docs/testing.md. Compilation, Clippy, all tests, release build and full native
workflow run on GitHub Windows. Continue the same task-scoped hosted-validation
authorization recorded in ADR 0020. Existing uncommitted documentation is preserved.
No full vMix equivalence, GPU reliability, cadence or production-readiness claim.

First run 39447863807/0d04901 failed compilation: egui 0.35 has no
Context::style. Read the current Ui visuals instead, consistent with the existing
frontend. No tests or native milestones passed that failed run. Flat source-card,
mixer-frame and rail corners complete the same presentation treatment without
changing control geometry. Exact revised hosted validation remains required.

Second run 39448348943/85e72a1 passed check and warnings-as-errors Clippy;
125 UI tests passed and the new painted monitor-title regression failed.
Label::halign alone does not override add_sized's centered child layout. Reserve
the title field with an explicitly left-to-right child, as existing source-card
names do. Preserve the original 8-point inset gate and all tolerances. The failed
run did not build or launch the application; exact revised acceptance is pending.

2f9d63c/run39449101218 passes every hosted check,126 UI tests plus all suites,
release build and all 13 native processes. Exact workspace and Recording PNG/JSON
were inspected. Monitor titles and take labels are corrected and recording uses
one page with bounded navigation/footer;13 GPU drains pass and stderr is empty.
The native step takes629s versus99s on UI-022; cadence targets still fail. The
fresh-runner repeat39450879950 remains pending. This is not GPU/production readiness.

Visual follow-up: Recording's label column leaves only4px of separation and a
316.4px filename field at1600. Relative to the observed page, reserve21% for the
label (86..112 px) and12 px separation, bringing the filename close to340px while
preserving the accepted page/rail/footer bounds. Use familiar Size/Frame Rate/
Bit Rate/Audio labels; values still describe actual Program and unavailable output.
Require actual painted-glyph gutter and field-width regressions at compact/large
sizes, plus fresh exact-source hosted/native screenshots. Do not relax any gate.

The exact1100 capture also reveals the long Vertical wipe label expanding its
minimum-width button and pushing the caret beyond the other take rows. Allocate
a bounded sized child for each transition effect and elide its actual accessible
text inside that width. The full label stays available on hover. Add headless
and native gates for all four effect/caret pairs inside the take column. No
transition mapping, engine command or enabled-state change. Supersede the
not-yet-accepted9d2c863 native candidate with the combined visual correction.

## Final exact-source result

Source 126291d/run 39452377970 passes all 251 tests including 127 UI, check, Clippy,
release build,13 native processes and packaging. Exact workspace and Recording
PNG/JSON were inspected: selected alignment, flat treatment, single-format page,
12 px gutter,339.6 px filename field and compact effect/caret containment are accepted.
Every GPU drain passes and stderr is empty. No local compile/test/native run.

Full UI equivalence remains open. Light icon contrast, dialog title/footer sizes,
typography and matching-viewport/DPI comparison need more work. The native step
takes675 s and the cadence/profile targets still fail on the software adapter;
earlier intermittent GPU failure has no demonstrated causal resolution. These
limitations, intermediate failures, cancelled candidate and same-source repeat
are retained in ui023-visual-review.md. No encoder/backend/readiness claim.
