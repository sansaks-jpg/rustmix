# UI-025 production workspace review

2026-10-21. Source e44050b, hosted run 39539415245. Workspace check,
warnings-as-errors Clippy, all test suites and release build passed. Full native
acceptance FAILED; exact main images were downloaded and inspected. Preserve the
failure below even if a later revision passes.

Observed f0b0ea4 workspace at 1917x1198: both809x455 pictures are centered in
approximately 809x537 black surfaces, moving the transport away from the actual
picture. The default mixer occupies24% of the viewport. The supplied Interface
reference instead anchors its16:9 monitor pictures below their headers, retains
unused workspace below them, and uses approximately 15% for the initial mixer.

e44050b uses fitted-height top-aligned monitor surfaces and an85/15 initial/reset
Production dock. Saved dock topology and Audio workspace remain supported. New
real layout regressions require header/picture/transport adjacency, 16:9 picture
mapping, full Master/group-rail containment, six large-workspace source pictures
per row, Audio workspace switching and saved dock restoration without changing
Program/Preview identities. Native geometry applies the adjacency gate too.

The main native workflow keeps the1917x1198 image and adds a1917x1168 client-area
capture with actual logical/raster/scaling JSON and dock bounds. This compensates
for the approximately 30 px OS title bar in the supplied screenshot. Existing
timeouts, queue/drain requirements, frame targets and source/output proofs are
unchanged. Formatting, diff, repository/reference inventory and PowerShell syntax
checks passed locally. No local Rust compilation, Rust tests or native launch.

Full reference correspondence, physical-device qualification, hosted GPU teardown
reliability and high-rate Program cadence remain open. Light theme is outside this
follow-up, per the user's instruction.

Exact1917x1168 native image/JSON: PVW picture[32.7, 67.0]..[840.7, 521.5], header
bottom 64.0, transport[32.7, 524.5]..[840.7, 550.5], bank bottom 620.9. Both gaps
are 3 px. The native input pane is1600 wide, Audio292 wide, with Master and both
rails fully visible. Pictures remain16:9; unused bank space is below transport.
Compact1100 capture also fits. This resized/stored native dock fits five cards
on its first row; six is verified only in a fresh large Production layout. Do not
infer that every saved dock layout fits six.

Main native completed stage 57 in 209431 ms (process 215152 ms), then main/reopen GPU
drains passed. Input inherited1917x1168, its stage 24 readback remained pending,
and the150s input deadline failed at 151663 ms. Shutdown drain failed, followed by
wgpu Queue::drop panic and exit-1073741819. Native step 03:38:49..03:45:24 UTC.
Later input-reopen/PNG/reference/output/audio/video/perf acceptance did not run.
ADR0027 restores1600x900 after actual reference capture completion to isolate
the extra large workload. It does not waive any failure or claim driver recovery.

Final ba3b209/run 39540613350 passes all 261 tests (137 UI), check/Clippy/build,
all 13 native processes and packaging. Its exact main 1917x1168/compact 1100 images
and JSON were inspected: the same 3px monitor gaps and full Master/rail containment
are retained. Restoration/next-input viewport evidence confirms 1600x900. See
ui027-native-acceptance.md; the earlier failed e44050b result remains above.
