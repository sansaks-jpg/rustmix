# ADR 0030: Responsive default dialog width

Date: 2026-10-21
Status: candidate; exact hosted resize/native/image evidence required.

The new e3b12a9 native comparison exposes an existing width persistence gap.
Input Select is1131px wide at1600. After resizing the parent to1917x1168 it remains
1131px wide while its body grows vertically. A fresh default at that viewport is
approximately1355px wide, corresponding to the supplied reference proportions.
egui's default_size intentionally does not override a retained resize state.

Track each original operator window's last viewport/preferred/rendered widths
in bounded per-window context preferences. Validate finite positive widths when
reading that cache. On a parent-width change, adapt width only when its
last rendered width still matched the prior default. Use public min_width and
max_width constraints for that paint; normal resize bounds resume afterward.
Scale its horizontal position proportionally within the existing constraints.
Preserve an operator's custom width and position, and retain ordinary header/body
drag behavior. Do not manipulate egui's private resize state or change engine,
project schema, persistent layout format, dependencies or assets.

Acceptance before implementation: one live dialog through1600/1917/1100/1917
parent sizes, real border drag to a custom width then parent resize, unchanged
project/Program/Preview, retained footer/chrome bounds. Existing native Input
Select capture at1917x1168 checks its intended width proportion as well as the
file row/list/footer geometry. Keep all current native and GPU drain gates.
Light theme remains deprioritized; compilation/tests/native run on hosted CI only.
