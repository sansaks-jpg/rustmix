# rustmix-automation

Loopback control API over the headless engine: a reference-compatible XML state
document, a `GET /api/?Function=<name>&Input=<n>&Value=<v>` dispatcher, and a
minimal HTTP server. The crate contains no switcher logic of its own —
`dispatch` translates validated wire requests onto typed engine commands
(`rustmix-engine`), and the server never starts itself.

- `document` renders engine state with the observed element/attribute
  vocabulary and capitalised `True`/`False` booleans. `<version>` identifies
  *this* implementation, never the reference product. Known gap, documented in
  `document.rs`: all sixteen overlay channels render as empty self-closing
  elements because the occupied-channel content form is not evidenced anywhere
  in this repository's captured material.
- `control` accepts 25 executable function spellings (`accepted_functions()`):
  `OverlayInput1–8` and `OverlayInput<n>Off` (mapped onto `overlay.input.set`),
  `Stinger1–8` (mapped onto the stinger trigger), `PlayPause` (reads the
  engine's public per-input clock and dispatches the existing pause/play
  commands), plus the pre-existing families. Semantics follow the re-verified
  wire measurements recorded in `docs/reference-audit.json`:
  `Off` clears an overlay channel, `Out` does not (it is a declared-unsupported
  spelling that never clears), and `OverlayInput9–16` are rejected with the
  operator-readable reason "overlay channels 9-16 are Stingers 1-8; trigger
  them with Stinger1-Stinger8". An absent or unresolvable `Input` is the
  observed accepted no-op.
- `server` binds `127.0.0.1` only (default port 8088, `HTTP/1.1` GET, one
  request per connection, 8 KiB request bound) and answers 200 for the state
  document and dispatched functions, 500 for unknown functions. It is
  deliberately not started by the application; an embedder opts in. There is no
  TCP text protocol, WebSocket or authentication beyond the loopback binding —
  the reference TCP protocol was never characterised and is recorded as a
  deferred family in `docs/gaps.md`.

Verification: `cargo test -p rustmix-automation --locked` runs the api_contract
suite — 14 tests as of UI-040 (2026-10-24), re-run green in the workspace gate
recorded in `docs/testing.md`, covering the Off-clears/Out-preserves bounds,
the 9–16 stinger reason, vocabulary bounds (unknown spellings stay
`UnknownFunction`), PlayPause playing/paused/no-transport inputs and the state
document shape.
