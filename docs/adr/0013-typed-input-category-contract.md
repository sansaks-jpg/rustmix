# ADR 0013: typed input-category contract and the engine visibility query

2026-10-18 · reference Task 1.2 port (input category model and toolbar filter) · Implemented in
source; local headless verification passed, hosted acceptance for this commit is pending.

The reference task list models input categories as a typed value: `All` is the default
"uncategorised" encoding, six preset colours follow the catalogue, sixteen user-definable
categories clamp into `1..=16`, and `Search` is a UI-only pseudo filter that shows everything.
The reference also exposes the filter as a model query (`State::visible_inputs`) and assigns
categories through one command. RustMix already persisted per-input assignments (`Option<u8>`
definition ids over 22 editable definitions: six standard plus sixteen custom), had the
`input.category.set` command, toolbar chips, a context menu and a staged Settings editor — but it
had no typed value, no clamping contract and no engine-level filter query; the tile filter lived
inline in the UI.

Decision, implemented in `crates/engine/src/categories.rs`, `crates/engine/src/lib.rs` and
`crates/ui-egui/src/panels.rs`:

- New `InputCategory` typed value: `All`, the six preset colours in catalogue order
  (`PRESET_CATEGORIES`), `Custom(u8)` with `InputCategory::custom` clamping into `1..=16`
  (`CUSTOM_COUNT = 16`), and the `Search` pseudo filter. Typed values address definition
  *slots*; operators keep renaming or recolouring any of the 22 definitions, and the preset
  labels are default catalogue names, not a fixed vocabulary.
- `stored_id()`/`from_stored()` map the typed contract onto the persisted space without a schema
  change: presets are ids `0..=5`, custom `1..=16` are ids `6..=21`, `All`/`None` stay the
  uncategorised encoding, and ids outside the 22 definitions are rejected so a caller keeps its
  previous value instead of inventing a category.
- `ProjectState::visible_inputs(category)` reproduces the reference query semantics: `All` and
  `Search` show every input; every concrete value matches its own assignment exactly. The tile
  grid now filters through that query, so the category rule has one owner (the engine) and the
  UI keeps only the text-search matcher.
- Deliberate divergences, documented in the module and the audit ledger: `Search` is
  filter-only, never storable (the reference command can store its pseudo value); no wire
  ordinal encoding is exposed because RustMix has no control endpoint (the reference's
  inferred, unprobed ordinals are 0 = `All`, 1..=16 = custom); the reference code names the
  last two preset slots `Aqua`/`Blue`, while RustMix keeps the task specification's `Azure` and
  `MidnightBlue` labels used by its default definitions.
- Existing behavior stays intact: schema 5, the `input.category.set` command with its
  unknown-id disabled reason, migration of older files, chips/context menu/Settings editing and
  the combined category+search filter all remain additive.

Verification (local, headless): `cargo fmt --all` clean,
`cargo clippy --workspace --all-targets -- -D warnings` clean,
`cargo test --workspace` green (engine 61: 17 unit + 8 categories + 8 inputs + 7 overlays +
8 production + 2 transitions + 11 workflow; UI 47), and `scripts/check-repository.ps1` plus
`scripts/check-reference-coverage.ps1` pass. New tests cover custom clamping (0→1, 99→16),
stored-id round-trips for all 22 slots, the default definition names matching the typed preset
order, filter semantics for `All`/`Search`/presets/custom values, and the Engine-level query
against real assignments.

Verification (hosted): pending for this commit; the previous hosted run
[39129117131](https://github.com/sansaks-jpg/rustmix/actions/runs/39129117131) (commit
`f597c22`) covered the category capture and reopen on the pre-port code path.

Remaining gaps: no control endpoint, so no wire ordinals or protocol parity; no vendor identity
compatibility (`.vmix` category ids) is claimed; the text matcher remains a UI concern; nothing
in this port changes rendering, audio or Program routing.
