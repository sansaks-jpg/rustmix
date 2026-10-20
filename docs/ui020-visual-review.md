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

## Overlay Settings follow-up observation

Read-only parallel review viewed the supplied Overlay Settings reference and61f4286 native capture
with matching bounds. Reference width is approximately .620 of viewport versus RustMix .474;
RustMix title/subtitle fields and420x236 preview leave unused space. Source review also found that
preview paints only the title with UI typography, omits subtitle and ignores draft.enabled; switching
channels replaces an unsaved draft. These are open UX/correctness gaps, not accepted correspondence.
Future scoped work should preserve drafts/explicit discard, show truthful title/subtitle/visibility,
and group fields with a bounded preview. Effect/alpha/free-transform/border controls must remain
unavailable until their engine paths exist. Verify Cancel/slot switching without unintended Program
mutation, long captions and resize1100/1600/1920. No overlay-dialog implementation is included here.

## Exact current code review: bec97a8 / run39413208502

Status: specific visual corrections accepted; full native workflow FAILED. Windows2022 check,
warnings-as-errors Clippy, all workspace suites including106 UI tests, and release build passed.
The main native process produced workspace1100/1600/1920, all nine dialogs, pause/stability and
smoke-ok evidence, then exited0 with a failed10s GPU drain proof. Harness correctly rejected it.
No reopen/PNG/reference/output/audio/video/perf acceptance ran after that failure. Original GPU
reliability remains open; this run does not demonstrate a fixed hang. Previous023002b/run39412738765
failed main native stage34 after741 frames and also failed GPU drain.

Viewed current workspace1100/1920 and Overlay Settings directly; two read-only agents independently
viewed all eight other dialogs with their matching JSON and supplied reference images. At1920:

| Element | Current logical size/placement | Visual finding |
|---|---|---|
| PVW / PGM picture |809.1x455.1 each; x32.8 /1078.1|16:9 with symmetric separation; observed picture dimensions closely correspond|
| CUT |122x44|Observed large-display target dimensions reached|
| T-Bar |122x36; y478..514|Inside monitor bank, visible at1100 too|
| Input picture |260x146.3|Observed large-display picture size reached|
| Input control rows |25high;1px inter-row gap|Both rows visible on first row of cards; later cards use scrolling|
| Mixer headers |124.8x21|Observed width/height reached; bank remains horizontally scrollable|
| New / Save / Last |94x30 each|Targets bounded; Open/SaveAs split groups93.3x30 incl3px gap|
| Pause / Basic |120x30 /108x30|Observed large-display target dimensions reached|

Settings page now fills its light content extent597..1173 by219..739. Playlist light bodies401x150
are separate by4px, Close92x29. Shortcuts exposes nine102.6x74 tiles at6px gaps and a second row of
three, with clear footer. MultiCorder folder/duration have12px separation and all10 rows fit.
Add nav18.4%, Number64x22 sits16px before OK92x29; Properties nav14.2%, preview347.6x195.5 ends734
inside pagebottom744. Disabled record/stream form text is now legible on an opaque nearwhite fill.

Remaining differences: Settings width47.5% versus observed~60%,17.5-high sidebar rows are too dense;
Playlist panes are shorter and transfer/reorder placement differs, Close is mid-footer; Shortcuts
needs stronger key/action hierarchy; MultiCorder field text remains centered and setup structure
is narrower/different. Properties fields/preview remain compressed, CopyFrom placement differs.
Add capture uses Generators/PNG, so it does not establish Video-page correspondence. Recording
combines multiple format sections instead of format navigation/single selected page; Streaming
Profile/channel arrangement differs and its disabled green Start looks overly active. Overlay
Settings retains the previously documented draft/preview gaps. No full parity or production
readiness claimed. No reference artwork was imported or committed.
