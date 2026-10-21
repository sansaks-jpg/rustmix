# Audit UI/UX Add Input — 21 Oktober 2026

Catatan lanjutan: audit di bawah adalah snapshot baseline 56048cb. Perbaikan
sumber berikutnya tercatat di [UI-032](ui032-add-input-changes.md); penerimaan
runtime/visual terpilih diterima pada 4850b0f/run39587449408 attempt3. Bukti hijau baseline tidak berlaku
otomatis untuk perubahan baru.

**Hasil: belum lulus kesesuaian Add Input secara menyeluruh.** Kerangka dialog dan
alur Video sudah mendekati referensi serta mempunyai bukti pengujian native.
Katalog, formulir Image/Colour/Title/Audio, ikon, dan pemisahan draft antarjenis
masih mempunyai kesenjangan. Audit ini tidak mengubah kode aplikasi atau menambah
backend. Temuan berikut masih terbuka, bukan perbaikan yang sudah diterapkan.

ID/owner: `ADD-UX-AUDIT-2026-10-21` / Codex. Lingkup hanya Add Input beserta menu
cepat, pemilihan sumber, validasi, Number, OK, Cancel dan Escape. Input Settings,
layout switcher, recording, streaming, output dan tema Light tidak dinilai ulang.

## Dasar bukti dan batas validasi

- Seluruh **33 JPEG `23*`** di
  `C:\Users\WORKPLUS\Documents\WEB\vmix-photos\screenshots` dilihat langsung,
  termasuk variasi Stream, navigasi setelah scroll dan pesan dependensi.
  `PETA-NAVIGASI.md` bagian 3 juga dibaca. Screenshot diam membuktikan tampilan
  yang tertangkap; catatan referensi menyebutkan OK tidak ditekan. Keberhasilan
  menambahkan sumber di aplikasi referensi tidak dapat disimpulkan darinya.
- Sumber runtime workspace dibandingkan dengan
  `56048cbb92d5dc4fd17f7e8b051b37ef555e27f5`: seluruh `src` UI, engine, platform,
  aplikasi, Cargo.toml/Cargo.lock, workflow Windows dan harness native **identik**.
  Perubahan dokumentasi yang sudah ada di workspace tetap dipertahankan.
