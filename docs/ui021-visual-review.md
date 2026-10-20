# UI-021 visual acceptance ledger

Candidate: staged overlay and Playlist workflow. Follow-up visual acceptance pending.

Hosted run39420668981/f6db67c passed check, Clippy, all suites including114 UI tests, release
build and the complete native/reopen/media/output/audio/video/package workflow. Adapter evidence:
Microsoft Basic Render Driver, CPU device, Dx12, driver10.0.20348.5386,4 logical CPUs, DPI1.0.
Cadence remains below target:42.286/41.092/41.093fps for50/59.94/60 at640x360. A single successful
GPU drain does not establish that the intermittent shutdown failure is resolved.

Exact-build native screenshots revealed an unacceptable Overlay Settings label overlap despite
passing tests: the Grid counted only each child label's occupied bounds. The follow-up reserves
the label column explicitly, records label bounds and tests at least6px separation from controls.
It also fills the capability pane's formerly202px light frame across its allocated half-width.
Those corrections require a new exact-code screenshot; f6db67c Overlay visual acceptance is rejected.
Dirty-channel prompt/buttons, genuine title/subtitle preview and footer were visibly intact.

Follow-up a21d358/run39421878826 passed check/Clippy but failed the existing strict preview
containment regression: the expanded capability frame pushed the preview2px beyond the body.
The next candidate subtracts both1px frame strokes as well as its margins. The regression remains
unchanged. Run39421789232/2992781 was cancelled as superseded before acceptance.

Playlist native review also found shortened text-only row hit areas. The follow-up restores
full-width selectable rows and tests painted left alignment and pointer selection at the far
right edge of both lists, preserving Preview/Program. Fresh native visual proof is required.

Playlist screenshot visibly places transfers between lists, reorder controls right, flags below
order and footer right, with all7 input rows visible. Further dimensions remain compared below.

Locally observed references: sibling rustmix-reference/images/Overlay Settings.png and
Playlist.png. These are observation-only files and are not bundled here.

| Surface | Reference observation | Candidate | Required exact-build evidence |
|---|---|---|---|
| Overlay Settings | approximately62% viewport width, aligned upper fields, two lower panes, bottom-right actions | responsive62%; original title-panel controls and read-only capability rows | overlay-settings-1600.png and matching element JSON; inspect field/pane/footer bounds |
| Dirty overlay channel | operator-safe staged editing requirement | explicit apply/discard/keep; Cancel isolates engine | overlay-unsaved-1600.png/element JSON and native unchanged-state assertions |
| Overlay preview | preview reflects selected configuration | cached engine title/subtitle/visibility raster, other channels isolated | raster regression plus native screenshot; no source-overlay capability claim |
| Playlist | approximately60% viewport width, tall unequal lists, transfers between lists, reorder right, footer right | responsive60%; available/order ratio .436/.564; expandable naming/timing | playlist-1600.png/element JSON; inspect available/order bodies, transfers, flags and footer |

No current screenshot is accepted from source geometry alone. Windows CI must provide exact-build
captures for visual comparison. Previous bec97a8 captures remain UI-020 evidence only. Prior GPU
shutdown failure and cadence shortfalls remain unresolved until measured otherwise; a passing
visual screenshot alone does not establish production readiness.

## Playlist measurement targets

Reference image1917x1198 has a28px root caption. Available pane418x420 and ordered pane541x442;
transfer96x44, reorder32x44, Close146x44. Scaling x by1600/1917 and client y by900/1170 gives
approximately349x323,452x340,80x34,27x34 and122x34 respectively. Treat these as comparison targets,
not claims about the current application. The candidate intentionally retains equal pane heights
and92x29 Close, so those differences must stay reported even if placement checks pass.

Compare same-build PNG edges and element JSON: pane tops within1px; ordered/available width ratio
1.294 within.03; transfers entirely between panes; reorder controls outside the ordered pane;
Add/Up and Remove/Down aligned within1px. Footer order is Start, Previous, Next, Close with Close
rightmost; flags below ordered pane; no clipped controls or unintended overlaps above.5px.
