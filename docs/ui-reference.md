# UI reference investigation

Reviewed 2026-10-17. This is workflow research, not a UI asset specification.

| Evidence | Observed workflow | RustMix decision / difference |
|---|---|---|
| [vMix Getting Started](https://www.vmix.com/help29/GettingStarted.html), browser page and visible documentation inspected | Separate left Preview and right output, explicit Cut/Fade, inputs and mixer navigation | Separate labeled Preview/Program; procedural graphics and independent Rust code |
| [vMix Audio Mixer](https://www.vmix.com/help29/Mixer.html) | Per-input controls, mute/solo, meters, bus sends | Independent typed input controls, Master/Aux A/Aux B; all meters explicitly simulated |
| [vMix feature documentation](https://www.vmix.com/software/features.aspx) | Multiple input classes, graphics, transitions and independent outputs | Only synthetic sources enabled. Unsupported capture/record/stream are disclosed; no false output status |
| [FilmCraft architecture](https://github.com/storytold/filmcraft/blob/main/docs/architecture.md), public source documentation | Headless engine, commands, rational timing, CPU reference before GPU optimization | Small three-package workspace; deterministic transition clock, isolated UI; no FilmCraft source copied |
| [REA repository](https://github.com/morluto/rea), installed skill reviewed | Observation capabilities depend on server/provider availability | No REA MCP tools advertised in this session. No proprietary binary inspected; no setup or installation performed |

The official Getting Started documentation was opened in a browser and visually inspected. Reference screenshots were not downloaded, traced, committed or shipped. Native REA GUI observation was unavailable in this session. A separate browser follow-up encountered a viewport-coordinate error; no additional observation is inferred from that attempted scroll.

On the owner's later explicit request, opened the installed vMix Trial 29.0.0.48 using Computer Use and inspected its native window. Observed orange left Preview, green right Program, a narrow central vertical transition stack, source category colors, thumbnail control rows, right-side mixer and bottom production/status strips. A source-properties click was refused after detected user input; a fresh screenshot was taken, and no dialog observation or successful click is claimed. No source switching, recording, streaming or settings mutation was performed.

RustMix UI-002 now follows this observed generic main-window arrangement. Inputs/Graphics and Audio dock below a separately resizable monitor bank. Source tiles have explicit Preview/Cut, simulated audio and properties controls. Four active lower-third slots are provided; 5–8 are disabled. The name, procedural media, rendering code and control drawings are RustMix's own work. See [ADR 0002](adr/0002-observed-broadcast-layout.md).

Unverified: exact vMix T-Bar direction/reversal, default shortcut mapping, docking persistence, source dialogs, hardware audio behavior and pixel identity. RustMix defines its own explicit semantics and tests. Its thumbnail overlay buttons use source names as lower-thirds, not full-frame video overlays. References do not prove RustMix performance or compatibility. No vMix project interoperability is claimed.

The owner prohibited cua-driver during this task; no cua-driver command was executed. Executable generation and native verification were moved to GitHub Actions at the owner's request. The public repo is [sansaks-jpg/rustmix](https://github.com/sansaks-jpg/rustmix).

## UI-004 reference review

On 2026-10-17 the supplied local files `Interface.png`, `Add Input.png`, `Input Settings.png`, and `Settings.png` in `C:\Users\WORKPLUS\Documents\WEB\rustmix-reference` were visually inspected read-only. They informed operator placement, sidebar navigation, source selection and confirmation hierarchy. No reference bitmap, logo or vendor icon was copied into the project. Current accepted hosted RustMix screenshots were also inspected as the pre-change visual baseline; they do not depict UI-003/004.

## UI-005 reference review

Re-inspected `Interface.png` and `Input Settings.png` in the same private, untracked local folder for control placement, read-only. Only generic observed layout facts were used; no vendor bitmap, wording, font or pixel was copied into the project.

| Local reference | Observed generic layout fact | RustMix UI-005 change |
|---|---|---|
| `Interface.png` | Vertical transition handle with a percentage readout in the centre column | Vertical T-Bar: handle rests at the top, pulled down to take Preview; READY / T-BAR / AUTO readout |
| `Interface.png` | Bottom strip groups each output with its own settings affordance | Disabled Record / External / Stream / MultiCorder / PlayList buttons each gained a disabled gear with its reason (`output.<id>`, `output.<id>.settings`) |
| `Interface.png` | Source tiles separate per-overlay, preview/take and audio/properties controls into rows | Tile rows: overlays 1–4 + PVW + Cut, then overlays 5–8 (disabled) + Audio + properties gear |
| `Interface.png` | Mixer labels its bus and input groups with vertical side rails | OUTPUTS / INPUTS rails with rotated labels drawn through epaint `TextShape` |
| `Interface.png` | Compact monitor-header access to the shown source's settings | Gear glyph replaces the Edit text button; automation IDs unchanged |
| `Input Settings.png` | The properties page shows the input's picture above the fields | General page shows the committed demo frame (`dialog.properties.preview`) with a "No frame yet" fallback |

Verified by hosted Windows run 39046367904, commit `a19fc03`; the workspace captures at 1600×900 and 1920×1080 were inspected against these observations. No local build was used.

## UI-006 reference review

Inspected the remaining local reference files (`Overlay Settings.png`, `Shortcut.png`, `External.png`, `Multicoder.png`, `Playlist.png`, `Recording Setup.png`, `Streaming Setup.png`) alongside `Interface.png`, `Add Input.png`, `Input Settings.png`, and `Settings.png` read-only. No reference bitmap, icon, font or proprietary text was copied.

| Local reference | Observed generic layout fact | RustMix UI-006 decision |
|---|---|---|
| `Overlay Settings.png` | Per-slot numbered overlay dialog with Type/Effect rows, placement preview and staged Apply/Cancel | Implemented bounded `Overlay Settings` window (`dialog.overlay.*`) for slots 1–4 with live 16:9 normalized preview (`overlay_panel_rect`) and staged per-slot edits; reachable from Overlays panel `⚙`, bottom production bar `⚙` beside `Overlay` (`output.overlay.settings`), `☰` menu and `Settings ▼` caret |
| `Shortcut.png` | Structured shortcut table/cards showing key combination, function and scope | Rendered 2-column broadcast shortcut keycap cards on `Settings → Shortcuts` while keeping custom remapping explicitly disclosed as unimplemented |
| `Input Settings.png` & `Interface.png` | Left-aligned sidebar categories with icons; direct per-input audio settings access from mixer strips and input properties | Left-aligned `sidebar_button` with procedural `egui::Painter` category glyphs; added `⚙` on Audio Mixer input strips (`audio.input.<id>.settings`), `"Audio Settings…"` on `Input Properties → General` (`dialog.properties.audioSettings`) and disabled `UNAVAILABLE` category disclosures |
| `Interface.png` | Toolbar dropdown carets on file/settings groups; green Master strip header; graduated T-Bar scale; Multiview presets | Added real-action `▼` menus beside `Open`, `Save As` and `Settings`; green `MASTER` strip header when unmuted; procedural T-Bar graduations; and `Multiview` presets (`Split: Monitors + Grid` vs `Full Source Grid`) |
| `External.png`, `Multicoder.png`, `Playlist.png`, `Recording Setup.png`, `Streaming Setup.png` | Dedicated modal setup dialogs for external output, ISO recording, playlist automation, disk recording and RTMP streaming | Kept `Record`, `External`, `Stream`, `MultiCorder` and `PlayList` buttons and their `⚙` buttons explicitly disabled with clear hover reasons per `AGENTS.md` §0 ("Do not implement a fake UI with nonfunctional buttons") |

