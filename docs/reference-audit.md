# Audit seluruh temuan referensi

## Re-verification addendum (2026-10-24, UI-040)

The pinned study snapshot was re-verified against the installed product
(29.0.0.48) by **static clean-room inspection only**: the product was never
launched, decompiled or copied from. Method — directory and file-version
inventory of the install tree; import-table/section inspection of the core
binaries; read of the embedded configuration and XML provider files; extraction
of the contiguous CamelCase shortcut/API function table and the transition,
transport and layout enum strings from the binary image; one-to-one
cross-check against the vendor's public feature and shortcut-function pages;
per-claim confirmation against this repository's own wire measurements.

Confirmed corrections (full detail in the `reverification` object of
[reference-audit.json](reference-audit.json); every one is enforced in code and
tests on the rustmix side):

- `OverlayInput` spellings exist only for channels 1–8 (binary string table,
  committed shortcut catalogues, measured wire). Channels 9–16 are addressed by
  the separate `Stinger1–8` family. The older specification table claiming
  `OverlayInput` with N in 1..=16 is wrong.
- `OverlayInput<n>Off` clears a channel's assignment; `OverlayInput<n>Out`
  leaves the assignment in place. The older summary made `Out` the clear
  spelling and omitted `Off` entirely.
- The `SetLayer` family covers geometry members (X/Y/Width/Height/Zoom plus
  crop/pan siblings) but contains no Alpha or Rotation member; a separate
  unindexed `SetAlpha` exists.
- The observed state document's `outputs` element contains only `output`
  children (some carrying a fullscreen type attribute); no separate
  `<fullscreen>` child element exists in any of the seven captured state XMLs.
- The verified input-category catalogue members are Aqua and Blue; the older
  task summary's Azure/MidnightBlue names appear nowhere in the binary.
- The main window shows **five** transport setup/arm pairs; the "six transports"
  summaries contradict their own geometry tables.
- Further confirmed document-internal contradictions (stale duplicate-capture
  claim disproven by differing SHA-256 hashes, an "unresolved" finding the same
  document later resolves, an unmeasured theme colour, a misapplied CONFIRMED
  label on an unsound era inference, a dependency claim contradicted by the
  manifests, a promised inventory file never produced, an off-by-one
  replay-controller member count, and gitignored gui-pass images behind some
  overlay/category resolutions) are recorded in the JSON addendum so they are
  not re-imported into rustmix.

RustMix status rows below were reconciled to the post-UI-040 tree (20 effects
with golden curve pins, input-backed standard overlay channels, the nine-bus
sample mixer with meter taps, and the loopback HTTP control API). Full-match
acceptance remains false.

Task 1.3 source update (2026-10-19): four external-output route/configuration records, atomic
complete-slot Apply and schema-6 persistence are implemented in RustMix. This is **source-only**
experimental configuration work; new engine/widget/native acceptance is pending. No external
frame sink, hardware device, network output, XML state endpoint or reference parity is claimed.
Historical test results and schema-5 evidence below apply only to their named revisions.

Tanggal: 2026-10-18. **Kesimpulan: belum semua fitur sesuai.** Audit ini mencakup seluruh 30 area inventaris, 40 tipe input, 20 efek transisi, 16 kanal overlay/stinger, dan 9 tujuan audio. Jumlah ini adalah cakupan pemeriksaan, bukan jumlah fitur yang sudah tersedia.

Snapshot sumber: `rustmix-reference/vmix`, studi versi 29.0.0.48. Katalog memuat 777 nama shortcut, bukan 777 fungsi HTTP. Catatan trace T11 mengoreksi ringkasan awal: sejumlah nama shortcut ditolak endpoint, sedangkan Cut/Fade diterima tanpa konstanta shortcut terpisah. RustMix belum memiliki endpoint/protokol kompatibel. Referensi Rust sendiri adalah model domain/API dengan backend tertentu dinyatakan unavailable, sehingga daftar enum tidak dipakai sebagai bukti perangkat/codec berjalan.

Status: `experimental` = sebagian alur prototipe berjalan; `stub` = hanya simulasi/kendali; `not_started` = belum ada backend/alur kerja. Tidak ada area di bawah yang diklaim setara penuh atau production-ready.

