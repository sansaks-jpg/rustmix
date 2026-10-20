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


## Intermediate native inspection: 7e0d0b7

Downloaded screenshot/JSON evidence from run39362838055 into local ignored artifacts/ui020-7e0d0b7.
Inspected workspace-1600.png and input-properties-1600.png directly against the supplied references.
Vertical mixer order is visibly corrected; the old dark form and small take targets remain visibly
different. These captures precede db29d67 and7517b9f, so they do not accept those later refinements.
Numeric extraction is saved locally as ui020-measured-geometry.json beside the screenshots.

| Control / region | Measured intermediate geometry | Review consequence |
|---|---|---|
| Settings at1920x1080 | 82x28 | Height close after client normalization; width still below observed115 |
| Fullscreen | 106x28 | Below observed150-wide group; compare current menu/button combined width |
| CUT / FADE | 110x26 /88x25 | Confirmed undersized before db29d67 transition expansion |
| Production Add /Record gear | 72x30 /26x30 | Confirmed narrow before7517b9f responsive widths |
| Master header /strip | 76x16 /82x242 | Vertical layout corrected; width below reference before7517b9f |
| PVW/PGM videos | Both847.6x476.8 | Symmetry and16:9 preserved; different viewport aspect from supplied reference |
| Add body at1600x900 | 1106x520 at247,174; buttons92x29 | Body width improved; footer controls remain smaller than observed reference |
| Properties body at1600x900 | 1146x540 at227,164; preview337.8x190 | Wider form; preview is partly clipped in default scroll view and requires review after height refinement |

Run39362838055 passed PNG native save/reopen after the generation acknowledgment fix but failed
the later caption framebuffer assertion342/692. Independent pixel-center reproduction gives342/692
with rounded coordinate indexing and692/692 with floor indexing. Acceptance brightness thresholds
remain unchanged. Missing live failure image motivated retaining unverified captures in the next
harness revision. This is a failed overall native run, not full acceptance.


## Intermediate native inspection: db29d67

Inspected workspace-1920.png and matching ui-elements.json from run39363210970 directly. CUT
now measures110x44, FADE88x44. Overlay rows were375..395 and422..442:47px pitch despite20px
buttons, because the horizontal row inherited the44px parent minimum before its closure reset it.
The next candidate resets the parent minimum before allocating either row and uses20–29px overlay
targets; resize regression caps row pitch at33. Source names also remained visually centered:
Ui::add_sized centered the Label despite its halign. An explicit left-to-right child now positions
the name and records its own bounds; regression checks the left tile inset. These corrections still
need exact-patch captures. The overall db29d67 run failed the same caption sampling342/692 assertion.


## Full changed-surface inspection: 7517b9f

Viewed all nine dialog captures with their matching JSON and workspace1600/1920 against supplied
reference images. Add Input and General are readable with light forms/dark navigation; all21 Add
choices and footers remain visible. Add body1117.2x570, navigation205.6 (18.4%); Properties body
1155.6x570, navigation164.1 (14.2%). Production Add150x36, setup50x36 and Record92x36 at1920 closely
follow observed dimensions. Mixer header124.8 wide and rails28 wide now follow the measured width.

Actionable findings: Properties preview y538.5..766.5 exceeded page bottom744 by22.5px; unavailable
record/stream/multicorder forms faded into gray-on-gray; mixer rail text was displaced outside the
blue rail; fader18x100 used only part of the adjacent14x182 meter height. The next candidate fits
preview to the remaining unscrolled page height, preserves readable opacity without enabling
unsupported controls, centers rotated rail text and gives the vertical Slider its actual height.
Add's Number selector moves beside confirmation. Regression assertions cover preview containment,
footer grouping, rail text bounds and fader height. Fresh native inspection remains required.

Remaining visual/workflow differences: Settings light card leaves a dark side/lower region; Overlay
Settings is a simplified title editor; Playlist lists remain dark with a smaller Close action;
Shortcuts is a3x4 bank rather than the observed wide bank. Mixer header is16 high versus observed21;
source pictures232x131 versus260x146, controls20 versus25; take groups110 wide versus122. These
are not accepted as equivalent, even if the functional native workflow passes. These screens need
further focused increments and matching visual evidence.

## Accepted baseline inspection: 61f4286

Windows run 39365300339 completed all steps. Inspected its workspace-1920, Properties and recording
captures plus matching bounds; all nine changed dialog captures received direct visual review.
Properties preview now ends at734 inside page bottom744; Add Number selector ends1154.2 with OK
starting1170.2. Mixer rail labels are visible and centered, faders occupy their intended height,
source names begin at the left inset, and overlays use compact row pitch. These are accepted fixes
for that exact baseline, not proof of complete visual or feature equivalence.

The subsequent e231217 run failed the native PNG workflow at stage10 and GPU shutdown, despite
passing unit suites. Its captures do not establish successful workflow acceptance.

## Next candidate and required inspection

Review new captures for: monitor/video widths and symmetric gutters, take122-wide targets on large
displays, source260x146 picture and25-high controls at1920, mixer21-high headers, bounded toolbar
presets and split menus, Settings light page extent, disabled form background contrast, Playlist
list separation/Close target, Shortcuts nine-wide large-display bank and compact wrapping, and
MultiCorder folder/duration separation. Inspect all changed screens at their exact CI revision.
The simplified Overlay Settings and missing production backends remain outstanding.
