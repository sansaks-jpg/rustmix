# UI-035 — UI/UX reference review

21 October 2026. Owner request: focus on UI/UX against the local
`C:\Users\WORKPLUS\Documents\WEB\vmix-photos` reference; local Rust tests are
explicitly authorized, but building/packaging the RustMix application executable
is prohibited. Work stays on `main`. Pre-existing documentation edits are retained.

## Changes

- Streaming: Profile first, five destination tabs, aligned Destination/URL/key
  and Quality fields, Advanced and View Status navigation, bottom Start All /
  Start selected / View Status / Save and Close / Close controls. Navigation
  works without an encoder. Configuration and Start/Save remain disabled with
  reasons. The initial destination is 1, including the production menu's five
  zero-based routes; the previously inconsistent 2..4 menu mapping is corrected.
  No default real service URL, fabricated LIVE state, bitrate or drop count.
- MultiCorder: independent AVI/MKV/MP4 presentation tabs, Start at the upper
  right, bounded input table and folder controls. Original RustMix naming only;
  no proprietary recording format is claimed. All project inputs can be browsed
  instead of truncating to ten rows. Unknown output metrics show a dash and
  folders show Not configured. Initial checkboxes no longer pretend sources are
  selected for an unavailable recorder. Close stays outside the scrolling body.
- Settings: Outputs, Options, Decoders, Recording, Audio, Audio Outputs, Tally,
  Activators, Scripting, Alerts and About have topic-specific summaries or
  explicit unavailable states. Unrelated shortcut preference checkboxes are
  removed from unavailable pages. Options retains its actual staged shortcut
  setting. No backend, credential form or fake device selector is added.
- Input Settings: Position has zoom/pan, colour pages have lift/gain/saturation,
  Effects has mirror/sharpen. Reset affects only that page's staged values.
  Cancel retains the engine project. The shared source preview remains available.
- About uses the same bounded original RustMix caption/Close treatment and
  truthful product information as Settings → About.
- Recent video selection: the filename's child label no longer consumes the
  containing row's click. The existing real-click/validation/clear/cancel
  regression reproduced the failure twice and passes after this correction.

No dependency, engine command, project schema, source worker or saved layout
change. UI actions continue to dispatch existing validated commands. No copied
reference image, logo, icon, template, font or other vendor asset is committed.

## Coverage and reference boundaries

All 137 supplied JPEGs were visually inventoried in six local study sheets;
main, Display, Streaming, MultiCorder and Overlay were also inspected at full
image scale. The accompanying navigation map was read. Study sheets remain in
ignored `artifacts/ui035-reference-study/` and must never be distributed.

| Existing area | Review scope | Correspondence boundary |
|---|---|---|
| Main switcher / Basic / Multiview | Two monitors, take stack/T-bar, input bank, audio bank, production/status bars | Original RustMix controls; no copied artwork |
| New Preset / project actions | Staged format, Cancel/OK, existing save/reopen gates | Native file chooser and Windows event delivery not exercised here |
| Add Input | All 26 existing pages, including real Video/PNG/Photos/Browser/Camera/WAV/microphone and generators | Unsupported source types retain explicit unavailable pages |
| Input Settings | All nine enabled pages, processing drafts and reset isolation | PTZ/triggers/advanced hardware pages remain unavailable |
| Settings | All 17 existing pages | Summaries replace unsupported configuration; no full vendor options parity |
| Overlay | Existing engine-backed caption/PIP/alpha/effect draft editor | Advanced stinger/media rendering remains a backend gap |
| External Output | Four stored slot drafts, Apply/Cancel | Configuration is distinct from transport/hardware output |
| Recording | Existing selected-format read-only navigation and disabled Start | No recorder, encoding or recovery acceptance |
| Streaming | Five destination tabs, Advanced/status and fixed exit | Inspection only; no network session, save or encoder |
| MultiCorder | Format navigation, all-input table, unavailable folder/Start | No ISO recording backend |
| Playlist / Shortcuts / Search | Existing real selection, keyboard mapping and staged search | MIDI/surface/controller integration remains absent |
| Audio I/O | Current device/monitoring form, actual state labels | Devices are never started by this review |
| About / Alerts | Current product/session information | No external diagnostics/notification service |

Reference-only companion applications (social, title designer, video tools,
converter), remote web controller/tally/LiveLAN/telestrator, Data Sources and
Triggers managers are not existing RustMix features. They are outside this UI
pass; drawing nonfunctional substitutes would misstate support. Settings
summaries deliberately differ from full vendor configuration forms. This is
workflow alignment of the existing application, not full feature or pixel parity.

## Verification method

`ui_review` injects actual egui pointer events and inspects resulting UI and
engine state. Its optional capture helper tessellates the actual egui output,
font atlas and image textures into software PNGs with matching automation bounds.
These are **headless software captures**, not native Windows/wgpu screenshots,
HTML mockups or physical-DPI proof. Sources are paused in this isolated fixture;
configuration review must not repeatedly decode moving full-HD frames.

Set `RUSTMIX_UI_REVIEW_DIR` to an absolute ignored artifact directory to capture
24 representative views at 1100×700 and all 70 selected views at 1600×900:
26 source categories, nine enabled Input Settings pages, all Settings pages,
and the production dialogs/modes. Without the variable, the optional exporter
does no work and proves nothing about screenshots.

