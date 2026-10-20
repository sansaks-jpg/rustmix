# UI-021 visual acceptance ledger

Candidate: staged overlay and Playlist workflow. Native acceptance pending.

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
