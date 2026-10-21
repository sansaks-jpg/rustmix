# ADR 0028: Video selection form and recent files

Date: 2026-10-21
Status: candidate; exact hosted code/native/image acceptance required.

The supplied Add Input reference opens Video, with a dark instruction strip,
file path and Browse on one row, a full-width scrolling recent-file list and a
fixed Number/OK/Cancel footer. RustMix instead opens Generators and its Video page
is a narrow stack of fields and implementation details in a mostly empty form.
The user's Light-theme exclusion remains in force.

Open Video by default and retain the real category after subsequent selections.
Scale source-navigation rows from 25px compact targets to 38px at the supplied
reference width, with readable 12..14px labels and independent navigation scrolling.
Use the existing asynchronous file picker/probe and typed AddInputAt commit.
Place the path/Browse row above a full-width recent-file list, with concise
operator-facing status, optional input name and explicit Check/Clear actions.
OK is unavailable until the current path has validated; changing a path retires
its previous metadata. Recent selection probes again and never adds/takes a
source without OK. File-name defaults are bounded and editable.

Keep at most 32 validated local MP4 identities in application preferences, with
bounded JSON and path validation. Store no metadata in the project schema and
never scan folders, copy reference paths, open network URLs or delete media.
Clear only clears recent preferences. Cancel drops draft/probe state and rejects
late results; the recent list remains an operator preference. No new dependency,
engine/platform API, proprietary asset or media backend.

Acceptance is written before implementation: MRU order/cap/deduplication,
malformed/oversized/unsupported preference rejection and round-trip; initial
Video, path/Browse/list/footer containment from compact 734x466 through 1917x1168;
real row/clear/cancel clicks with unchanged project and existing file; current
validation gating and no backend jargon in the rendered form. Existing generator
and PNG tests select those actual pages explicitly. Native acceptance opens
default Video then uses the quick menu, validates the generated MP4, selects the
real recent entry, checks unchanged Program/Preview, and retains existing
decode/pause/resume/save/reopen and GPU drain gates. Inspect exact new Add/Video
captures. Local verification stays formatting/diff/inventory/script syntax only.