- Status job dan log [Windows run 39556368477](https://github.com/sansaks-jpg/rustmix/actions/runs/39556368477)
  dibaca ulang dari GitHub dalam audit ini: check, Clippy, **274 tes di 17 suite
  (150 tes UI), nol gagal/diabaikan**, release, native smoke dan packaging sukses.
  Ini bukti run sebelumnya pada kode runtime yang sama, bukan tes baru sesi ini.
- PNG native `add-input-1600.png`, `add-input-reference-1917x1168.png` dan
  `video-source.png` di `artifacts/ui031-56048cb-selected/` dilihat langsung.
  Dimensi PNG dibaca ulang dari file; elemen JSON, validasi MP4/WAV dan enam
  kumpulan stage terkait main/input/dialog/layout/video/audio diperiksa.
  Keenamnya mencatat tidak ada stage gagal, stderr kosong, dan drain GPU sukses.
- Tidak ada kompilasi, tes Rust, peluncuran native, pengambilan screenshot baru,
  pengujian perangkat fisik atau perubahan cloud pada sesi audit ini. Kebijakan
  verifikasi hosted pada `docs/testing.md` tetap diikuti. Formulir non-Video
  dinilai melalui sumber dan tes/log yang ada; tidak tersedia screenshot baru
  untuk menyatakan seluruh formulir tersebut sudah cocok secara visual.

`23p-add-input-virtual-set.jpg` masih memperlihatkan pilihan Title dan galeri
Title; jangan menjadikannya spesifikasi formulir Virtual Set yang pasti.
`23r-add-input-video-call.jpg` memuat sisa gambar halaman sebelumnya;
`23t-vmix-call-error-closed.jpg` memberi tampilan Call yang lebih jelas.
Semua referensi hanya diamati secara lokal. Gambar, ikon, logo, media, alamat
riwayat pribadi dan template referensi tidak disalin ke proyek.

## Bagian yang dapat diterima dalam lingkup bukti

| Pemeriksaan | Hasil | Bukti dan batas |
|---|---|---|
| Pembukaan Video dan kategori yang diingat | Lulus alur terpilih | Default `VIDEO_TAB`; tes menu cepat 11 kategori dan Cancel/reopen lulus dalam run di atas. Ini belum membuktikan formulir kategori sama dengan referensi. |
| Susunan Video | Lulus struktur utama | Sidebar gelap, pilihan aktif, strip instruksi, path/Browse sebaris, daftar recent dan footer terpisah terlihat pada tiga PNG native. Name/Check dan batas MP4 video-only merupakan perbedaan yang disengaja dan terlihat. |
| Ukuran/footer | Lulus pada tangkapan yang diperiksa | Pada 1600×900 dialog 1131.2×660.9; pada 1917×1168 dialog 1355.3×860.6. Sidebar sekitar 18.4% lebar body; path/list/footer tidak bertumpuk. Referensi umumnya 1085×670, sehingga perbandingan ini proporsi, bukan klaim kesamaan piksel atau DPI. |
| Validasi Video | Lulus alur terpilih | OK tidak aktif sebelum probe; PNG native berkas tervalidasi menampilkan 160×90, 30 fps dan OK aktif. Tes perubahan path, file salah, cancel/hasil terlambat dan nama terbatas lulus. MP4 audio tidak didekode. |
| Recent/Clear | Lulus alur terpilih | Riwayat nyata maksimal 32, seleksi memvalidasi ulang; tes Clear/Cancel tidak menghapus berkas atau menambah input lulus. Berkas dalam screenshot adalah fixture prosedural CI. |
| Number dan komit | Lulus alur terpilih | Typed `AddInputAt`; tes PNG/WAV membuktikan posisi sisip. Penambahan/validasi tidak menukar identitas Program/Preview. Number default RustMix adalah posisi terakhir+1; referensi yang diamati memperlihatkan 1, sehingga default keduanya tidak dinyatakan sama. |
| Cancel/Escape | Lulus alur terpilih | Tes popup terlebih dahulu, topmost, tombol ditahan/modifier, pembatalan probe dan pelestarian transisi lulus. |
| Input audio | Lulus descriptor terbatas | Fixture WAV divalidasi tanpa mulai monitoring; metadata mencatat `audio_monitoring_started: false`. Menambah mikrofon tidak membuka perangkat. Ini bukan bukti suara, callback perangkat atau kesamaan formulir. |

## Temuan yang perlu diperbaiki

### ADD-UX-01 — draft PNG dapat terbawa ke kategori lain (prioritas tinggi)

**Dikonfirmasi melalui alur sumber; belum direproduksi lewat klik native baru.**
Setelah PNG siap, `poll_media` mengisi `add_name` dengan path dan menyimpan
`add_image`. Klik Colour atau Title hanya mengubah `add_tab`/`add_kind`.
`request_add_input` kemudian memutuskan jenis impor dari isi path, dan dapat
memanggil `commit_png_input` walaupun kategori aktif sudah berubah.

Skenario verifikasi perbaikan: Image → Browse PNG prosedural → tunggu ready →
Colour → OK. Ulangi untuk Title dan generator. Hasil yang diharapkan: kategori
aktif menentukan kontrak sumber; PNG sebelumnya tidak boleh diam-diam menjadi
input Colour/Title. Program/Preview harus tetap sama sampai operator melakukan
Take. Tidak ditemukan jalur ini langsung memotong Program saat Add.

Bukti: `crates/ui-egui/src/inputs.rs:291`, `dialogs.rs:552-555`,
`inputs.rs:87`, `inputs.rs:114-118`. Pisahkan draft perjenis atau bersihkan hanya
state yang tidak kompatibel saat berpindah kategori, lalu tambahkan regresi
lintas kategori. Tes quick-menu yang ada memeriksa tab dan pembatalan, bukan
kontrak sumber yang dikomit setelah berpindah dari PNG.

### ADD-UX-02 — formulir Image masih bercampur dengan generator (prioritas sedang)

Image referensi berisi path/Browse, daftar berkas dan Clear. Halaman PNG RustMix
masih memuat daftar tujuh generator, input gabungan nama/path, preview, serta
opsi interlaced nonaktif. Memilih sampel generator bahkan mengubah kategori
aktif. Browse PNG juga muncul pada semua halaman generator.

Bukti: `dialogs.rs:576-690`, `dialogs.rs:622-636`; referensi
`23i-add-input-image.jpg`. Buat halaman Image tersendiri dengan hanya format
yang benar-benar didukung, path, status/preview dan Clear yang jelas. Dukungan
JPG/BMP serta daftar recent Image belum tersedia dan tidak boleh diklaim ada.
Setelah mengganti nama/path PNG, jangan tampilkan preview berkas lama atau
izinkan komit berkas yang berbeda tanpa validasi.

### ADD-UX-03 — Colour dan Title/GT tidak menawarkan alur sesuai nama (prioritas sedang)

Colour referensi mempunyai pilihan warna, bars dan transparent; RustMix membuka
generator umum dan menghasilkan `InputKind::Color` yang berlabel Colour gradient.
Tidak ada pemilih warna pada Add Input tersebut. Title / XAML dan GT aktif tetapi
masing-masing memakai generator Graphics/Countdown, tanpa katalog template,
editor judul, pembacaan XAML/GT atau preview template yang dipilih.

Bukti: `dialogs.rs:210-331`, `dialogs.rs:576-690`; referensi
`23l-add-input-colour.jpg`, `23o-add-input-title.jpg`. Gunakan label yang menyatakan
generator yang sebenarnya sampai alur khusus diimplementasikan. Kelompokkan
sampel prosedural pada Generators. Template yang ditambahkan kelak harus asli
atau berlisensi sesuai kebijakan; jangan menyalin template vendor.

### ADD-UX-04 — katalog dan akses kategori belum sesuai (prioritas sedang)

DVD, Camera, NDI/OMT/Desktop Capture, Video Delay dan Telestrator tidak ada pada
katalog RustMix. Tambahan sampel generator terselip di antaranya. Referensi
mengelompokkan protokol pada Stream/SRT; RustMix hanya memiliki baris SRT nonaktif.
Kategori lain yang belum didukung juga dinonaktifkan sepenuhnya, sehingga tidak
dapat membuka halaman penjelasan persyaratan seperti referensi Zoom/Telestrator.

Bukti: `dialogs.rs:210-331`, `dialogs.rs:531-555`; bandingkan daftar kategori
di bagian berikut. Penonaktifan mencegah klaim backend palsu dan harus tetap
dijaga. Jika navigasi kategori dibuat tersedia untuk inspeksi, tampilkan status
Unavailable/alasan spesifik, jangan menambah sumber simulasi; OK tetap nonaktif.
Jangan mengaktifkan kamera, URL jaringan atau pemasangan SDK hanya demi audit UI.

### ADD-UX-05 — kesiapan OK dan pola formulir tidak konsisten (prioritas sedang)

Video memakai `video_add_ready`; WAV membuat OK aktif walaupun kosong, belum
divalidasi atau validasi berjalan. Handler WAV menolak komit dengan benar, jadi
ini masalah umpan balik dan konsistensi, bukan bukti WAV invalid diterima.
Tombol OK juga tidak mencerminkan kapasitas 32 input/prekondisi engine; kegagalan
baru dilaporkan ketika diklik. Form Audio memakai heading dan field bertumpuk,
berbeda dengan strip instruksi + path/list pada referensi Audio.

Bukti: `dialogs.rs:707-713`, `audio_controls.rs:291-391`,
`audio_controls.rs:394-407`, `inputs.rs:102-113`; referensi
`23m-add-input-audio.jpg`, `23n-add-input-audio-input.jpg`. Samakan indikator
kesiapan dengan validasi descriptor dan prekondisi engine; tampilkan alasan
singkat dekat field/footer. Pertahankan validasi ulang saat komit. Jangan
menyamakan perangkat default terkonfigurasi dengan perangkat yang sudah diuji.

### ADD-UX-06 — ikon tidak menyatakan jenis sumber (prioritas sedang)

Ikon dibagi dengan `idx % 6`, sehingga Video memakai tiga garis, Audio memakai
kotak, Audio Input memakai lingkaran/disc dan GT memakai kamera. Pola ini terlihat
di PNG native; bukan sekadar beda gaya dari referensi. Operator harus membaca
ulang label untuk mengenali sumber.

Bukti: `dialogs.rs:103-192`, khususnya baris 129. Gunakan ikon vektor asli yang
dipetakan secara eksplisit ke jenis sumber: film, image, colour, mikrofon,
speaker, text, network, dan seterusnya. Jangan menyalin artwork referensi.

## Matriks kategori referensi

Status berikut menilai kesesuaian **Add Input**, bukan kematangan produk. Semua
baris berplatform Windows x64; tidak ada hardware baru yang diuji. Tanggal
verifikasi 2026-10-21; owner/task `ADD-UX-AUDIT-2026-10-21`.

| Kategori yang terlihat pada referensi | RustMix saat audit | Status/keterbatasan | Bukti utama |
|---|---|---|---|
| Video | Video file | Experimental; alur MP4 video-only teruji, struktur utama diterima; alpha/interlaced dan format lain belum didukung | `23-add-input-video.jpg`; tiga PNG native; `video_controls.rs` |
| DVD | Tidak ada | Not started dalam Add Input | `23a`; `ADD_CATEGORIES` |
| List | Baris nonaktif | Stub navigasi; tidak ada daftar berkas/Shuffle pada Add Input | `23b`; `ADD_CATEGORIES` |
| Camera | Tidak ada; Moving shapes adalah generator terpisah | Not started; generator tidak membuktikan kamera | `23c`; `ADD_CATEGORIES` |
| NDI/OMT/Desktop Capture | Tidak ada; Studio pattern adalah generator | Not started; tidak ada discovery atau local-desktop form | `23d`, `23d1`; `ADD_CATEGORIES` |
| Stream/SRT | SRT nonaktif | Stub navigasi; tidak ada RTSP/TS/VLC/SRT form | `23e`–`23ea`; `ADD_CATEGORIES` |
| Instant Replay | Baris nonaktif | Stub navigasi | `23f`; `ADD_CATEGORIES` |
| Image Sequence/Stinger | Baris nonaktif | Stub navigasi | `23g`; `ADD_CATEGORIES` |
| Video Delay | Tidak ada; Reference bars adalah generator | Not started | `23h`; `ADD_CATEGORIES` |
| Image | PNG Image | Experimental; impor PNG teruji, formulir belum sesuai; hanya PNG | `23i`; `dialogs.rs`; tes PNG |
| Photos | Baris nonaktif | Stub navigasi; tidak ada folder/slideshow form | `23j`; `ADD_CATEGORIES` |
| PowerPoint | Baris nonaktif | Stub navigasi | `23k`; `ADD_CATEGORIES` |
| Colour | Colour / generator Color | Experimental generator; belum ada Colour/Bars/Transparent selector pada Add | `23l`; `dialogs.rs` |
| Audio | Audio / WAV | Experimental descriptor teruji; formulir dan recent belum sesuai; bukan MP3/semua WAV | `23m`; `audio_controls.rs`; native WAV JSON |
| Audio Input | Microphone descriptor | Experimental; device selector/Refresh, tanpa format/headphone form setara; capture fisik belum teruji | `23n`; `audio_controls.rs` |
| Title | Title / XAML dan GT | Experimental generator Graphics/Countdown; tidak ada loader/template gallery setara | `23o`; `dialogs.rs` |
| Virtual Set | Baris nonaktif | Stub navigasi; screenshot `23p` sendiri ambigu | `23p`; `ADD_CATEGORIES` |
| Web Browser | Baris nonaktif | Stub navigasi; URL/raster/CSS form tidak ada | `23q`; `ADD_CATEGORIES` |
| Video Call | Baris nonaktif | Stub navigasi; tidak ada host/connect/return-feed form | `23r`, `23t`; `ADD_CATEGORIES` |
| Zoom | Baris nonaktif | Stub navigasi; tidak ada halaman dependensi khusus | `23u`, `23v`; `ADD_CATEGORIES` |
| Telestrator | Tidak ada | Not started; tidak ada halaman persyaratan | `23w`; `ADD_CATEGORIES` |

## Gerbang penerimaan perbaikan berikutnya

1. Selesaikan ADD-UX-01 terlebih dahulu dengan fixture PNG asli/prosedural dan
   klik nyata perpindahan PNG → Colour/Title/generator → OK. Verifikasi descriptor,
   tidak ada reuse gambar salah, dan Program/Preview tetap sama; tambahkan kasus
   generator dengan nama `Studio A.1` agar titik pada nama tidak dianggap file.
2. Image, Colour dan generator mempunyai kontrak/form yang jelas. Title tidak
   menyiratkan dukungan XAML/GT sebelum tersedia. Tambahkan screenshot native
   tiap halaman terpilih, bukan hanya keberadaan ID field/tab.
3. WAV kosong, corrupt, sedang diperiksa, kapasitas penuh dan penolakan engine
   mempunyai kesiapan OK/alasan inline yang konsisten; Cancel tetap tersedia.
4. Katalog dipetakan sengaja, kategori belum didukung tidak pernah mengomit
   dummy input. Ikon asli mengikuti makna sumber; uji label panjang, scroll
   sidebar, Tab/Enter/Space, tooltip dan pembacaan status melalui aksesibilitas.
5. Jalankan tes terkait, check/Clippy dan native screenshot di hosted Windows
   pada revisi perbaikan yang persis sama. Pertahankan tes Cancel/Escape, late
   result, posisi sisip dan isolasi Program/Preview. Bukti hijau lama tidak
   menerima kode perbaikan baru. DPI/perangkat fisik tetap perlu sesi tersendiri.

Audit selesai; keenam temuan belum diperbaiki. Tidak ada klaim kesetaraan penuh,
persentase parity, kesiapan siaran, kinerja perangkat atau dukungan backend baru.

Pemeriksaan lokal sesi ini: `./scripts/check-repository.ps1` keluar 0 (layer,
asset dan inventaris referensi lulus; full reference match tetap false),
`git diff --check` keluar 0, dan perbandingan runtime terhadap 56048cb keluar 0.
Tidak ada perubahan Rust, dependency atau workflow. Perubahan sesi ini hanya
laporan ini serta rujukan singkat pada gaps, progress, roadmap dan feature matrix.
