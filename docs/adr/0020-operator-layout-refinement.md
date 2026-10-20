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