Test-only root-crate optimization is selected on the command line; no manifest
or release profile changes. No application build or packaging command is used.

## Verification ledger

- Initial focused run compiled and passed the Settings regression; output
  navigation spent excessive time rerendering moving sources and was stopped.
  A premature second invocation hit a locked Windows test runner at link time
  (`Permission denied`). Both stopped processes were verified gone before retry.
- The fixture now pauses sources. Initial captures exposed a test-renderer
  issue: constant egui time left window opening animation translucent. Time now
  advances per egui pass so captures show settled paint; those initial captures
  are rejected. No runtime opacity workaround was made.
- One extended-capture compilation failed because of an incorrect helper name
  (`select_add_tab`). It was corrected to existing `select_add_category`.
- Broad local attempts initially failed Windows temporary-directory/atomic-file
  operations with `PermissionDenied`. The engine code/tests were unchanged.
  Using the ignored task-specific TEMP/TMP and a sequential run subsequently
  passed all engine suites, including all preset roundtrips. This is evidence
  of the successful retry, not an asserted causal filesystem fix.
- Platform decode tests initially failed because `RUSTMIX_TEST_VIDEO` was unset.
  The existing developer-only `scripts/ci-video-fixture.ps1` generated its
  original quadrant/motion MP4 using the installed external FFmpeg tool, with
  provenance and hashes. Both decode tests then passed with that explicit path.
  No FFmpeg runtime/library is added to RustMix.
- The first full UI pass had 176 passes and one failure in recent-video row
  selection. A focused repeat reproduced it. Making the child filename label
  nonselectable restores the parent's actual click; the exact regression passes.
- Initial 94-view review used bundled egui fonts. Final captures use the native
  host's same optional locally installed Windows Segoe UI font policy; no font
  file is committed. The final capture pass also includes the output field
  contrast and bounded table refinements found during visual inspection.
- Final UI command below passes **177 tests, zero failures/ignored/filtered**,
  including the fixed Recent selection regression and all four `ui_review`
  gates. Duration 382.51 seconds including the complete software export.
  Final log: `artifacts/ui035-ui-final.log`.
- Core sequential run passes 121 engine tests and nine platform unit/buffer
  tests. The explicit fixture-backed decode retry passes two more. Combined
  final acceptance is **309 passing tests** (177 UI + 132 core/platform), with
  one existing opt-in release performance test not run. No new test is ignored
  or disabled, and no performance claim is made. Initial failed runs stay in
  `ui035-tests*.log`, `ui035-core-tests.log`, `ui035-ui-tests.log` and
  `ui035-history-retry.log`; they are not presented as passing commands.
- Final formatting and warnings-as-errors workspace/all-target Clippy pass
  (`ui035-clippy-final.log`). Repository/layer/asset/reference inventory and
  `git diff --check` pass. The reference guard correctly reports full reference
  match **false**; selected UI corrections do not qualify missing backends.
- Final **94 PNG/JSON pairs** inspected in eight study sheets, with full-size
  output/Settings/Input views checked separately. Dimensions are exactly
  1100×700 (24 views) / 1600×900 (70 views); all geometry is finite and every
  registered operator window is within the viewport. Changed controls/footers
  are reachable; long compact labels can still truncate with hover text.
  `artifacts/ui035-review/capture-manifest.json` records image/bounds SHA256;
  `artifacts/ui035-source-manifest.json` records the reviewed source hashes.
- `target/debug/rustmix.exe` retains its pre-session 17 October timestamp.
  Only test runners were compiled/executed. No application executable build,
  packaging, native launch, remote push, stream or device capture was performed.

Repeatable local commands (PowerShell, from the repository root):

```powershell
$env:TEMP = Join-Path (Get-Location) 'artifacts/ui035-temp'
$env:TMP = $env:TEMP
# Ensure the task-specific temporary directory exists first.
New-Item -ItemType Directory -Force -Path $env:TEMP | Out-Null
$fixtureDirectory = Join-Path (Get-Location) ('artifacts/ui035-fixture-' + [guid]::NewGuid().ToString('N'))
& ./scripts/ci-video-fixture.ps1 -OutputDirectory $fixtureDirectory
# The fixture script sets RUSTMIX_TEST_VIDEO in this PowerShell process.
cargo test -p rustmix-engine -p rustmix-platform --lib --tests --locked --no-fail-fast -- --test-threads=1
$env:RUSTMIX_UI_REVIEW_DIR = Join-Path (Get-Location) 'artifacts/ui035-review'
cargo test -p rustmix-ui --lib --locked --config 'profile.test.package.rustmix-ui.opt-level=2' -- --test-threads=2
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
pwsh -NoProfile -File scripts/check-repository.ps1
git diff --check
```

`RUSTMIX_UI_REVIEW_PAGES` optionally limits exports to comma-separated numeric
page IDs (for example `6,7,8,9` for the changed output forms). Clear it for the
complete 94-view export. No application build, release, packaging, publishing or
native launch command belongs to this procedure.

Native Windows interaction, GPU shutdown, high-DPI, physical camera/audio,
long-duration stability and complete reference correspondence remain unqualified
by this headless UI pass. Historical hosted evidence does not qualify this patch.
