# UI-032 — Perbaikan Add Input

21 Oktober 2026. **Perbaikan terpilih diterima pada hosted Windows**:
source `4850b0fcab942f1d39ce66ede356225de017e526`, [run 39587449408 attempt 3](https://github.com/sansaks-jpg/rustmix/actions/runs/39587449408/attempts/3).
Check, Clippy, 282 tes (158 UI), release dan seluruh 16 proses native lulus.
Enam capture halaman Add Input dari source yang sama sudah dilihat langsung.
Kegagalan percobaan sebelumnya tetap tercatat di bawah; ini bukan perbaikan
reliabilitas GPU atau klaim kesetaraan seluruh fitur referensi.

| Temuan audit | Perubahan yang ditulis | Batas penerimaan |
|---|---|---|
| ADD-UX-01 — PNG terbawa lintas kategori | Path PNG terpisah dari nama generator; pemilihan kategori membersihkan draft tidak kompatibel dan meretir token; hanya Image yang dapat mengomit PNG tervalidasi | Regresi klik PNG → Colour/Title/generator, nama `Studio A.1`, edit path dan hasil worker terlambat lulus pada hosted Windows |
| ADD-UX-02 — Image bercampur generator | Halaman PNG tersendiri: path/Browse, preview nyata, Check image, Clear selection dan status. Worker tidak lagi otomatis menambah input setelah selesai membaca | Hanya PNG; belum ada recent Image, JPG/BMP. Clear selection tidak menghapus berkas |
| ADD-UX-03 — Colour/Title/GT menyesatkan | Colour menampilkan dua generator yang memang tersedia (gradient/bars); solid/transparent dijelaskan belum tersedia. Title menjadi halaman unavailable; sampel lower third tetap di Generators. GT diganti label Countdown | Belum ada pemilih solid colour, loader XAML/GT, editor/template gallery atau backend baru |
| ADD-UX-04 — kategori hilang/tidak bisa diperiksa | Katalog berurutan dengan DVD, Camera, NDI/OMT/Desktop, Video Delay dan Telestrator; semua kategori unavailable dapat dibuka untuk membaca alasan, dengan OK nonaktif | Halaman alasan bukan implementasi perangkat/protokol. Tidak ada discovery, koneksi atau pemasangan SDK otomatis |
| ADD-UX-05 — OK/form audio tidak konsisten | OK mengikuti validasi terkini serta prekondisi engine, kapasitas, posisi sisip dan transisi. WAV memakai path/Browse sebaris, ringkasan pemeriksaan dan nama; status tetap berada dalam formulir | Belum ada recent WAV/MP3; monitoring tidak dimulai oleh Add. Uji callback/perangkat fisik tetap belum dilakukan |
| ADD-UX-06 — ikon berdasarkan ordinal | Ikon vektor asli dipetakan ke semantik sumber; metadata selected/accessibility dan tanda fokus ditambahkan; kategori aktif dapat diperlihatkan di sidebar yang di-scroll | Tidak menyalin ikon atau artwork referensi; pembacaan screen reader dan DPI fisik belum diuji |

Seluruh perubahan perilaku berada di `crates/ui-egui`; typed AddInputAt tetap
menjadi jalur komit. Tidak ada perubahan engine/platform source, Cargo.toml,
Cargo.lock, format proyek atau runtime pihak ketiga. Perubahan dokumentasi yang
sudah ada di workspace dipertahankan dan tidak dianggap hasil pekerjaan ini.

Delapan regresi baru lulus:

1. Klik nyata PNG tervalidasi → Colour/Title/generator → OK; descriptor tidak
   membawa PNG lama, nama bertitik tetap label, Program/Preview dan berkas tetap aman.
2. Edit path PNG membuang pixel/validasi lama; hanya hasil file baru yang dapat dikomit.
3. Hasil PNG terlambat tidak mengisi ulang kategori lain.
4. Seluruh halaman unavailable tidak dapat mengomit sumber simulasi.
5. Form/field/footer di 734×466, 1100×700, 1600×900 dan 1917×1168.
6. Kapasitas penuh dan transisi aktif menonaktifkan OK tanpa mengubah Program.
7. Tab/Enter di sidebar dan semantik ikon sumber.
8. Berpindah kategori meretir metadata video tetapi tetap melacak satu probe
   pending; hasil generasi lama ditolak setelah kembali ke Video.

Regresi PNG/WAV/menu cepat sebelumnya disesuaikan dengan halaman sumber dan
tahap pemeriksaan eksplisit. Tidak ada tes dihapus atau gate kegagalan dilonggarkan.
Uji penolakan file MP4 dilakukan pada halaman Image; nama generator berisi titik
sekarang sah sebagai label dan tidak lagi ditafsirkan sebagai file.

Harness hosted menambahkan `--ui-add-smoke`: fixture PNG prosedural, klik sidebar
Image → Colour → Title → Audio → Microphone → Generators → Cancel, enam pasangan
PNG/element JSON dan `add-input-workflow-ok.json`. Proses harus membuktikan proyek
dan output tidak berubah, tidak mulai monitoring, serta menyelesaikan stage,
readback dan GPU drain. Batas proses internal 180 detik, batas proses eksternal
600 detik dan batas mode lama tetap. Workflow PNG lama kini memeriksa gambar
sebelum mengklik OK; save/reopen dan pemeriksaan pixel tetap diwajibkan.

Pemeriksaan lokal yang dijalankan setelah perubahan sumber:

| Pemeriksaan | Hasil |
|---|---|
| `cargo fmt --all -- --check` | Lulus; hanya formatting/parsing, bukan pemeriksaan tipe |
| `scripts/check-repository.ps1` | Lulus inventaris/layer/asset; full reference match tetap false |
| `git diff --check` | Lulus |
| Parser PowerShell untuk `scripts/ci-native-smoke.ps1` | Lulus |
| Compile, Clippy, tes Rust, release build | Hosted Windows lulus pada 4850b0f, run39587449408 attempt3 |
| Screenshot native baru dan perbandingan visual | Enam PNG/JSON source4850b0f diperiksa; seluruh16 proses native lulus attempt3 |

Penerimaan dibatasi pada fixture dan halaman terpilih. Tidak ada bukti perangkat
fisik, screen reader, DPI fisik, endurance atau backend yang belum diimplementasikan.
Lihat [ADR 0032](adr/0032-isolated-add-input-drafts-and-source-pages.md).

## Catatan kandidat gagal dan perbaikannya

`185dd78` / [run 39582724746](https://github.com/sansaks-jpg/rustmix/actions/runs/39582724746)
lulus formatting, inventaris, locked workspace check dan Clippy dengan warnings
as errors. Suite UI mencatat 153 lulus dan lima gagal; build release/native belum
dijalankan. Empat regresi baru mendeteksi harness yang belum memakai frame aplikasi
lengkap (metadata chrome tidak tercatat), Ctrl+A tanpa modifier command, dan
fokus sidebar yang tidak diminta saat diklik. Satu regresi lama mengklik kategori
Geometry yang kini berada di luar viewport setelah katalog disusun ulang.

`3920b4c` memperbaiki pengiriman Ctrl+A, memakai frame aplikasi nyata, meminta
fokus keyboard pada sidebar dan mengarahkan tes busy-editor ke kategori Grid
yang benar-benar terlihat. Tidak ada assertion dihapus, tes diabaikan atau gate
build/native dilonggarkan.

Run `39585833992` untuk `3920b4c` lulus check, Clippy, seluruh 282 tes (158 UI)
dan release build. Native gagal pada proses `--ui-add-smoke`, stage 3:
`Choose a PNG image and check it before adding.` Empat proses sebelumnya
menyelesaikan screenshot/reopen dan GPU drain. Kegagalan tetap dipertahankan;
paket executable dari run gagal tidak dianggap diterima.

`4850b0f` memisahkan pengisian path dari klik Check image dan menunggu formulir
stabil sebelum mengirim klik native. Assertion readiness, screenshot, deadline,
state dan GPU drain tetap wajib. Run `39587449408` attempt 1 lulus seluruh 282 tes, release, enam capture
Add Input dan `add-input-workflow-ok.json`; seluruh enam PNG dilihat langsung.
Proses `--ui-input-smoke` juga menulis `input-smoke-ok.json` pada stage 10,
namun drain GPU 10s timeout dan destructor queue wgpu-core 29.0.4 panic
(exit 101). Adapter: Microsoft Basic Render Driver/DX12/CPU, Windows2022.
Tidak ada smoke-error atau assertion Add Input yang gagal pada percobaan ini.
Run keseluruhan tetap gagal, proses berikutnya belum berjalan. Timeout GPU ini
bukan dianggap selesai; attempt 2 pada SHA yang sama dijalankan dengan seluruh
gate tetap untuk memeriksa reproduksinya. Attempt 2 kembali lulus 282 tes/release, tetapi gagal pada proses berbeda:
`--ui-dialog-smoke` stage 51 telah mengambil seluruh capture tanpa assertion
gagal, kemudian drain GPU/destructor queue timeout (exit 101). Proses Add baru
belum dijalankan pada attempt 2. Ini sama dengan kelas kegagalan penutupan yang
sudah tercatat sebelum UI-032, bukan bukti perbaikan GPU. Attempt 3 dijalankan
pada SHA yang tetap dengan seluruh gate asli; hasilnya masih pending.

Attempt 3 pada SHA4850b0f lulus seluruh langkah termasuk seluruh16 proses native dan packaging. Dua kegagalan GPU sebelumnya tetap merupakan batas reliabilitas terbuka.
