# UI-022 visual acceptance ledger

Exact-source 7aeac9c captures inspected; selected corrections and hosted workflow accepted.
Targets derive from locally observed supplied
Settings.png, Playlist.png and Overlay Settings.png; vendor files are not committed or distributed.

| Surface | Source defect | Measured target | Required proof |
|---|---|---|---|
| Settings |760x612 at1600;164px navigation,18.5px row pitch,210px fields | approximately960x600,.218 sidebar,27px rows,300px fields | settings-1600 PNG/JSON, viewport regressions and Cancel isolation |
| Playlist |358x307/462.6x307 panes,76x30 transfer,92x29 Close,3px flag gap | approximately349x323/452x340,80x34 transfer,122x34 Close,15px flag gap | playlist-1600 PNG/JSON and real transfer/reorder/row-edge interactions |
| Overlay | reserved column separates labels but paints them centered | right-aligned labels with12px control gutter | actual text geometry regression plus overlay normal/pending PNG/JSON |
| GPU | intermittent pre-dialog slowdown and teardown failure | measured completion observation with at most one callback pending | exact environment/stage traces plus strict native result; no timing/performance guarantee |

Prior UI-021 evidence:115 UI tests/all suites/check/Clippy/build passed and selected screenshots
were corrected, but native shutdown failed. The current exact-source result below supersedes
that narrow workflow baseline without erasing its intermittent failure. See ADR0022 and
ui021-visual-review.md.

## Hosted and exact visual outcome

Windows2022 run39434584384/7aeac9c passed formatting/repository guards, locked check, warnings-as-errors
Clippy, all suites including121 UI tests, release build, all13 native processes, save/reopen,
PNG/reference/output/audio/video workflows and packaging. Exact same-source Settings, Playlist,
Overlay normal and dirty-channel PNGs plus element JSON were inspected locally and independently
reviewed by the responsible agents. Selected corrections are accepted; full equivalence is not.

| Surface | Actual1600x900 measurement | Remaining difference |
|---|---|---|
| Settings |960x612 window;946x520 body;206.2px nav=.218;27px rows/28px pitch;300px rate/name fields | navigation denser than scaled reference;92x29 footer buttons smaller; color swatches on separate rows |
| Settings actions |194x27 navigation footer at705..732 inside nav219..739; OK/Cancel751..780 belowbody739 | no new import/export or output-router capability; both remain explicit |
| Playlist panes |349x323 and451.2x340 versus normalized348.878x323.077/451.539x340 | pane tops about26.5px below normalized reference; header selector remains next to New/Delete |
| Playlist controls |80x34 transfers;135/104/104/122x34 footer;15px flag gap;6px gutters | footer vertically lower due extra timing/details strip; reorder32px vs approximately26.7px normalized target |
| Playlist rows |339px hit/highlight width inside349px pane with5px inset and left captions | typography/header placement still differ; all7 demo inputs visible |
| Overlay | glyphs end at473.6 with12px field gap;794.4x28 caption fields; previews483x271.7 normal/471.1x265 pending | only title panels; advanced source/effect/alpha/transform unavailable; typography differs |

No clipping/overlap was observed for the changed1600 surfaces. Actual native input-playlist.png
also confirms full-width ordered-row highlighting. Production-size/compact pointer and geometry
regressions passed hosted; no physical-DPI or device qualification is inferred.

## Reliability and performance limits

All13 native stderr files are empty and all13 per-mode GPU drain proofs pass. Environments report
DX12 Microsoft Basic Render Driver CPU adapter, driver10.0.20348.5386,4 logical CPUs,1.0ppp.
Main20.639s, peak working set134500352bytes. Twelve bounded trace files contain167samples, no
failed samples, monotonic completion tokens and at most one unobserved token. Main ends stage51
after297frames at19.698s with tokens26/25; the final request is newer than the last recorded
completion, and the separate shutdown drain passes. The largest completed observation interval
[0,2373]ms spans35 paused UI frames between samples; it is NOT GPU execution latency.

This successful run does not establish a causal fix for intermittent be79b2e shutdown failure,
which remains part of the evidence. Cadence still fails targets:43.1605/43.1390/42.4105fps at
50/59.94/60,640x360, with55/135/141 skipped frames. No performance, endurance, physical output or
production-readiness qualification. Last fully green source is now7aeac9c; broad goal stays open.
