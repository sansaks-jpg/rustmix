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