## Semua area yang diperiksa

| # | Area referensi | Status RustMix | Yang benar-benar ada | Kekurangan terhadap temuan | Bukti sumber / pengujian |
|---|---|---|---|---|---|
| 1 | Preview / Program | experimental | Independent Preview, Program, explicit Cut/Take and source exchange | Synthetic/PNG reference raster only; no qualified broadcast output | [kode](../crates/engine/src/lib.rs) / [uji](../crates/engine/tests/workflow.rs) |
| 2 | Transitions | experimental | Four independent saved slots and all 20 effects as shared per-pixel CPU renderers, each motion curve pinned by golden frames at 25/50/75% | No vendor-pixel parity or GPU parity; Full HD benchmark of the eight per-pixel effects pending; stinger clip compositing absent | [kode](../crates/engine/src/render.rs) / [uji](../crates/engine/tests/transitions.rs) |
| 3 | Inputs | experimental | Seven own generators plus bounded external PNG | 40 observed wire types are not 40 working backends; video/capture/network/browser absent | [kode](../crates/ui-egui/src/media.rs) / [uji](../crates/engine/tests/inputs.rs) |
| 4 | Overlays | experimental | Eight standard channels composite a bound input (PiP) under the caption text via `overlay.input.set`, plus title/subtitle panels, staged text/position edits, F1–F8 defaults and a verified 4→5 schema migration inside the 16-channel overlay/stinger bank | No per-channel effects/timing/alpha/transforms on standard channels. Stinger channels 9–16 hold model/trigger/cut-point state and exchange buses but render no clip; the occupied-channel state-document content form is not evidenced | [kode](../crates/engine/src/overlays.rs) / [uji](../crates/engine/tests/overlays.rs) |
| 5 | Multiview | experimental | In-app Split and full source grid layouts | No two independent multiview outputs or all five observed layout contracts | [kode](../crates/ui-egui/src/panels.rs) / [uji](../crates/ui-egui/src/smoke.rs) |
| 6 | Titles / GT graphics | experimental | Original title/subtitle panels and generated graphics | No GT/XAML compatibility, animation templates, ticker or editable production clock | [kode](../crates/ui-egui/src/panels.rs) / [uji](../crates/ui-egui/src/smoke.rs) |
| 7 | Audio mixer / matrix | experimental | Real 48 kHz sample mixer with nine configured buses (Master, A–G, Headphones), per-input nine-way sends, mix-minus guest returns, schema-11 migration and per-source post-gain pre-clamp meter taps | No device-callback/hardware qualification, DSP/EQ, loudness metering, A/V sync or dedicated headphone device output | [kode](../crates/engine/src/audio_engine.rs) / [uji](../crates/engine/tests/audio_samples.rs) |
| 8 | Routing / external outputs | experimental, source only | Four persisted configuration records; atomic complete-slot command and independent staged UI drafts | New regression execution/native acceptance pending; no external frame sink, SDI/virtual camera/NDI/OMT/SRT or hardware output | [kode](../crates/engine/src/output.rs) / [uji disiapkan](../crates/engine/tests/outputs.rs) |
| 9 | Streaming | not_started | Unavailable starts remain gated | No encoder, RTMP/RTMPS transport, profiles, reconnect or five destinations | [kode](../crates/ui-egui/src/lib.rs) / [uji](../crates/ui-egui/src/smoke.rs) |
| 10 | Recording | not_started | Unavailable starts remain gated | No Program recording, independent WAV, splitting, recovery or two recording outputs | [kode](../crates/ui-egui/src/lib.rs) / [uji](../crates/ui-egui/src/smoke.rs) |
| 11 | Replay | not_started | None | No replay storage, camera channels, events, marks, speed control or exports | [kode](../Cargo.toml) |
| 12 | PTZ camera control | not_started | None | No VISCA/vendor/UVC PTZ provider, movement or presets | [kode](../Cargo.toml) |
| 13 | Shortcuts / MIDI / surfaces | experimental | Saved keyboard action/key/modifier mappings with validation and repeat/focus guards | No MIDI, activators, controllers, voice or 19 input trigger events | [kode](../crates/engine/src/shortcuts.rs) / [uji](../crates/ui-egui/src/smoke.rs) |
| 14 | Presets / sessions | experimental | Own schema6 with schema1–5 migration, atomic settings/saves, backups and checkpoints | New schema6 tests prepared, execution pending; hosted run 39129117131 verifies earlier schema5 only; no .vmix interoperability, media packaging or power-loss qualification. | [kode](../crates/engine/src/persistence.rs) / [uji](../crates/engine/tests/inputs.rs) |
| 15 | Settings | experimental | Validated preferences, categories/theme/colors and one atomic name/preferences apply; unavailable scaffolds explicitly read-only | 256 observed properties are not implemented; theme/color/display and advanced fields include unavailable/scaffold controls | [kode](../crates/ui-egui/src/dialogs.rs) / [uji](../crates/ui-egui/src/smoke.rs) |
| 16 | HTTP / TCP / WebSocket API | experimental | Opt-in loopback HTTP/1.1 GET endpoint (default port 8088, 8 KiB request bound, never auto-started): state document rendering plus a 25-spelling executable vocabulary (OverlayInput1–8 +Off, Stinger1–8, PlayPause and the earlier families) | No TCP text protocol, WebSocket, write-side spellings beyond the mapped set, authentication beyond loopback binding or an operator-facing enablement workflow | [kode](../crates/automation/src/server.rs) / [uji](../crates/automation/tests/api_contract.rs) |
| 17 | Scripting | not_started | None | No scripts, timers, runtime sandbox or script permissions | [kode](../Cargo.toml) |
| 18 | Data sources | not_started | None | No database/CSV/JSON/browser/social connector or live title binding | [kode](../Cargo.toml) |
| 19 | Browser inputs | not_started | None | No embedded browser engine or texture/audio integration | [kode](../Cargo.toml) |
| 20 | Virtual sets | not_started | Procedural studio picture is a generator | Not a composited camera/talent/virtual-set workflow; no imported set assets | [kode](../crates/engine/src/render.rs) |
| 21 | Video call / guests | not_started | None | No WebRTC, guest sessions, consent workflow or mix-minus return | [kode](../Cargo.toml) |
| 22 | Voice control | not_started | None | No speech engine, microphone source or confidence/test behavior | [kode](../Cargo.toml) |
| 23 | Tally lights | not_started | None | No hardware/network tally backend or controller status | [kode](../Cargo.toml) |
| 24 | Statistics / diagnostics | experimental | Measured reference Program cadence, process memory and local status counters | Hosted cadence targets failed; no full media/QoS/GPU/AV/hardware diagnostics | [kode](../crates/ui-egui/src/profile.rs) / [uji](../crates/ui-egui/src/program.rs) |
| 25 | Production clocks | stub | Fixed one-minute procedural countdown and independent source transports | No editable production clock/countdown engine or title countdown API | [kode](../crates/engine/src/render.rs) / [uji](../crates/engine/tests/workflow.rs) |
| 26 | VST3 audio plugins | not_started | None | No real audio/plugin hosting or license-reviewed VST3 adapter | [kode](../Cargo.toml) |
| 27 | NDI | not_started | None | No SDK/runtime integration, sender, receiver or passthrough recording | [kode](../Cargo.toml) |
| 28 | OMT | not_started | None | No protocol/runtime adapter, input or output | [kode](../Cargo.toml) |
| 29 | Color correction / LUT | experimental | Persisted lift/gain/saturation, mirror/sharpen, approximate chroma key and PIP | No full CC family, LUT/masks, ten-layer crop/rotation/animation graph or color-managed GPU pipeline | [kode](../crates/ui-egui/src/panels.rs) / [uji](../crates/ui-egui/src/smoke.rs) |
| 30 | Social / cloud destinations | not_started | None | No authenticated provider integration; no credentials requested or internet Program sent | [kode](../Cargo.toml) |

