# ADR 0014: bounded external-output configuration, without a device backend

2026-10-19 · Task 1.3 continuation · Implemented in source; runtime acceptance pending.

The existing working tree contained a four-slot output model and editor. The observed reference
workflows (`rustmix-reference/vmix/FEATURE-MAP.md` §8.1 and `UI-UX.md` §12.10) justify independent
external-output configuration. They do not establish that RustMix can transmit a frame. The
reference task list and its implementation are not acceptance evidence for this repository.

Decision:

- Keep four configuration slots in the headless engine, with stable input IDs and independent
  source, rational rate, raster, device/connector, audio device/bus, alpha and audio delay.
  No dependencies, SDKs, codecs, foreign assets or runtime licences are added.
- Validate source and configuration together before committing one complete slot with one
  revision. Invalid input must preserve the project and Program/Preview. Removing an unused
  source clears its saved output references to None rather than rerouting to another input.
- Stage four independent UI drafts. Apply explicitly stores the selected output only and leaves
  the editor open so other drafts survive; Cancel discards only unapplied drafts. Preserve
  actionable validation errors without closing the editor or partially changing the route.
- Treat `port: Option<u16>` as an original RustMix hardware connector-number placeholder:
  None means the default connector; explicit numbers start at 1. It is not a TCP/UDP port or a
  claim of correspondence with a vendor SDK's connector numbering. Device enumeration and
  backend capability checks must precede future hardware activation.
- Schema 6 persists the bank. Schemas 1–5 migrate with default idle configurations. A schema-6
  document lacking its output bank is corrupt, rather than silently resetting authored routes.
  Older binaries reject schema 6. Preserve original bytes in existing save/backup handling.
- No slot becomes live, no device is opened and no audio is emitted. External Start stays
  unavailable. Configuration changes are not broadcast output or protocol compatibility.

Acceptance requires regression execution, warnings-as-errors compilation and actual hosted
native editor captures/save-reopen. Prepared tests are not passing evidence. Local formatting,
metadata and source/inventory checks follow the existing environment policy; no local executable
is built. The owner authorized commit/push to a codex branch and hosted CI on 2026-10-19;
runtime acceptance must be updated from the resulting run, never inferred from that authorization.

G01 independent qualified Program delivery, G02 sample-based audio and G05 output backends remain
open. Device enumeration, connector capabilities, color/alpha formats, resampling/delay DSP,
transport health and continuity must be designed and tested before activating real output.
