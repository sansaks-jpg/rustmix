# ADR 0031: Input General form proportions

Date: 2026-10-21
Status: selected experimental workflow and capture timing accepted in
56048cb/run39556368477; full qualification remains open.

Directly observed reference and1600 native images show narrow fixed175px Name,
Aspect Ratio and Category controls, left-aligned labels, a centered source caption
and fixed25px page rows. These are presentation gaps in existing working controls.

Use the established reserved right-aligned form-label helper for General, with
fields expanding inside their existing left column. Retain8px field gutters and
the two-column content structure. Align the source caption at the left, clipped
before its existing Change control with full hover text. Scale page rows to38px
at the large reference width, retaining compact independent navigation scrolling.
Keep all existing engine commands, drafts, unavailable controls and source actions.
No new backend, asset, dependency, schema, Light-theme or preview-size change.

Acceptance before implementation: painted label glyphs end at their reserved
columns, field bounds remain in the page at734/1100/1600/1917 widths, production
fields use at least30% of the full page width, source caption starts at the left
and does not meet Change, page rows scale and footer remains reachable. Real
name-field typing followed by Escape must discard the draft; typing followed by
OK changes only the selected source name with unchanged Program/Preview identity.
Inspect the exact new native1600 Properties screenshot/geometry alongside the
matching reference; full native/GPU gates remain mandatory on hosted Windows.

Exact fa5e4ba images expose remaining checkbox/colour/fit and metadata alignment
gaps. Extend the same field alignment to those controls and reserve right-aligned
metadata labels. Preserve each actual action and the unavailable/fixed states.
All273 tests pass for fa5e4ba, but main native's last screenshot exceeds its540s
budget and GPU drain/drop fails. Separate its dialog-capture batch into another
mandatory native process after loading the saved core project and a current
Program texture. Retain all existing picture, key, pause, stability, draft/cancel,
geometry, readback and GPU drain gates and existing per-process time limits.
The resulting15-process workflow has no single-session/endurance or GPU-fix claim.