## 40 tipe input

| Temuan | Status | Batas aktual |
|---|---|---|
| Video | not_started | No corresponding working source backend |
| VideoList | not_started | No corresponding working source backend |
| VideoLoop | not_started | No corresponding working source backend |
| Image | experimental | External PNG only; no generic still-image format parity |
| Photos | not_started | No corresponding working source backend |
| ImageSequence | not_started | No corresponding working source backend |
| AudioFile | not_started | No corresponding working source backend |
| Audio | not_started | No corresponding working source backend |
| DVD | not_started | No corresponding working source backend |
| PowerPoint | not_started | No corresponding working source backend |
| Flash | not_started | No corresponding working source backend |
| Xaml | not_started | No corresponding working source backend |
| Waveform | not_started | No corresponding working source backend |
| Capture | not_started | No corresponding working source backend |
| DesktopCapture | not_started | No corresponding working source backend |
| NDI | not_started | No corresponding working source backend |
| OMT | not_started | No corresponding working source backend |
| SRT | not_started | No corresponding working source backend |
| VLC | not_started | No corresponding working source backend |
| Stream | not_started | No corresponding working source backend |
| Zoom | not_started | No corresponding working source backend |
| VideoCall | not_started | No corresponding working source backend |
| Telestrator | not_started | No corresponding working source backend |
| Browser | not_started | No corresponding working source backend |
| Title | experimental | Own Program title panels, not a general title/template input |
| GT | not_started | No corresponding working source backend |
| Colour | experimental | Own solid-color generator |
| VirtualSet | not_started | No corresponding working source backend |
| Virtual | not_started | No corresponding working source backend |
| Mix | not_started | No corresponding working source backend |
| Output | not_started | No corresponding working source backend |
| Preview | not_started | No corresponding working source backend |
| Blank | not_started | No corresponding working source backend |
| Placeholder | not_started | No corresponding working source backend |
| Signal | experimental | Own procedural color bars |
| VideoDelay | not_started | No corresponding working source backend |
| ProductionClocks | not_started | No corresponding working source backend |
| Replay | not_started | No corresponding working source backend |
| ReplayPreview | not_started | No corresponding working source backend |
| ReplayOverlay | not_started | No corresponding working source backend |

