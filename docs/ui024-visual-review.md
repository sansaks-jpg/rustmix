# UI-024 dialog captions and Display form review

2026-10-21. Current source: f0b0ea4. All hosted workflow steps pass in
[run 39537127873](https://github.com/sansaks-jpg/rustmix/actions/runs/39537127873),
including the complete native workflow and packaging. All nine exact dialog
images, dirty-overlay and workspace captures were downloaded and inspected.
Earlier failed candidates are retained below; a green short workflow does not
prove full UI equivalence.

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

First candidate 16f42da/run 39461892507 failed `cargo check`: the new wrapper
omitted the existing Shortcuts caller's min_size override. Tests, build and native
acceptance did not run. Preserve that small-window override in the wrapper and
repeat exact-source verification; no check or warning is disabled.

Second candidate 31626d7/run 39462227659 passed workspace check, then Clippy's
test-target compilation rejected a new contrast test passing a Vec2 to the
existing caret helper's scalar-height argument. Correct the test call to 24.0;
the production helper and warnings-as-errors policy stay unchanged. No Rust test
or native acceptance ran in this failure.

2026-10-21 continuation: 56ae940/run 39462559391 passed check and Clippy but
failed eight Settings tests (124 UI tests passed). Each failure was egui's
`add_space makes no sense in a grid layout` assertion in the new Display details.
Replace both direct grid spacer rows with allocated empty cells; horizontal
sub-layout spacing stays unchanged. Existing compact, staged Apply/Cancel and
painted-layout regressions exercise the exact failing states. Build/native
acceptance did not run; the subsequent exact-source result is still required.

327ed47/run 39536589420 passed check/Clippy and 131 UI tests; the remaining
painted-layout test rejects Preview/Output colour controls on different vertical
centres. Reserve the complete colour row height before laying out either swatch,
so the intervening taller label cannot shift only the second control. Keep the
one-point alignment gate; add the actual rectangles to its failure message.

The 1917x1198 capture matches the supplied image raster, not its client area:
Interface.png includes an approximately 30 px Windows title bar, whereas the GPU
capture contains only the application client. A strict content comparison must
account for that offset/use a1917x1168 client capture. Neither raster equality nor
the short hosted tests proves full visual equivalence.

The exact f0b0ea4 run passes 256 tests across 17 suites, including132 UI tests,
with no ignored or filtered tests, plus locked workspace check, warnings-as-errors
Clippy, release build and the complete native workflow/package. The native step
took99s (03:13:38..03:15:17 UTC), compared with 675 s in the accepted UI-023 run;
that observation establishes no causal performance fix. Its short Program
samples still miss50/59.94/60 fps:39.7989/46.7101/35.4245 fps at 640x360, with
82/106/197 skipped frames. UI sample rates are 15.294/14.910/14.621 fps. These are
visible diagnostic warnings, not a production or endurance qualification.

The downloaded f0b0ea4 Settings/Recording/Overlay/Add/Properties/Streaming/
MultiCorder/Playlist/Shortcuts screenshots, dirty overlay, Light theme and External
capture were inspected. Captions and fixed footers fit; the paired colours align.
The Light External capture exposes an additional defect: Cancel's default dark
text is nearly invisible on its explicit charcoal fill. Give literal charcoal
buttons an explicit white label and regress real Light-theme footer rendering
plus cancel safety across all nine affected dialogs. This follow-up needs a new
exact-source run; f0b0ea4's green result cannot qualify it in advance.

acb00d7/run 39538665307 passed check, Clippy, tests and release build, then failed
native acceptance with process exit -1073741819 after approximately 462 s in the
native step. Downloaded evidence shows main/reopen succeeded with drained GPU
queues; the input process inherited1917x1198, timed out during stage 24 readback
at 150 s, then failed shutdown drain and panicked in wgpu Queue::drop. Preserve
the original stderr/stage/memory evidence. This is not a successful exact-source
UI acceptance. The user explicitly deprioritized Light theme on 2026-10-21; do not
continue Light refinement. ADR0025/0026 focus the next changes on main monitor
proportions, source navigation and unambiguous unavailable controls.
