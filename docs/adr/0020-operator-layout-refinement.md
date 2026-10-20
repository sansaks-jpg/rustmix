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
patch are required. Repository policy requires authorization before commits/push; none performed.
