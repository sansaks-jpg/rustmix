# UI reference investigation

Reviewed 2026-10-17. This is workflow research, not a UI asset specification.

| Evidence | Observed workflow | RustMix decision / difference |
|---|---|---|
| [vMix Getting Started](https://www.vmix.com/help29/GettingStarted.html), browser page and visible documentation inspected | Separate left Preview and right output, explicit Cut/Fade, inputs and mixer navigation | Separate teal Preview / red Program with text labels; original charcoal layout and original procedural graphics |
| [vMix Audio Mixer](https://www.vmix.com/help29/Mixer.html) | Per-input controls, mute/solo, meters, bus sends | Independent typed input controls, Master/Aux A/Aux B; all meters explicitly simulated |
| [vMix feature documentation](https://www.vmix.com/software/features.aspx) | Multiple input classes, graphics, transitions and independent outputs | Only synthetic sources enabled. Unsupported capture/record/stream are disclosed; no false output status |
| [FilmCraft architecture](https://github.com/storytold/filmcraft/blob/main/docs/architecture.md), public source documentation | Headless engine, commands, rational timing, CPU reference before GPU optimization | Small three-package workspace; deterministic transition clock, isolated UI; no FilmCraft source copied |
| [REA repository](https://github.com/morluto/rea), installed skill reviewed | Observation capabilities depend on server/provider availability | No REA MCP tools advertised in this session. No proprietary binary inspected; no setup or installation performed |

The official Getting Started documentation was opened in a browser and visually inspected. Reference screenshots were not downloaded, traced, committed or shipped. No local vMix executable was used. Native REA GUI observation was unavailable in this session. A separate browser follow-up encountered a viewport-coordinate error; no additional observation is inferred from that attempted scroll.

RustMix design: compact header, paired 16:9 monitors around a narrow transition column, scrollable source tiles, simulated audio strips and four editable Program graphics slots. Panels are genuinely draggable/dockable using egui_dock. Workspace presets are an original implementation choice. The current four slots deliberately differ from the larger overlay count advertised in current vMix features.

Unverified: exact vMix T-Bar direction/reversal, default shortcut mapping, docking persistence, source dialogs, hardware audio behavior and pixel proportions. RustMix defines its own explicit semantics and tests. References do not prove RustMix performance or compatibility. No vMix project interoperability is claimed.

The owner prohibited cua-driver during this task; no cua-driver command was executed. Executable generation and native verification were moved to GitHub Actions at the owner's request. The public repo is [sansaks-jpg/rustmix](https://github.com/sansaks-jpg/rustmix).