## 20 efek transisi

| Temuan | Status | Batas aktual |
|---|---|---|
| Fade | experimental | Own CPU algorithm with endpoint/direction tests; no cross-vendor pixel parity |
| Zoom | experimental | Own CPU zoom-dissolve algorithm with endpoint tests; golden curve pin at 25/50/75% |
| Wipe | experimental | Own CPU algorithm with endpoint/direction tests; no cross-vendor pixel parity |
| Slide | experimental | Own CPU algorithm with endpoint/direction tests; no cross-vendor pixel parity |
| Fly | experimental | Own CPU fly-in with scale trajectory and endpoint tests; golden curve pin at 25/50/75% |
| CrossZoom | experimental | Own CPU opposing zoom-dissolve algorithm with endpoint tests; golden curve pin at 25/50/75% |
| FlyRotate | experimental | Own CPU fly-in with rotation and endpoint tests; golden curve pin at 25/50/75% |
| Cube | experimental | Own CPU cube-turn algorithm with an edge-on black midpoint and row tests; golden curve pin at 25/50/75% |
| CubeZoom | experimental | Own CPU cube-turn with squash and endpoint tests; golden curve pin at 25/50/75% |
| VerticalWipe | experimental | Own CPU algorithm with endpoint/direction tests; no cross-vendor pixel parity |
| VerticalSlide | experimental | Own CPU algorithm with endpoint/direction tests; no cross-vendor pixel parity |
| Merge | experimental | Own CPU recede/advance algorithm with endpoint and distinctness tests; golden curve pin at 25/50/75% |
| WipeReverse | experimental | Own CPU algorithm with endpoint/direction tests; no cross-vendor pixel parity |
| SlideReverse | experimental | Own CPU algorithm with endpoint/direction tests; no cross-vendor pixel parity |
| VerticalWipeReverse | experimental | Own CPU algorithm with endpoint/direction tests; no cross-vendor pixel parity |
| VerticalSlideReverse | experimental | Own CPU algorithm with endpoint/direction tests; no cross-vendor pixel parity |
| BarnDoor | experimental | Own CPU algorithm with endpoint/direction tests; no cross-vendor pixel parity |
| RollerDoor | experimental | Own CPU algorithm with endpoint/direction tests; no cross-vendor pixel parity |
| AlphaFade | experimental | Own CPU alpha-coverage dissolve with target-alpha regression tests; golden curve pin at 25/50/75% |
| Cut | experimental | Own CPU algorithm with endpoint/direction tests; no cross-vendor pixel parity |

