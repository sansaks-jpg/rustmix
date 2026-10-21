# UI-034: Add Input and New Preset

2026-10-21. Implementation candidate; runtime acceptance pending.

- Normal startup and New open a staged format selector with size/orientation,
  progressive/interlaced rate and OK/Cancel. A loaded project bypasses startup.
  Unsaved replacement requires explicit acknowledgement; Cancel preserves state.
- A new project contains two muted black safety inputs instead of seven demos.
- Portrait 1080x1920 and landscape Full HD settings persist with rational rates
  and scan/field order in schema 10. Monitoring uses 360x640 or 640x360 progressive;
  full-HD/interlaced broadcast output is not implemented.
- Add Input Photos/Browser/Camera use bounded native forms with stable IDs,
  check-before-OK, visible errors and selection clearing. Photos keep real sorted
  files; Browser includes Full HD/portrait viewport presets; Camera loads actual
  device modes and validates the selected size/rate/scan/pixel subtype.
- Browser HTTP discovery no longer waits for EOF on a persistent connection.
  Two process-owner permits and the existing 5fps cap bound browser concurrency.
- Video/PNG/WAV/microphone flows remain connected to their existing workers.
  MP4 audio, MP3 and physical camera/audio compatibility remain unqualified.

Prepared evidence: engine/descriptor/migration tests, real widget tests, keepalive
failure regression, mandatory `--ui-preset-smoke` captures and preset roundtrip,
portrait loopback Browser/real changing Photos, all existing native smoke modes.
No unrun suite is reported as passed.

Local allowed checks: formatting, diff, repository/layer/asset/reference inventory
and native PowerShell syntax. One initial formatting attempt hit Windows file
mapping error 1224; a sequential retry succeeded. Hosted compile/lint/test/build/
native screenshot and resource evidence require an authorized remote revision.
No executable was compiled or launched on the owner's laptop.

Previous source f6ecbdd/run 39603431352 is a failed baseline: Browser discovery
timeout and wgpu teardown panic. Its Photos screenshot is old-source evidence,
not acceptance of UI-034. Physical hardware, extended tests and full production
readiness remain pending. See [ADR 0034](adr/0034-new-preset-and-native-input-formats.md).
