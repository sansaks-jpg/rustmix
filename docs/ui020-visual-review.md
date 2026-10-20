# UI-020 visual review

Status: Exact-patch native captures pending. No full visual correspondence accepted.
Date: 2026-10-20. Owner: UI-020.

## Evidence and comparison method

Observed inputs: `rustmix-reference/images/Interface.png`, `Add Input.png`, `Input Settings.png`,
and `rustmix-reference/vmix/UI-UX.md`. References are inspected outside this repository; vendor
screenshots/artwork are not committed. Only original RustMix captures and numeric observations are
used for review. Compare native screenshots and the matching automation-bounds JSON from the same
CI revision. Crop the OS caption before comparing client geometry; never compare raw whole-image
pixels across different viewport aspect ratios, media contents, DPI, themes or modes. Report logical
control dimensions and normalized client/frame position separately. A geometry test does not prove
visual similarity, legibility or native event delivery.

The observed reference screenshot is approximately 1918x1198 including its 28-pixel OS caption.
The table is an observation ledger, not a cloned design specification or acceptance percentage.

| Surface | Observed reference measurements | Candidate / review requirement |
|---|---|---|
| Preset toolbar | Buttons94x30, pitch102; Fullscreen150x30; Pause120; Basic108; Settings115 | Responsive targets; inspect centering, baseline and all dropdown hit areas at 1100/1600/1920 |
| Take controls | Seven122x44, pitch52; overlay29x29 | 26–44 target heights; inspect every row, menu and manual T-Bar containment |
| Monitors | PVW809x455; PGM810x455; title30 | Independent buses and16:9 raster; compare symmetric placement and headers |
| Source cards | About261x228; header32, picture146, controls50 | Names left aligned; verify no truncation overlap, two control rows and empty-filter feedback |
| Mixer | Master content125 wide; rails28; strip225 high | Vertical channel orientation fixed; compare header/fader/meter separation, retained horizontal scrolling |
| Production bar | Height36; Add150; settings50; actions86–130 | Responsive heights; narrower original controls remain an explicit mismatch |
| Add Input | About1356x836; .707 viewport width; sidebar .184 dialog width | Wider source dialog; compare navigation readability, content, availability, footer and reachable Video |
| Input Settings | About1402x840; .731 viewport width; sidebar .142 dialog width | Wider dialog; category stages real values, generator independent, aspect ratio explicitly fixed |

## Known mismatches before precision follow-up capture

The precision follow-up changes forms to light surfaces with dark navigation/headers, scales source
dialogs by viewport ratios, widens mixer rails/content and production controls, and grows the source
preview within its bounded page. These are source changes, not visually accepted results. Sidebar
symbols remain original generic procedural icons, not vendor assets. Exact native capture review
must check contrast, footer/navigation reachability, field labels, menu state, every changed form
and numerical geometry before closing these gaps. No backend completeness follows from alignment.

## Verification history

- Run39362483497 /4a70d9f: check/Clippy passed; UI88 passed,3 failed because caret helper ignored
  supplied target height. No native screenshots accepted from that revision.
- Run39362838055 /7e0d0b7: check/Clippy/workspace tests/release build passed; native acceptance
  and screenshots were still pending when this report was prepared.
- Run39363210970 /db29d67: transition sizing revision under verification; do not reuse an earlier
  run as proof of its final appearance.