## 16 kanal overlay/stinger

| Temuan | Status | Batas aktual |
|---|---|---|
| 1 | experimental | Standard channel: 16-channel bank, Program render, UI, F-key and persistence are real; an optional bound input composites as PiP under the caption text via `overlay.input.set` (serde-default binding, legacy files load idle); no per-channel effects, timing or alpha |
| 2 | experimental | Title-panel channel: 16-channel bank, Program render, UI, F-key and persistence are real; no source overlay, per-channel effects, timing or alpha |
| 3 | experimental | Title-panel channel: 16-channel bank, Program render, UI, F-key and persistence are real; no source overlay, per-channel effects, timing or alpha |
| 4 | experimental | Title-panel channel: 16-channel bank, Program render, UI, F-key and persistence are real; no source overlay, per-channel effects, timing or alpha |
| 5 | experimental | Title-panel channel: 16-channel bank, Program render, UI, F-key and persistence are real; no source overlay, per-channel effects, timing or alpha |
| 6 | experimental | Title-panel channel: 16-channel bank, Program render, UI, F-key and persistence are real; no source overlay, per-channel effects, timing or alpha |
| 7 | experimental | Title-panel channel: 16-channel bank, Program render, UI, F-key and persistence are real; no source overlay, per-channel effects, timing or alpha |
| 8 | experimental | Title-panel channel: 16-channel bank, Program render, UI, F-key and persistence are real; no source overlay, per-channel effects, timing or alpha |
| 9 | stub | Model, SetStinger/Trigger/Stop commands and cut-point Program/Preview exchange only; no stinger clip compositing, timing raster or effects |
| 10 | stub | Model, SetStinger/Trigger/Stop commands and cut-point Program/Preview exchange only; no stinger clip compositing, timing raster or effects |
| 11 | stub | Model, SetStinger/Trigger/Stop commands and cut-point Program/Preview exchange only; no stinger clip compositing, timing raster or effects |
| 12 | stub | Model, SetStinger/Trigger/Stop commands and cut-point Program/Preview exchange only; no stinger clip compositing, timing raster or effects |
| 13 | stub | Model, SetStinger/Trigger/Stop commands and cut-point Program/Preview exchange only; no stinger clip compositing, timing raster or effects |
| 14 | stub | Model, SetStinger/Trigger/Stop commands and cut-point Program/Preview exchange only; no stinger clip compositing, timing raster or effects |
| 15 | stub | Model, SetStinger/Trigger/Stop commands and cut-point Program/Preview exchange only; no stinger clip compositing, timing raster or effects |
| 16 | stub | Model, SetStinger/Trigger/Stop commands and cut-point Program/Preview exchange only; no stinger clip compositing, timing raster or effects |

## 9 tujuan audio

| Temuan | Status | Batas aktual |
|---|---|---|
| Master | stub | Real 48 kHz sample mixing with explicitly started local WASAPI monitoring and measured peaks; no loudness metering |
| Headphones | stub | Mixes in the engine matrix with per-input sends; no dedicated headphone device output |
| A | stub | Real sample mixing and per-input sends in the engine matrix with measured per-bus peak/RMS surfaced in the mixer strips; no dedicated device output |
| B | stub | Real sample mixing and per-input sends in the engine matrix with measured per-bus peak/RMS surfaced in the mixer strips; no dedicated device output |
| C | stub | Real sample mixing and per-input sends in the engine matrix with measured per-bus peak/RMS surfaced in the mixer strips; no dedicated device output |
| D | stub | Real sample mixing and per-input sends in the engine matrix with measured per-bus peak/RMS surfaced in the mixer strips; no dedicated device output |
| E | stub | Real sample mixing and per-input sends in the engine matrix with measured per-bus peak/RMS surfaced in the mixer strips; no dedicated device output |
| F | stub | Real sample mixing and per-input sends in the engine matrix with measured per-bus peak/RMS surfaced in the mixer strips; no dedicated device output |
| G | stub | Real sample mixing and per-input sends in the engine matrix with measured per-bus peak/RMS surfaced in the mixer strips; no dedicated device output |

