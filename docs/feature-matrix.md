# Feature matrix

Last reviewed 2026-10-17 · Owner/task UI-001. Evidence status is updated in ui-progress.md. `Experimental` means implemented in source and awaiting or limited to prototype verification. No row is production-ready. Hardware tested is none unless explicitly recorded in the hosted-run evidence.

| Feature | Status | Scope | Test evidence | Platform | Hardware tested | Known limitations / failure behavior | Last verified date | Owner/task ID |
|---|---|---|---|---|---|---|---|---|
| Preview / Program / CUT | Experimental | Typed state; explicit source exchange | Core workflow + actual egui click; hosted smoke configured | Windows x64 | None; hosted runner planned | No broadcast output clock; invalid commands rejected | Pending CI | UI-001 |
| Fade / wipe / T-Bar / black | Experimental | CPU demo composition; pinned identity | Core transition tests + hosted FADE | Windows x64 | None | Display sRGB; no custom GPU parity; transient state resets on open | Pending CI | UI-001 |
| Inputs | Experimental | Seven procedural classes, add/edit/remove/filter/offline | Source lifecycle/validation tests | Windows x64 | None | No hardware/media; selected IDs protected from removal | Pending CI | UI-001 |
| Audio controls | Experimental | Deterministic meters, gain/mute/solo, 3 sends/buses | Routing tests + native fader smoke | Windows x64 | None | Simulation only; no samples/devices/mix-minus | Pending CI | UI-001 |
| Graphics | Experimental | Four Program slots, title/subtitle and position | Overlay command + native screenshot | Windows x64 | None | Monitor graphics only; no template engine | Pending CI | UI-001 |
| Workspace | Experimental | Drag docking/floating/resizing, 2 presets, app fullscreen/multiview | Widget resize + layout roundtrip; native resize configured | Windows x64 | None | Floating panels inside app; not external video output | Pending CI | UI-001 |
| Projects | Experimental | Schema 1, validated atomic save, backups/journal/checkpoints | Save/corrupt/reopen tests | Windows x64 | None | Manual recovery; no legacy migrations/power-failure proof | Pending CI | UI-001 |
| Record / stream / capture | Not started | Disabled with explanation | UI source review | Windows x64 | None | No backend; never false REC/stream-live | 2026-10-17 | UI-001 |
| GPU video compositor / production audio | Not started | Future independent media pipeline | None | Not tested | None | egui presentation is not a custom video compositor | 2026-10-17 | UI-001 |
