# ADR 0036 — Core production scope and CPU/GPU ownership

2026-10-21. Status: implementation and local verification in progress.

The owner requests a lightweight core, connected operator controls, and inspection
of every existing backend. This supersedes the earlier broad reference-menu scope.

## Decisions

- Keep independent Preview/Program, typed Cut/Fade/T-bar, FTB, existing caption
  overlays, project save/load/recovery, audio routing/explicit local monitoring,
  native Program monitoring, shortcuts and playlists. Add Input exposes local
  MP4, PNG, colour, camera, WAV, microphone and the procedural generator library.
- Hide unfinished recording, streaming, external transport, multicorder and
  future source/settings menus in normal operation. Retain existing saved-source
  readers and private draft code for compatibility; no new backend is implied.
- CPU owns composition and its full-HD surface. wgpu owns UI/monitor texture
  display. This is not a GPU compositor or hardware-decode qualification.
- Immutable decoded pixel identity participates in the cache key. Independent
  decoders with equal descriptors/local sequence numbers cannot share a wrong
  processed picture. Unchanged pixels/texture handles remain reusable.
- Program thread creation and worker failures return visible errors while the
  last valid frame remains available. Handoffs stay bounded to one latest item.
- Blank/static/live sources have no fabricated media duration/audio activity.
  Full HD / 30000:1001 defaults and the 640x360 monitor remain distinct.
- Untouched sessions preserve existing recovery. A dirty revision is checkpointed
  once, rather than repeatedly rotating identical backup generations while idle.

## Acceptance

Regress source capability/idle timing, independent decoded-frame identities,
static-render pixel equivalence, worker failure isolation, core menu reachability,
real command dispatch/Program isolation, atomic save/load and malformed media.
Inspect real native Windows UI and record measured CPU/memory/cadence separately
from deterministic tests. Keep all failures and limitations in the audit ledger.
GitHub builds/packages the executable without automated test execution, as
requested. Manual laptop acceptance precedes any push.