## Kontrak alur UI yang diperiksa terpisah

UI-013 adalah kandidat sumber; uji hosted masih berjalan. Bukti runtime di bawah tetap baseline UI-012 sampai hasil baru diterima.

| Alur | Status | Perilaku aktual | Batas terhadap temuan |
|---|---|---|---|
| Basic / Advanced presentation | experimental | Independent persisted UI mode; Basic retains both monitors, Quick Play, input selection/cut/overlays 1-4, Add Input, audio mixer and lock. Does not change Program or multiview choice. | Hosted widget/native Basic correction passed; overlays/audio/inputs retain the general prototype limitations. Global-mode restore has source coverage only; native save/reopen verifies project configuration. The existing lock suppresses production keys; a complete pointer/layout lock contract is unverified. |
| Named input playlist | experimental | 1-16 saved lists, at most 128 items, independent effects/hold times, reorder, loop/manual/selected start/clear-on-start, start/stop/next/previous and offline alarms. | Explicit synthetic/PNG holds only; no file-duration inference, video end events, pause/seek or media playback engine. |
| Keyboard bindings / palette | experimental | Validated persisted logical key/modifier/action mappings, focus/repeat guards and quick action palette. | At most 64 bindings and the implemented engine action subset; no 777-function dispatcher, MIDI or control surfaces. |
| Four transition slots | experimental | Each saved slot has a real CPU effect and duration; in-flight transition pins its configuration. | 12 of 20 observed effects; no stingers/GPU equivalence or vendor-pixel parity. |
| Manual T-bar | experimental | First slot selects manual effect/duration; gesture pins effect/duration/start, reverses and commits endpoint once. | UI-013 regression pending; CPU raster only, no hardware controller qualification. |
| Input Settings transactions | experimental | Staged persisted PNG/generator effects, Cancel discard, bounded loading/relink, duplicate and source lifecycle. | No ten-layer transform/crop/rotation animation graph, full color tools or real video/device properties. |
| Input category filter | experimental | Schema5 stable per-input category assignment, six standard plus sixteen editable custom definitions, typed `InputCategory` contract (All, six preset colours, `Custom(1..=16)` clamped, filter-only Search) with the engine `visible_inputs` query, context-menu assignment and real category filters. | Task 1.2 port added the typed contract, clamping and engine query with unit/integration tests; hosted native acceptance passed in run 39129117131 (category capture + reopen) on the pre-port code path; no vendor pixel/protocol parity claim and no wire ordinals (no control endpoint). |
| Input search | experimental | Staged Input Search dialog with OK/Cancel, case-insensitive title/type/number filtering and no implicit switching. | Hosted native acceptance passed in run 39129117131 (search capture); at most 48 query characters, no remote search protocol. |
| Theme and Preview/Program colors | experimental | Original persisted Dark/Light appearance and RGB Preview/Program roles; colors reach monitor headers and source selection. | Hosted native acceptance passed in run 39129117131 (light-theme capture + reopen); operator/media cards retain dark surfaces, vendor theme list/pixel parity and physical DPI are unqualified. |
| Fullscreen Program outputs | stub | Fullscreen expands the application window; routed-output menu explicitly says unavailable. | No independent Program-only monitor window, two routed fullscreen outputs, display selection or hotplug handling. |
| Audio Mixer pane toggle | experimental | Actual Audio Mixer hide/show with complete previous dock restoration; saves retain the complete layout. | Hosted native acceptance passed in run 39129117131 (audio-pane hide/restore capture); changes to the temporary hidden layout are not persisted; no physical monitoring/audio backend. |
| Settings categories and fields | experimental | Atomic name/preferences apply, staged categories/theme/keys, explicit read-only unavailable display/output scaffolds. | Hosted native acceptance passed in run 39129117131 (Settings dialog capture + atomic apply/reopen). Most of 256 declared reference properties have no backend and cannot be configured. |

## Koreksi bukti referensi

