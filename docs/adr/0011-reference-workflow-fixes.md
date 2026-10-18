# ADR 0011: functional category, search, appearance and file controls

2026-10-18 · UI-013 · Implemented in source; hosted acceptance pending.

The second complete documentation review found generator-kind filters masquerading as input
categories, an inline field instead of Input Search, discarded settings selections, synchronous
snapshot encoding and a manual T-Bar hardcoded to Fade. The supplied reference's corrected
trace log takes precedence over its older summaries: 777 names are shortcut inventory, not a
proven HTTP vocabulary; wire overlays are 1–8 while state includes eight additional stingers;
Off clears an assignment, Out retains it, and a control response is pre-action state.

Use original implementations in existing crates, without new dependencies or vendor assets:

- Schema 4 stores stable per-input category assignment, six standard plus sixteen custom
  category definitions and original Dark/Light appearance preferences with Preview/Program RGB.
  Schemas 1/2/3 migrate; old backup bytes remain unchanged. Duplication retains category identity.
- Input category chips filter assignments independently of generator kind. Context menus assign
  categories; Settings stages bounded category names/colors. Input Search is an OK/Cancel dialog
  and never selects Program. Its open state blocks production shortcuts/background controls.
- `project.settings.apply` validates name/preferences together and commits one revision, preserving
  live transitions. Invalid names/preferences and revision overflow leave the old project intact.
- The first configured transition slot defines a manual T-Bar gesture; its effect/duration/start
  remain pinned until that gesture finishes or is reset. No invented aliases fill missing effects.
- Audio hide/show temporarily removes the pane and restores its complete previous docking tree;
  saves serialize that complete layout, and tab dragging is disabled while the pane is hidden.
- Snapshot joins the existing capacity-one file-job mechanism. Only a complete bounded Program
  buffer is accepted; encoding/disk IO run on a worker, files use exclusive creation, and success
  follows actual write/flush/sync completion. Missing frames and errors cannot report success.
- Unsupported overlay/output/Display controls are read-only or replaced with unavailable labels.
  A menu entry or scratch field is not an implemented media feature. The chosen lock scope is
  explicitly production keyboard mappings and tab dragging; pointer switching stays available.

The native harness adds category/search/audio visibility/theme captures and a second-process
schema-4 reopen. Worker and widget regressions cover the above behavior. No local compile/tests
or app launch are permitted; use the GitHub Windows workflow. Physical hardware, full raster/color
parity, real media/audio/outputs/replay and the unspecified eight transition algorithms remain open.
