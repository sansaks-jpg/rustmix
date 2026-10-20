# ADR 0020: Incremental operator layout refinement

Date: 2026-10-20
Status: Source candidate; hosted verification pending

The observed reference has larger toolbar targets on a production display, compact vertical mixer
channels, and wide source dialogs. Existing RustMix screenshots show small toolbar targets and
mixer internals inheriting a horizontal parent layout, spreading controls across the channel bank.
Use these observations as workflow evidence while retaining original RustMix branding and assets.

Keep layout changes in the existing native frontend, with no dependencies, schema changes, new
backends or engine API. Toolbar targets scale with width and retain compact behavior at 1100.
Establish a vertical child inside each mixer strip. Keep stable controls/commands and add geometry
IDs for regression coverage. Improve source tiles and responsive dialog navigation; fields must
stage real data or clearly explain their unavailable state. Preserve Preview/Program isolation.

Workers own separate panels.rs and dialogs.rs worktrees; root owns lib.rs, smoke.rs and integration.
No manifest edits or conflicting file ownership. Existing uncommitted documentation is preserved.

Local checks: cargo fmt --all -- --check, scripts/check-repository.ps1, git diff --check and source
review only. New widget tests and the expanded native geometry checks are prepared, not executed.
No full reference parity, native appearance acceptance, physical device or production readiness is
claimed. Hosted Windows check/Clippy/tests/build/native smoke and screenshot inspection of this exact
patch are required. The user authorized task-scoped commits, push and hosted Windows CI on 2026-10-20.


First hosted run 39362483497 (4a70d9f): workspace check and warnings-as-errors Clippy passed;
88 UI tests passed and three failed the expanded toolbar target-height gate. The caret helper
ignored its supplied height and still allocated a 20-point menu target; it now scopes the requested
interaction height instead of discarding it. Mixer and dialog regressions passed this run.

Inherited baseline run 39284383232 passed initial PNG import but failed reopen: media completion
submitted a new producer generation after Program polling, then acceptance inspected an old black
raster. Track the presented Program generation and wait for current acknowledgment before strict
reopen state/pixel acceptance. The existing timeout still bounds the wait. A regression covers stale,
missing and current acknowledgments and verifies current invalid state still fails. This is harness
readiness, not relaxation of pixels or a new media capability. Revised hosted verification pending.


Hosted run 39362838055 (7e0d0b7) passed check, Clippy, all workspace tests and release build;
native acceptance was still running when the next geometry refinement was prepared. Transition
targets now grow with the monitor bank (26–44 logical points); compact overlay rows stay 20 high
so the manual T-Bar remains inside the bank. Resize coverage asserts take/fade target minimums
and bank containment. New geometry still needs exact-patch native evidence and precise comparison.


Precision follow-up: measured source-dialog, rail/strip and production-control differences are
tracked in ui020-visual-review.md. Light native forms now contrast with dark headers/navigation;
source dialogs grow proportionally with viewport width and height while compact footers remain
bounded. Mixer content/rails scale independently of dock width, retaining horizontal scrolling.
Production actions/setup targets grow across1100–1920. New proportions and exact-raster screenshots
must be reviewed on the exact final revision; no full visual correspondence is claimed from source.


Native run39362838055/7e0d0b7 failed the inherited caption capture proof342/692 after PNG reopen
passed. The capture validator rounded geometric pixel-center coordinates to screenshot indices,
introducing a positive half-pixel error at1.5x scaling. Independent bilinear reproduction matched
342/692 exactly; floor indexing yields692/692 without threshold changes. The revised validator
uses floor, has an independent3:2 bilinear synthetic positive/absent-caption negative regression,
and saves failed captures as -unverified.png before any validation. Canonical accepted filenames
and success JSON are produced only after checks. Revised exact-patch native acceptance pending.


Hosted run39365300339 (61f4286) passed every workflow step, including98 UI tests, all other
workspace suites, native workflows, caption692/692 and236/236, media, save/reopen and packaging.
The three cadence targets still failed (39.56/44.49/42.79fps against50/59.94/60 at640x360).
Other intermediate runs e7e714d/e175423 hit a real WARP queue-drain/destructor timeout after
functional assertions. The passing run is not proof that intermittent teardown failure is fixed.

A follow-up candidate exactly caches last uploaded thumbnail pixels per source and skips uploads
when only project/audio state changed; dynamic pixels continue to upload. It moves existing
processed buffers into a32-source384x216 cache (10,616,832bytes,10.125MiB maximum plus container
overhead). Program composition is unchanged; no lossy hash or newdependency. Actual Context
texture-delta regressions cover unchanged audio-only revisions and moving generators. Drain timeout
and fatal failure checks stay intact. New exact-patch acceptance remains required, and a passing
short smoke is not GPU hang/endurance qualification.

## Follow-up candidate after e231217 (2026-10-20)

Run 39366079519 passed workspace check, warnings-as-errors Clippy and all unit suites,
but failed native input workflow stage 10 timeout and then WARP queue-drain/destructor timeout
(exit 101). Exact thumbnail deduplication has NOT resolved the GPU reliability blocker.
Evidence is retained under artifacts/ui020-e231217; no checks or thresholds were disabled.

The next UI candidate adjusts monitor gutters/take width, fixed responsive source-card geometry,
source control/header heights and mixer headers from the observed layout measurements. Toolbar
preset targets now have stable automation bounds. Settings fills its content page, unavailable forms
paint an opaque readable surface while staying disabled, Playlist lists use separate light bodies,
Shortcuts uses a responsive bank up to nine columns, and MultiCorder columns reserve bounded widths.
The recording filename default is neutral Program.mkv rather than a personal machine path.

A separate read-only agent reviewed the combined source. Local formatting, diff and repository
layer/asset/reference guards pass. New geometry, disabled-state paint and workspace tests are
prepared for hosted Windows execution; no local compile, tests or application launch occurred.
These changes do not implement missing record/stream/capture/network backends. Fresh native captures
must be inspected before accepting their visual result. GPU reliability and cadence remain open.
