# UI-031 next source-dialog observations

2026-10-21. Baseline observations followed by a selected candidate; exact hosted
verification is pending. Light theme remains deprioritized. Existing UI-028/029/030
work keeps its independent acceptance ledger.

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

The current candidate implements the General form, source-header and navigation
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
