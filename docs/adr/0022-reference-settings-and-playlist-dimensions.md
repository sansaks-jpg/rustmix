# UI-022: reference Settings and Playlist dimensions

Status: experimental; selected visual corrections and short hosted workflow accepted.
Date:2026-10-20. Follow-up to ADR0021, not a new parity or production-readiness claim.

## Evidence and decisions

Observe the supplied sibling Settings/Playlist screenshots locally; do not bundle vendor art.
Exact be79b2e screenshots corrected Overlay labels/preview containment and Playlist row hit areas,
but further dimensional differences remain (see ui021-visual-review.md).

Settings reference1148x720 on1912-wide normalizes to approximately961x603 at1600. Current760x612
is too narrow. Target.60 viewport width, explicit sidebar ratio.218 (avoid shared Properties.142
rule), approximately27px navigation rows,300px frame-rate/name fields at1600, and pinned footer.
Preserve staged changes, Cancel isolation and explicit limitations. Navigation bottom actions
should sit together where space permits. Keep all other dialog ratios intact.

Playlist measured targets at1600: available349x323, ordered452x340, transfer80x34, Close122x34,
Start135/Previous104/Next104x34 and approximately15px gap below ordered pane before flags.
Use responsive geometry and compact scrolling rather than clipping. Preserve full-width rows,
typed commands, existing identifiers, available timing/details and Program/Preview isolation.
Overlay reserved labels should be right-aligned within their column, with actual painted-text
evidence rather than trusting the reserved bounds alone.

## Reliability instrumentation

be79b2e main native reaches finalstage51 with all captures but fails GPU drain and Queue::drop.
Its slowdown begins in unopened main-workspace stage0:60frames take41.213s versus3.574s on green
f6db67c, despite the same reported DX12 CPU adapter/driver and low measured application UI CPU.
The changed form paths are not active then. This does not prove a cause or justify a backend,
frame-pacing, timeout or failure-gate change.

Add opt-in completion tokens at the existing bounded diagnostic samples using the public queue
completion callback and atomic state only. Permit at most one pending callback, so a stalled
queue cannot accumulate one per heartbeat. Report requested/observed tokens and age; callback
delivery is poll-observation timing, not hardware execution latency. No callback I/O or repaint.
Keep production behavior, shutdown Wait+Poll and all acceptance deadlines unchanged.

## Verification contract

Parallel ownership: Settings/Overlay labels in dialogs.rs; Playlist in controls.rs; diagnostics
in narrow lib.rs/smoke.rs sections. No manifest/dependency/schema change. Separate worktrees;
local formatting/diff/repository checks only. Hosted Windows must run compilation, Clippy, tests,
release build and native workflows. Inspect exact new Settings/Playlist/Overlay normal+pending
captures and their bounds against references. Strict native geometry checks should cover changed
label spacing and Playlist relations. Functional tests must click actual controls, preserve
Program/Preview and distinguish Cancel from commit. No missing hardware/capability claim can be
inferred from drawn controls or one successful GPU shutdown. Record every failed verification.

Local formatting/diff/repository/reference guards and PowerShell syntax inspection pass. An
independent combined-source review found no definite blocker; compilation/tests/native remain
unrun locally by policy. The harness requires non-null queue samples, at most one unobserved
token and at least one observed callback in every accepted smoke mode. These are diagnostic
evidence checks, not a relaxed GPU drain or a performance claim.

## Exact hosted result

7aeac9c/run39434584384 passes all hosted checks,121 UI tests plus other suites, release build,
all13 native processes/workflows and packaging. Exact Settings/Playlist/Overlay normal+pending
captures and bounds were independently reviewed: selected dimension/alignment corrections accepted.
Queue callback evidence is real and bounded;13 GPU drains pass. Cadence still misses50/59.94/60
targets and the earlier intermittent shutdown failure has no demonstrated causal resolution.
Full reference equivalence, hardware/endurance and production qualification remain open.
Measurements, remaining visual differences and trace interpretation are in ui022-visual-review.md.
