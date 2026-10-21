# UI-032 — Perbaikan Add Input

21 Oktober 2026. **Kandidat perubahan sumber; build, tes Rust dan validasi native
revisi baru belum dijalankan.** Referensi dan baseline audit tetap tersedia di
[audit Add Input](add-input-uiux-audit.md). Tidak ada klaim tampilan baru sudah lulus.

| Temuan audit | Perubahan yang ditulis | Batas penerimaan |
|---|---|---|
| ADD-UX-01 — PNG terbawa lintas kategori | Path PNG terpisah dari nama generator; pemilihan kategori membersihkan draft tidak kompatibel dan meretir token; hanya Image yang dapat mengomit PNG tervalidasi | Regresi klik PNG → Colour/Title/generator, nama `Studio A.1`, edit path dan hasil worker terlambat disiapkan; belum dijalankan |
| ADD-UX-02 — Image bercampur generator | Halaman PNG tersendiri: path/Browse, preview nyata, Check image, Clear selection dan status. Worker tidak lagi otomatis menambah input setelah selesai membaca | Hanya PNG; belum ada recent Image, JPG/BMP. Clear selection tidak menghapus berkas |
| ADD-UX-03 — Colour/Title/GT menyesatkan | Colour menampilkan dua generator yang memang tersedia (gradient/bars); solid/transparent dijelaskan belum tersedia. Title menjadi halaman unavailable; sampel lower third tetap di Generators. GT diganti label Countdown | Belum ada pemilih solid colour, loader XAML/GT, editor/template gallery atau backend baru |
| ADD-UX-04 — kategori hilang/tidak bisa diperiksa | Katalog berurutan dengan DVD, Camera, NDI/OMT/Desktop, Video Delay dan Telestrator; semua kategori unavailable dapat dibuka untuk membaca alasan, dengan OK nonaktif | Halaman alasan bukan implementasi perangkat/protokol. Tidak ada discovery, koneksi atau pemasangan SDK otomatis |
| ADD-UX-05 — OK/form audio tidak konsisten | OK mengikuti validasi terkini serta prekondisi engine, kapasitas, posisi sisip dan transisi. WAV memakai path/Browse sebaris, ringkasan pemeriksaan dan nama; status tetap berada dalam formulir | Belum ada recent WAV/MP3; monitoring tidak dimulai oleh Add. Uji callback/perangkat fisik tetap belum dilakukan |
| ADD-UX-06 — ikon berdasarkan ordinal | Ikon vektor asli dipetakan ke semantik sumber; metadata selected/accessibility dan tanda fokus ditambahkan; kategori aktif dapat diperlihatkan di sidebar yang di-scroll | Tidak menyalin ikon atau artwork referensi; pembacaan screen reader dan DPI fisik belum diuji |

Seluruh perubahan perilaku berada di `crates/ui-egui`; typed AddInputAt tetap
menjadi jalur komit. Tidak ada perubahan engine/platform source, Cargo.toml,
Cargo.lock, format proyek atau runtime pihak ketiga. Perubahan dokumentasi yang
sudah ada di workspace dipertahankan dan tidak dianggap hasil pekerjaan ini.

Delapan regresi baru disiapkan:

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
| Compile, Clippy, tes Rust, release build | Belum dijalankan untuk revisi ini; memerlukan hosted Windows |
| Screenshot native baru dan perbandingan visual | Belum dijalankan; enam capture sudah diwajibkan di harness |

Sebelum penerimaan: jalankan hosted check/Clippy/semua tes/release/native pada
commit perbaikan yang sama, unduh dan lihat keenam PNG/JSON baru serta bukti
Video/PNG/WAV/reopen, dan pertahankan setiap kegagalan dalam catatan. Menulis
regresi atau harness bukan bukti bahwa tes atau tangkapan tersebut sudah lulus.
Lihat [ADR 0032](adr/0032-isolated-add-input-drafts-and-source-pages.md).
