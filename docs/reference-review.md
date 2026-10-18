# Review seluruh dokumentasi referensi

2026-10-18 · UI-013. Seluruh17 dokumen referensi dan28 dokumen proyek yang sudah ada dibaca ulang; tidak ada yang tidak dapat diakses. ADR0011 dan laporan ini ditambahkan sesudah review. Teks referensi diperlakukan sebagai bukti, bukan instruksi untuk menjalankan tool/probe. Tidak ada ekstraksi binary, penyalinan source/aset vendor, percobaan stream, atau perubahan aplikasi referensi.

## Urutan bukti dan koreksi

Catatan uji yang lebih baru mengoreksi ringkasan lama. `trace-log.md` T9/T10/T11 menjadi dasar kontrak wire, sedangkan UI-UX/FEATURE-MAP membuktikan inventaris dan observasi UI. Enum/metadata bukan bukti backend berjalan.

| Temuan | Ringkasan lama yang keliru/tidak lengkap | Keputusan yang dipakai |
|---|---|---|
|777 identifier | Disebut seluruh fungsi publik HTTP | Kosakata shortcut. T11 membuktikan beberapa nama ditolak HTTP dan Cut/Fade diterima tanpa konstanta shortcut terpisah. Tidak ada gate777/777 HTTP. |
| Overlay Off/Out | Out mengosongkan assignment | T9: Off mengosongkan; Out mempertahankan assignment. Efek raster Out belum dibuktikan. |
| Kanal overlay |16 kanal wire | T9/T11:1–8 wire; state16 terdiri dari8 standar+8 stinger. |
| Respons kontrol | Dianggap state hasil aksi | T10: state sebelum aksi, lalu baca terpisah untuk hasil. |
| Output2/3/4 | Dianggap pasti pemetaan fisik output | T11 menandai pemetaan fisik belum terjawab; jangan menebak. |
| Efek20/input40/properti256 | Dianggap semua implementasi berjalan | Inventaris nama; geometri8 efek, codec/SDK/perangkat dan banyak properti belum terverifikasi. |
| Clear Overlays | Di antara setiap item playlist | Panduan playlist publik: saat mulai. ADR0009 mempertahankan keputusan ini. |
| T-Bar | Fade tetap | Panduan Transitions publik: efek pertama. Gesture RustMix sekarang mengikuti slot pertama dan mem-pin konfigurasi. |

## Perbaikan sumber UI-013

Kategori kini assignment persisten, bukan generator-kind filter. Dialog Search mempunyai draft OK/Cancel. Tema/warna dan22 kategori diedit melalui Settings; nama/preferensi diterapkan satu transaksi. Audio Mixer benar-benar dapat disembunyikan/dipulihkan. Snapshot melakukan encode/tulis di worker dan tidak melaporkan sukses tanpa file. Kontrol overlay/Display/output yang belum berfungsi menjadi read-only/tidak tersedia; alamat Web Controller dan klaim routing1080p palsu dihapus. Lock secara eksplisit mencakup shortcut produksi dan drag tab, bukan pointer switching.

Regresi engine/widget/worker serta capture native baru dipersiapkan. Semua build/tes Rust tetap hanya di GitHub Windows; hasil runtime UI-013 dicatat pada ui-progress.md sesudah benar-benar selesai. Dukungan media/audio/encoder/replay/API/hardware belum dianggap selesai oleh perubahan ini.

## Inventaris dokumen referensi yang dibaca

Hash SHA256 berikut mengidentifikasi snapshot dokumen, bukan binary atau aset vendor.

| Dokumen | SHA256 |
|---|---|
| `README.md` | `d7f705ca13f2dd0d891c0c8afdee70220ddd963ec72215f8992c56ad4d6d83c0` |
| `FEATURE-MAP.md` | `b3e385517b978b93748672c3edb8151bafd18b13c67b1857848487fcebb60d4b` |
| `SPEC.md` | `cf1cc7caf437de007e17f9ee7e98a82168781a64f3b200d89a01b99457152613` |
| `UI-UX.md` | `7d112f6ee0fc31a3042448efb914d494716970bdfdaa479479769c26376e64b8` |
| `ARCHITECTURE.md` | `b3adfb25ecc4de33600a1bc1e1803a08da1e98647dac2ab145c7102adc43e99f` |
| `REA-TASK.md` | `e401614dddc1e1b7b12a894a268f51d325a9187886c7acad277470dc8bc08667` |
| `task.md` | `b38476c09c3c5e3ec3bd064602cbd0b3f99416d2ee82660b5c63a2c696151d55` |
| `evidence/README.md` | `048346b656d96885855af07def375b1eb51ae1d6c4eb37bcc3746a73f2722ca4` |
| `evidence/api-report.md` | `44632a29df4c1fb883bdf08ed0fe025138e7ddfd34f5ae7ce045a52b5fdbf423` |
| `evidence/config-data-report.md` | `7cfe3d1adbf7680194caae2434fdbac7beeb8ea732100e4daba397bbf9b74f90` |
| `evidence/enums.md` | `b6441d10fd49cfbe32d1846630c056cad41dfe6b9d61303c2c3aab6725eff151` |
| `evidence/feature-enums-rea.md` | `ba0ca0a8f3fa309cb19d83cc198d99da66d6111f03d942d15184f50726c613e6` |
| `evidence/shortcut-function-list.md` | `b4aea37f96dac881bf7b55eeb9f0388a1ceb87faeb89a4f6be74c83c93e13c25` |
| `evidence/shortcut-functions-categorised.md` | `379cfd07783db092af185e9b0ea9c655e1a4edcee1a24624372d1fbe0797dc50` |
| `evidence/summary-frmmain.md` | `611ecd9f64b5d8367b84f5501ec2d024bd186e157bcea6ef47bef28f6d7f4b3a` |
| `evidence/summary-mysettings.md` | `b66fb942992a202e9574c28c6fd3bba069810e5878659c6f0145fc3d6a6580b5` |
| `evidence/trace-log.md` | `34331abf7bc79e481fd51f832e8c04b88f7a256d4366cd13d9f99f547e52b9ad` |

## Dokumentasi proyek yang ditinjau

Root: AGENTS.md, README.md, ROADMAP.md, ATTRIBUTION.md. App/crate: apps/rustmix/README.md, crates/engine/README.md, crates/ui-egui/README.md. Docs: architecture, engine-audit, feature-matrix, gaps, licensing, reference-audit, reference-feature-comparison, testing, ui-architecture, ui-progress, ui-reference; ADR0001–0010.

Keterangan historis source-only/pending, schema2, audio Properties langsung, T-Bar vertikal dan mekanik monitor bank lama diperbaiki atau ditandai superseded. Catatan UI-008 yang pernah mengklaim GPU/rekaman/streaming diberi peringatan langsung sebagai klaim yang ditolak audit UI-009, dengan riwayat tetap dipertahankan.
