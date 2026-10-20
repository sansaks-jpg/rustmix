# UI-023 operator and Recording Setup comparison

2026-10-20. Accepted source for selected corrections:126291d. Hosted run:
[39452377970](https://github.com/sansaks-jpg/rustmix/actions/runs/39452377970).
Exact-source checks, all 251 tests, build and all 13 native processes pass. Current
workspace/Recording PNGs and geometry were inspected. Full UI equivalence remains open.
Earlier2f9d63c results below document the intermediate observations and corrections.

The supplied `Interface.png` and `Recording Setup.png` were inspected locally;
the Interface image was also opened and inspected in ego-browser. The official
[vMix 29 guide](https://www.vmix.com/help29/Introduction.html) was consulted for
workflow context. Reference files are observation-only and are not bundled.
Existing UI-022 native images provide the before evidence at 1600x900.

| Surface | Before | Target / change | Required evidence |
|---|---|---|---|
| Monitor titles | centered PVW/PGM-first captions | left input number/name; retain explicit PVW/PGM state | actual painted text bounds plus exact workspace PNGs |
| Take stack | left-aligned Quick Play/Cut/FTB; 10.5px effect labels | centered accessible text and 12 px effect labels | independent painted ink comparison at three viewport sizes |
| Operator treatment | rounded buttons; blue-black workspace | flat rectangular controls, charcoal workspace and dark top toolbar; original RustMix colors/icons | main captures with no clipped or overlapping controls |
| Disabled outputs | ordinary hover tooltip on disabled widget | disabled hover explanation points to setup gear | disabled controls remain unable to arm output |
| Recording | simultaneous MP4 and AVI/MKV forms; hardcoded 29.97p | 26% format rail, one selected page, Filename/Browse on separate rows, actual rational Program rate and reference raster | recording-setup-1600 PNG/JSON; selected-format pointer tests |
| Recording footer | excess blank area / configuration mixed with close flow | Start disabled; OK/Cancel outside page scroller | hosted geometry rejection tests and screenshot containment |

The isolated UI-023 source candidate was reviewed and integrated narrowly. No
engine, dependency, schema, clock or media backend changes were made. Navigation
changes local presentation only and cannot save configuration or start recording.
Full UI equivalence, remaining dialog typography/spacing, advanced workflows,
intermittent GPU failures, cadence, physical DPI/device and endurance qualification
remain open. A successful short CI run cannot close those unrelated gaps.

Local formatting, diff check and repository/reference inventory passed. No local
Rust compilation, test execution, executable generation or native launch occurred.
The final hosted result and exact screenshot measurements are recorded below.

First run 39447863807/0d04901 failed workspace compilation on the new
Context::style call (absent in pinned egui 0.35). It did not run tests, build or
native acceptance. The follow-up uses the current Ui visuals and also flattens
remaining source-card, mixer-frame and rail corners. Revised acceptance is pending.

Second run 39448348943/85e72a1 passed check/Clippy and125 UI tests; the
new painted-title alignment test failed. Label::halign still painted centered
because add_sized creates a centered child. Use an explicit left-to-right child
for the reserved title field. The8 px inset gate and tolerances stay unchanged.
Build/native capture did not run in that failure. Revised verification is pending.

Source2f9d63c in run39449101218 passes check, warnings-as-errors Clippy,
all250 tests including126 UI tests, release build, all 13 native processes and
packaging. Native acceptance takes629s versus the accepted UI-022 run's99s step.
No cause is inferred and no deadline or gate is changed. A fresh runner executes the same exact source
in [run39450879950](https://github.com/sansaks-jpg/rustmix/actions/runs/39450879950)
to check reproducibility; its result is pending. Any failed result must stay in
the final evidence ledger.

## Exact2f9d63c image and metadata review

Observed native workspace PNGs are1100x700,1600x900 and1920x1080, all1.0ppp;
the supplied Interface reference is1917x1198, so these are normalized observations,
not a same-viewport pixel equivalence claim. Titles now start at an8 px left inset,
take text is centered, flat controls/rails/source headers and charcoal surfaces are
visible. No proprietary image, icon, font or logo was imported.

The1600 Recording screenshot has a634x397 body,164.8 px navigation rail(26%),
463.2px page,316.4x22 filename field,96x26 Browse below it, and a separate footer
with92x29 OK/Cancel. One MKV page is selected;30.00p(30/1) and640x360 CPU reference
are truthful. Start stays disabled. Remaining observed defects in this image are
the4px label/value gap and narrower-than-reference filename; the1100 main capture
also shows the longest effect extending its caret past the other take rows.

The current follow-up reserves a12 px form gutter and approximately340px filename
field at1600, uses familiar field labels and bounds all four effect/caret pairs
inside the take column with ellipsis/hover for long names. Native geometry gates
and actual painted-glyph tests cover these targets. Run39451992479/9d2c863 was
explicitly cancelled before acceptance because the combined126291d correction
superseded it; it is neither a failure nor a passing milestone.

All 13 drained-GPU proofs in the accepted2f run pass and all 13 stderr files are
empty. Main native is188.499s with134115328-byte peak working set. End-stage trace
has183692ms elapsed, failed=false, current generation11/11, completion tokens28/28
and1.394 UI fps. The adapter is DX12 Microsoft Basic Render Driver CPU,
driver 10.0.20348.5386,4 logical CPUs. This is a significant hosted slowdown;
there is no causal fix claim for the older intermittent GPU failures.
Cadence remains below target:26.5641/33.6317/33.5603fps versus50/59.94/60 at 640x360,
with195/219/219 skipped frames. No physical hardware/endurance or production claim.

The same-source fresh-runner repeat39450879950 also passes every step, all 13
drains and empty stderr. Its native step lasts590s; main174.143s with132902912-byte
peak working set. Cadence30.4674/35.2110/43.0175fps still misses50/59.94/60,
with166/210/137 skipped frames and UI1.784/1.737/1.917fps in the profiler samples.
The slowdown therefore reproduces across these two hosted runs. No environment-
only or source-cause conclusion is established, and no responsiveness qualification
is inferred from their successful functional checks.

Settings, Overlay and Light-mode2f captures were also inspected for theme fallout.
Dark forms remain readable. Light mode still paints several original vector/caret
icons white on light buttons, a remaining contrast defect. Dialog title bars,
footer dimensions and some typography still differ from the supplied references.
These are concrete follow-up UI gaps; full UI equivalence stays open.

## Final126291d result and exact image review

Run 39452377970 on Windows 2022 passes formatting, repository/reference inventory,
locked check, warnings-as-errors Clippy, all 251 tests across17 suites(127 UI),
release build, all 13 native processes, save/reopen and packaging. All 13 GPU drain
proofs pass and every native stderr file is empty. No local compile/test/native run.

Inspected current1100x700,1600x900 and1920x1080 workspace PNGs, recording-setup-1600.png,
recording element JSON and ui-elements.json. Long effect text is elided at1100 and
the caret stays inside the take column; full text is retained for hover/accessibility.
Titles retain the8 px inset and take labels stay centered. At1600, the Recording
filename is339.6x22(previous316.4x22), beginning atx768.4(previous791.6); the12 px
label/value gutter is visible and independently checked from painted ink. The
634x397 body,164.8 px rail,463.2px page,96x26 Browse and92x29 footers are preserved.
The selected read-only MKV page shows actual30.00p(30/1) and640x360 reference raster;
Start cannot arm recording and project/Program/Preview remain isolated in tests.

This accepts these selected corrections. It does not establish complete vMix
appearance, matching physical DPI, full capabilities, production readiness or a
fix for earlier intermittent GPU failure. Remaining Light-theme icon contrast,
dialog title/footer sizing, typography and reference-viewport comparison are the
next concrete UI work. Recording/streaming backends remain unavailable.

Native step675 s; main200.955s with134602752-byte peak working set. The reported
software adapter/driver/4 CPU/1.0ppp match the earlier runs. Cadence28.8106/36.9233/
33.4939fps misses50/59.94/60 at 640x360 with178/192/220 skipped frames; profiler UI
1.549/1.491/1.564fps. The hosted slowdown remains unresolved and is not hidden by
the successful functional workflow. No further repeat was run after this acceptance.
