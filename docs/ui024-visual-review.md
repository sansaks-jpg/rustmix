# UI-024 dialog captions and Display form review

2026-10-20. Candidate source: 16f42da. Hosted acceptance is pending in
[run 39461892507](https://github.com/sansaks-jpg/rustmix/actions/runs/39461892507).
Do not treat prepared tests, this document or a CI launch as a passing result.

The supplied Settings.png was inspected beside the accepted UI-023
settings-1600.png. The reference uses a small left caption, a light title surface,
larger confirmation controls, right-aligned Display labels and Preview/Output
colour controls on one row. RustMix retained a centered 18-point dark egui
caption, 92x29 footers and left-aligned labels on separate rows. These observed
differences motivate ADR0024; proprietary artwork remains outside the repository.

The candidate introduces one bounded caption wrapper through egui's public
Window API. Its original procedural RustMix badge, close target and header-only
drag preserve stable window IDs, viewport constraints and existing Cancel paths.
The default egui caption is hidden, rather than painted over with a screenshot.
Display uses paired colours and the existing staged Preferences canvas/rate;
Fullscreen 1 opens existing output controls. Secondary routing and unsupported
aspect/input-size configuration remain visibly unavailable. Actual Program
processing remains 640x360, regardless of the saved project canvas setting.

Prepared acceptance includes painted caption/label geometry, actual close and
header/body pointer events, staged canvas Cancel/Apply, both-theme vector contrast,
existing compact footer gates, all nine native dialog captures, and an additional
native workspace capture requested at the supplied 1917x1198 reference size.
The screenshot event checks its actual physical raster and records the observed
logical size and scaling. No deadline, GPU-drain gate or cadence target is relaxed.

Local formatting, diff, script syntax and repository/reference inventory checks
passed. No local Rust compile, Rust test, executable build or native launch was
performed. Exact Windows outcomes and rendered screenshots must be reviewed before
accepting this candidate. Full UI equivalence and hardware/performance acceptance
remain open.

First candidate 16f42da/run39461892507 failed `cargo check`: the new wrapper
omitted the existing Shortcuts caller's min_size override. Tests, build and native
acceptance did not run. Preserve that small-window override in the wrapper and
repeat exact-source verification; no check or warning is disabled.

Second candidate 31626d7/run39462227659 passed workspace check, then Clippy's
test-target compilation rejected a new contrast test passing a Vec2 to the
existing caret helper's scalar-height argument. Correct the test call to 24.0;
the production helper and warnings-as-errors policy stay unchanged. No Rust test
or native acceptance ran in this failure.

2026-10-21 continuation: 56ae940/run39462559391 passed check and Clippy but
failed eight Settings tests (124 UI tests passed). Each failure was egui's
`add_space makes no sense in a grid layout` assertion in the new Display details.
Replace both direct grid spacer rows with allocated empty cells; horizontal
sub-layout spacing stays unchanged. Existing compact, staged Apply/Cancel and
painted-layout regressions exercise the exact failing states. Build/native
acceptance did not run; the subsequent exact-source result is still required.

327ed47/run39536589420 passed check/Clippy and131 UI tests; the remaining
painted-layout test rejects Preview/Output colour controls on different vertical
centres. Reserve the complete colour row height before laying out either swatch,
so the intervening taller label cannot shift only the second control. Keep the
one-point alignment gate; add the actual rectangles to its failure message.

The1917x1198 capture matches the supplied image raster, not its client area:
Interface.png includes an approximately30px Windows title bar, whereas the GPU
capture contains only the application client. A strict content comparison must
account for that offset/use a1917x1168 client capture. Neither raster equality nor
the short hosted tests proves full visual equivalence.
