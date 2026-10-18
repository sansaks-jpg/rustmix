# Reference findings mapped to RustMix

**Full audit: [all 30 areas plus the 40 input / 20 effect / 16 overlay / 9 audio contracts](reference-audit.md).**
The full-match acceptance gate currently fails; neither this comparison nor a green functional CI run means all reference features are implemented.

2026-10-18 · UI-012 · Source inspection at baseline `11a1fb3`, followed by this working-tree change.
Reference: `C:\Users\WORKPLUS\Documents\WEB\rustmix-reference\vmix`.
Behavioral reports and the playlist screenshot were inspected; no reference code/assets were copied.
This is a capability map, not a parity percentage. New features remain experimental until hosted
Windows tests, native launch, screenshot review and reopen pass for this exact change.

| Reference workflow | Current RustMix scope | UI-012 change / remaining gap |
|---|---|---|
| Preview, Program, Cut, manual T-Bar | Independent buses and explicit takes | Preserved; operator takeover stops playlist automation |
| Four configurable transition slots | Former UI-only names with shared duration | Persisted independent effects/durations; twelve actual CPU effects including Cut; 3D/Merge/stingers absent |
| Named playlists over current inputs | Former temporary UI sequence | Add/rename/delete, add/remove/reorder references, per-item hold/effect/time, selected start, loop/manual, start/stop/previous/next, offline alarm, project persistence |
| Keyboard shortcuts and command families | Former fixed keys and enable switch | Editable persisted key/action/modifier rows; conflict validation, exact modifiers, repeat/focus guards; MIDI/control surfaces/API/scripting absent |
| Basic / Advanced mode | Former Basic-to-Multiview alias | Independent persisted presentation state and reduced controls; buses/revision/multiview preservation tests and native Basic capture prepared; hosted acceptance pending |
| Input categories/search | Existing kind filters and search | Arbitrary custom color categories remain absent |
| Local media/stills | Bounded PNG files and seven procedural sources | No general video/audio decoding, playlist seek or file-duration inference |
| Device/network inputs | No backend | Webcam, capture card, desktop capture, RTSP/SRT/NDI/OMT/browser/guest inputs remain absent |
| Audio Master + A–G, DSP, mix-minus | Master/A/B simulated controls and meters | Real samples/devices/routing/DSP and larger bus graph remain absent |
| Sixteen overlay channels / key-fill | Four title panels plus input PIP/keying | Stingers, template animation, full-frame overlay channels and key/fill remain absent |
| Multiview and outputs | In-app monitor/grid layouts | Independent monitor outputs, SDI, virtual camera and network outputs remain absent |
| Record / Stream / MultiCorder | Disabled unavailable actions | Encoders, file recovery and network streaming remain absent |
| Replay and ISO | No backend | Storage, replay transport and clip export remain absent |
| Titles, data sources and virtual sets | Procedural graphics, title panels, PIP | No GT/proprietary templates, data connectors or virtual-set asset packages |
| PTZ, tally, MIDI, voice | No backend | Hardware/license/protocol-specific work remains absent |
| Presets, saved preferences and recovery | Own RustMix format | Schema 3; migrate 1/2, retain original backups; no `.vmix` compatibility claim |
| Diagnostics / production qualification | Existing measured prototype counters | New worker acknowledgment needs hosted regression and cadence review; device-loss, AV sync and endurance remain open |

The public [playlist guide](https://hwww.vmix.com/help29/playlist.html) supports per-item durations,
effects, named-project persistence, loop/manual/selected start and clearing overlays on start.
RustMix deliberately uses explicit hold times for its generators/PNGs and never pretends to seek
or decode a video file. See [ADR 0009](adr/0009-engine-owned-production-controls.md).
