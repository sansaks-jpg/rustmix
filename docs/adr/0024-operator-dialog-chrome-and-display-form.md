# ADR 0024: Operator dialog chrome and Display form

Date: 2026-10-20
Status: implementation candidate; exact hosted and visual verification required.

The supplied Settings/Recording/Input screenshots use a small left title on a
light caption bar, a right close target and larger bottom confirmation controls.
RustMix UI-023 uses egui's centered 18-point title and 92x29 footers. Its Display
form also left-aligns labels and separates Preview/Output colours onto two rows.
These are visible differences from the requested operator workflow.

Use an original RustMix caption component around the existing egui Window. Keep
its stable window ID, constraint/resize/persistence behaviour and accessible
title. Hide egui's default caption; drag only the custom header using public
response/current-position APIs. Body dragging must not move the dialog. Closing
the header must follow existing Cancel semantics and leave engine state untouched.
The header has an original procedural R badge, never vendor artwork.

Target a 26-point header at 1600, growing to 32 at 1920, with 12..14-point left
title text. Confirmation buttons normalize the observed 140x42 OK and 186x42
Cancel targets to the viewport, retaining compact usable minima. Preserve fixed
footers outside page scrollers. The Display form should use right-aligned labels,
paired colour controls, staged canvas/rate settings and explicit actual-raster
feedback. Existing typed Apply/Cancel and Program/Preview isolation stay intact.

Also correct the demonstrated Light-theme vector/caret contrast defect without
changing supported actions or introducing assets/dependencies. No engine, schema,
media/output backend or licensing integration is added.

Acceptance: real close and header/body drag events at compact/large viewports;
actual caption text/close geometry, all footer containment and target sizes;
Display control alignment plus staged Cancel/Apply; actual painted icon contrast
in both themes; exact hosted workspace/dialog screenshots and existing complete
native workflow. Keep all current failure, GPU-drain and cadence evidence/gates.
Local checks remain formatting/diff/inventory only, per docs/testing.md.