- **T9:** wire overlay hanya1–8, dokumen state16 termasuk8 stinger. Off mengosongkan assignment; Out mempertahankannya. Efek raster Out belum dibuktikan dari state XML.
- **T10:** respons kontrol adalah state sebelum aksi; request baca terpisah membuktikan hasil.
- **T11:** 777 nama adalah kosakata shortcut, bukan seluruh nama HTTP yang diterima. SetTransitionEffect/Duration ditolak di wire; Cut/Fade diterima. Tidak ada target palsu777/777 HTTP dalam gate.
- Nama20 efek dan40 input/256 properti adalah inventaris, bukan bukti algoritma, codec atau SDK berfungsi. Delapan efek yang geometrinya belum ditentukan tidak diisi alias.
- Clear Overlays tetap berlaku saat playlist mulai, mengikuti panduan publik yang sudah tercatat pada ADR0009.

Semua777 identifier dicatat di `shortcut_function_inventory.groups` dalam [data audit](reference-audit.json). Pemeriksa menghitung nama unik/hash. Gate protokol terpisah membutuhkan `wire_contract_qualified`; hitungan shortcut tidak dipakai sebagai denominator HTTP. Lihat [review seluruh dokumen](reference-review.md).

## Bukti dan keputusan penerimaan

- [Run Windows 39111788801](https://github.com/sansaks-jpg/rustmix/actions/runs/39111788801) pada kode `a01f498`: **75 tes lulus** (36 engine / 39 UI), kompilasi/lint, build, lima proses native, save/reopen dan packaging lulus. Dua pasangan JSON save/reopen identik; lima stderr native kosong. Ada 20 screenshot native dan satu fixture prosedural. Capture Basic, workspace 1600, playlist, Keyboard Settings dan input reopen dari run ini telah ditinjau; Basic menyisakan Quick Play dan kontrol inti, lalu Advanced pulih tanpa mengubah Program/Preview/revision.
- [Bukti native](https://github.com/sansaks-jpg/rustmix/actions/runs/39111788801/artifacts/12130639014) dan [paket Windows](https://github.com/sansaks-jpg/rustmix/actions/runs/39111788801/artifacts/12130624284). Tidak ada build atau peluncuran lokal.
- Target cadence **gagal** pada runner: 50 → 28.57 fps, 59.94 → 38.75 fps, 60 → 34.46 fps, dengan 177 / 175 / 212 frame terlewat; raster aktual 640×360. Hasil ini bukan tolok ukur GPU/perangkat operator dan tidak membuktikan broadcast real-time. Working set puncak terbesar 145.20 MiB dan private bytes maksimum 367.48 MiB pada lima proses singkat; bukan pengukuran VRAM/endurance.
- Kesetaraan penuh belum bisa diterima: backend video/capture/audio/record/stream/replay/SDK dan sebagian besar kontrol referensi belum tersedia. Perubahan UI-012 menutup beberapa celah, bukan seluruh inventaris.
- Data terstruktur ada di [reference-audit.json](reference-audit.json). Pemeriksa inventaris hanya memeriksa kelengkapan katalog dan konsistensi batas kode; bukan pengganti tes runtime setiap fitur. Mode `-RequireFullMatch` harus menolak keadaan saat ini, sehingga lulus inventaris tidak disamakan dengan kesetaraan penuh.

## Urutan pekerjaan berikutnya

1. Jadikan jalur picture-and-sound nyata: sumber audio 48 kHz, routing/impulse/mix-minus; video/file input berlisensi jelas, timestamp dan pengukuran default 30 fps.
2. Model 16 kanal (8 title panel + 8 stinger) dengan bank/command/migrasi sudah terpasang; berikutnya bangun overlay sumber nyata dan raster stinger dengan waktu cut/alpha yang diuji; lengkapi efek yang belum ada dan graph layer/crop/rotation.
3. Tambahkan output/encoder terisolasi, recording yang dapat dipulihkan dan stream ke server lokal; lakukan kajian lisensi sebelum runtime codec/SDK.
4. Tambahkan API/CLI/control schemas, kategori input, tema/warna, MIDI/triggers dan otomasi yang benar-benar dieksekusi.
5. Integrasi kamera/network/PTZ/tally/guests/NDI/OMT/VST3 hanya diterima setelah backend, izin lisensi dan perangkat yang diperlukan diuji. Tidak ada streaming publik atau lisensi berbayar yang diasumsikan disetujui.
