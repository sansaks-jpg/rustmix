# UI-023 Recording Setup handoff

Status: historical preparation record. The scoped candidate is now integrated in UI-023;
126291d passes the complete hosted/native workflow in run 39452377970, with selected
exact captures reviewed; the earlier2f9d63c also passed twice. The recording page,
gutter and compact effect-column corrections are accepted for this experimental slice.
See [current comparison and results](ui023-visual-review.md) for remaining full-equivalence,
contrast, performance and readiness gaps.

The remaining sections describe the original isolated handoff before integration.

Parallel worker prepared only dialogs.rs in the separate local worktree
C:/Users/WORKPLUS/Documents/WEB/rustmix-worktrees/ui023-recording, branch codex/ui023-recording,
base7aeac9c. Candidate file SHA256:
CB5D4898F0A0685EDBA2040657DC8EEE461203E42B88DB1B0FF2F1D660919F0A.
No local compile/test/build/native run occurred; formatting/diff/repository guards passed.
No worker continues in the background after this handoff.

Changes are narrow helper functions before sidebar_row, replacement recording_setup_dialog,
and a new recording_layout_tests module. Settings/Overlay code is unchanged from the base.
Review and apply the scoped diff rather than blindly replacing newer dialogs.rs revisions.

Reference issue: current Recording Setup shows MP4 and AVI/MKV sections simultaneously, even
when MKV is selected, and hardcodes29.97p despite actual Program30p. Supplied observation-only
Recording Setup.png shows approximately26% format rail and one selected-format page, vertical
filename/Browse/video/audio rows. Its dialog proportions are already near the current application.

Candidate provides working slot/format navigation, a single read-only page, actual rational Program
rate/raster, fixed footer outside scrolling and extra automation bounds. Configuration/output
remain disabled and explicitly not saved because no encoder backend exists. Generic format
labels avoid unsupported vendor-specific compatibility claims. Prepared tests drive actual
navigation and verify whole project/revision/actions/record counters unchanged, disabled fields,
truthful painted rate text and geometry at734/1100/1600/1920.

Next: inspect full diff/tests for API/lint/state issues, integrate narrowly, document ADR/feature
scope, run authorized hosted Windows CI and inspect exact recording-setup-1600 PNG/JSON against
the supplied reference. Keep Start unavailable. Do not call read-only configuration a working
recorder or confuse this candidate with accepted root source. GPU intermittent failure and
cadence remain open, independently of this UI increment.
