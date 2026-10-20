# UI-022 visual acceptance ledger

Exact-source captures and hosted acceptance pending. Targets derive from locally observed supplied
Settings.png, Playlist.png and Overlay Settings.png; vendor files are not committed or distributed.

| Surface | Source defect | Measured target | Required proof |
|---|---|---|---|
| Settings |760x612 at1600;164px navigation,18.5px row pitch,210px fields | approximately960x600,.218 sidebar,27px rows,300px fields | settings-1600 PNG/JSON, viewport regressions and Cancel isolation |
| Playlist |358x307/462.6x307 panes,76x30 transfer,92x29 Close,3px flag gap | approximately349x323/452x340,80x34 transfer,122x34 Close,15px flag gap | playlist-1600 PNG/JSON and real transfer/reorder/row-edge interactions |
| Overlay | reserved column separates labels but paints them centered | right-aligned labels with12px control gutter | actual text geometry regression plus overlay normal/pending PNG/JSON |
| GPU | intermittent pre-dialog slowdown and teardown failure | measured completion observation with at most one callback pending | exact environment/stage traces plus strict native result; no timing/performance guarantee |

UI-021 result remains authoritative until this candidate is tested:115 UI tests/all suites/check/
Clippy/build pass, selected screenshots visually corrected, full native shutdown fails. See
ADR0022 and ui021-visual-review.md. No current UI-022 visual or runtime acceptance claim.
