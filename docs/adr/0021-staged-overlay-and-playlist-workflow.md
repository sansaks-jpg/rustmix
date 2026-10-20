# UI-021: staged overlay and playlist workflow

Status: experimental; selected visual corrections verified, full native acceptance failed (2026-10-20).

## Problem and decision

The UI-020 visual review found Overlay Settings narrower than the supplied local reference,
a preview that did not represent both engine caption lines/visibility, and channel selection
silently dropping unsaved edits. Playlist placed transfers outside the two lists and provided
too little list space. Observe the supplied screenshots locally; commit no vendor artwork.

Use a responsive overlay editor near 62% viewport width with aligned editable fields and paired
capability/preview panes. Render the isolated draft through the existing engine compositor at
640x360 and cache by channel/draft. Changing a dirty channel requires Apply and switch, Discard
and switch, or Keep editing. Cancel leaves engine state untouched. Unimplemented source overlays,
transitions, alpha and transforms remain read-only with explicit reasons.

Use a Playlist near 60% viewport width, taller lists, transfer controls between lists, reorder
controls at the right, flags below the ordered list, and a right-aligned footer. Existing typed
commands and automation identifiers remain; naming/item timing remain in an expandable detail area.

## Reliability evidence

The previous exact source bec97a8/run39413208502 passed check/lint/tests/build but failed its
10-second shutdown GPU drain. This candidate does not claim a cure. Opt-in native runs now record
adapter/backend/driver, logical CPU count and DPI, plus at most128 stage samples with viewport,
UI CPU duration, Program generation/render statistics and nonblocking GPU poll status. No media
paths or project text are included. A successful blocking shutdown Wait is followed by a separate
nonblocking Poll to require QueueEmpty: pinned wgpu29 distinguishes WaitSucceeded from queue
emptiness. Timeouts and failed polls remain failures; no deadline or acceptance gate is relaxed.

## Verification

Prepared regressions exercise explicit draft decisions, Cancel/live isolation, engine caption
pixels/visibility, texture caching, invalid draft rejection, responsive bounds and Playlist
transfer/reorder/rename. The native workflow adds a real dirty-channel prompt screenshot and
checks unchanged Preview/Program/revision. Local formatting, repository/reference guards and
PowerShell syntax checks pass; local compile/test/build/native launch are prohibited. Hosted
Windows compilation, tests and native visual comparison are pending. See ui021-visual-review.md.

This is title-panel and modeled Playlist workflow work, not full vMix parity. Advanced overlay
composition, recording/streaming integrations, GPU reliability, cadence and hardware qualification
remain open. Existing source and reference gaps in ui020-visual-review.md remain applicable.

## Hosted outcome

f6db67c/run39420668981 passed all hosted checks and13 native processes, but visual review found
an unacceptable Grid label overlap. Follow-up be79b2e/run39422425429 passes check/Clippy/tests/build;
its exact screenshots verify reserved labels, contained preview, filled capability frame and
full-width Playlist selection. Main reaches finalstage51, then fails GPU drain and Queue::drop
panic with exit-1073741819. Later workflows do not run. a21d358 failed strict2px preview containment
before the frame-stroke correction; that regression stayed enabled. The final correction is
visually verified, not fully runtime accepted. Exact measurements and remaining gaps are in
ui021-visual-review.md. Local builds/tests/native runs remain unperformed by project policy.
