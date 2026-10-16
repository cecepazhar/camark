# Changelog

Semua perubahan penting yang dilakukan pada CAMark akan didokumentasikan dalam file ini.
Format ini mengacu pada [Keep a Changelog](https://keepachangelog.com/id/), dan proyek ini mematuhi [Semantik Versioning](https://semver.org/lang/id/).

---

## [v1.0.17] - 2026-10-15
### Added
- **Dukungan File Teks & Kode Non-Markdown**: Pengguna kini dapat langsung membuka dan mengedit file teks, konfigurasi, dan kode sumber dari File Tree Workspace.
- **Dynamic Syntax Highlighting**: Mesin CodeMirror 6 kini memuat dan menonaktifkan extension compiler otomatis (compartment) untuk berbagai bahasa saat tab dokumen dipindah:
  - Rust (`.rs`)
  - Python (`.py`, `.pyw`)
  - JavaScript & TypeScript (`.js`, `.mjs`, `.cjs`, `.ts`, `.mts`, `.cts`, `.jsx`, `.tsx`)
  - JSON (`.json`, `.jsonc`, `.json5`)
  - HTML, Svelte, Vue (`.html`, `.htm`, `.svg`, `.svelte`, `.vue`)
  - CSS, Sass, Less (`.css`, `.scss`, `.sass`, `.less`)
- **Custom File Tree & Tab Icons**: Setiap tipe berkas kini menampilkan ikon identitas yang unik dan berwarna (mis. badge `RS` oranye, `PY` kuning, `JS` cyan, `{}` amber, `⚙` ungu, `<>` rose).
- **Expanded MIME & OS Associations**: Ekstensi file baru ditambahkan secara dinamis di sistem backend `fs_workspace.rs` dan terintegrasi penuh dengan sistem OS (Open With / Right-Click context menu).

---

## [v1.0.16] - 2026-10-15
### Changed / Fixed
- **Pro Feature Gating**: Semua elemen antarmuka Pro (Pilih Avatar Profil & Pro Ambient Glow) kini benar-benar disembunyikan kecuali untuk pelanggan aktif.
- **Removed Family Tab**: Menu "Family (Keluarga)" dihilangkan secara absolut dari bar navigasi atas untuk fokus pada penggunaan profesional, pribadi, dan tim enterprise.
- **License Sanitization**: Field placeholder pada Lisensi Pro yang berisi key dummy (`sk-pro-...`) telah dibersihkan.
- **Local AI Proxy Routing (9Router)**: Endpoint AI secara default diarahkan ke local proxy `9Router` (`http://127.0.0.1:20128/v1`) untuk mengoptimalkan performa dan keamanan data lokal Indonesia.

---

## [v1.0.15] - 2026-10-15
### Fixed
- **Database Re-key Bug (Critical)**: Ditemukan dan diperbaiki bug pada logika *rekey* (perubahan master password) brankas SQLCipher yang secara tak terdampak menyebabkan database terhapus/loss (data wiping).
- **Pro Status Load**: Status lisensi Pro Pro sekarang dimuat secara reaktif secara akurat ketika aplikasi baru dibuka.
- **Activation Button Contrast**: Memperbaiki masalah tombol *Activate License* yang terlihat terlalu samar/dimatikan pada tema Light (kontras).

---

## [v1.0.14] - 2026-10-14
### Added / Changed
- **Adopsi Design System `@cecepazhar/caui`**: Kamark telah sepenuhnya beralih menggunakan modul desain CADS & `@cecepazhar/caui` dengan kontrol modern, termasuk:
  - `CaToggleSwitch`
  - `CaInputBox`
  - `CaButton`
  - `CaModal`
  - `CaBadge`
- **Founder Lifetime Unlock Protocol**: Porting protokol *pro founder lifetime unlock* dari CATerm. Pelanggan Pro akan diberikan badge "Lifetime Unlock" dan akses penuh ke Pro Avatar, Ambient Glow (RGB Chroma Underglow, Outer Glow Work Area), serta Pro Features.

---

## [v1.0.13] - 2026-10-14
### Fixed
- **Vault Data Cache Sync**: Cache memori sementara (`vaultDocs`) pada sisi React kini ditetaskan secara reaktif ketika autosave terjadi pada latar belakang.
- **Tab Loading Refresh**: Memuat tab dokumen yang baru saja dibuka akan secara otomatis memuat *in-memory* cache vault terbaru.
- **Auto-Flush on Tab Switch**: Dokumen brankas akan dikunci & disinkronkan secara otomatis dan aman ke `camark.db` SQLCipher saat pengguna beralih antar tab dokumen.

---

## [v1.0.12] - 2026-10-14
### Fixed
- **Eliminasi Startup Flicker**: Perbaikan masalah kedipan UI pada saat aplikasi pertama kali dibuka.
- **Default Split Mode & Persisted View Mode**: Aplikasi secara default menggunakan Split Mode, dan preferensi layar pengguna akan dipertahankan dengan benar tanpa *force* preview saat *restart*.

---

## [v1.0.11] - 2026-10-14
### Added / Changed
- **Save Indicator Movement**: Status bar penyimpanan (`Saved` / `Unsaved`) dipindah ke pojok status bawah (footer bar) secara elegan.
- **Vault Master Password Lock Cycle**: Siklus *lock* & *unlock* brankas dengan Master Password berjalan mulus tanpa *hang*.
- **Sequential Files Naming**: Nomenklatur file tanpa nama secara otomatis menjadi `files-1.md`, `files-2.md`, dst.
- **Smooth TOC Anchor Scroll**: Peta hierarki dokumen (Outline) kini menggunakan fitur *smooth scrolling* untuk berpindah antar heading.

---

## [v1.0.10] - 2026-10-14
*(Internal milestone)*
### Fixed
- **Auto-save Data Persistence & Version Automation**: Perbaikan tag input pada `vault_save_document` agar tidak *drop*, dan penambahan script otomatis `bump-version.sh`.

---

## [v1.0.9] - 2026-10-14
### Removed / Added
- **Hilangkan Hana AI Top-Bar Button**: Tombol *Hana AI* dipindahkan dan disatukan ke *right-side drawer* dan Contextual Bar AI.
- **AI i18n Status Greeting**: Status *greeting* pada panel AI kini secara dinamis mengikuti *locale* yang dipilih pengguna (Indo/Inggris).

---

## [v1.0.8] - 2026-10-14
### Added / Fixed
- **100% EN/ID Translations**: Seluruh teks pada seluruh bagian aplikasi CAMark telah diterjemahkan ke dalam Bahasa Indonesia secara 100% lengkap.
- **Nautilus Select Reveal**: Integrasi satu-klik *Show in File Manager* untuk pengguna Linux GNOME/Nautilus (dan Explorer/Finder untuk platform lain).
- **Light Mode Contrast Fixes**: Perbaikan bug teks yang sulit dibaca pada tema Light Mode.
- **Eliminasi Blank Editor DOM Drop**: Perbaikan *crash* / hilang tampilan editor secara berkala saat ganti dokumen cepat.

---

## [v1.0.7] - 2026-10-14
*(Prestasi & Fitur)*
### Added
- **Custom Tab Context Menu**: Menu klik kanan pada bilah Tab Dokumen (`Close`, `Close Others`, `Close to Right`).
- **Reveal File in OS Manager**: Integrasi fitur bawaan untuk menyorot file pada *File Manager OS*.

---

## [v1.0.6] - 2026-10-14
*(Grand Unification Architecture & Performance)*
### Added
- **Unified Studio Header**: Desain *unified top bar* dengan bilah tab dokumen aktif yang sangat elegan & ringkas.
- **Right-Side Drawer AI Copilot**: Panel AI dipindahkan ke samping kanan sebagai *overlay* dengan animasi transisi halus.
- **Editor Performance Optimizations**: Pengoptimalan render live-preview & sync scrolling secara signifikan.

---

## [v1.0.5] - 2026-10-13
*(Prestasi & Fitur)*
### Added
- **AI Endpoint Testing & Streamline Editor**: Perbaikan *streamlining* pada *layout* editor.
- **Connection Testing UI**: Pengujian koneksi secara real-time pada *settings* AI API Endpoint OpenAI-compatible (Ollama, vLLM, 9Router).
- **Default Preview View Mode**: Dokumen yang baru saja dibuka secara default langsung ditampilkan dalam mode Live Preview.

---

## [v1.0.4] - 2026-10-13
*(Prestasi & Fitur)*
### Changed
- Optimasi kecepatan respons editor dan perbaikan asosiasi default file pada sistem operasi.

---

## [v1.0.3] - 2026-10-08
*(Prestasi & Fitur)*
### Added / Fixed
- **Open With Launch Crash Fix**: Ditemukan dan diperbaiki *crash* saat *double-click* / integrasi *Open With* dari *File Manager*.
- **Marp Presentation Mode**: Konversi instan Markdown menjadi Slide Presentasi Layar Penuh yang mendukung keyboard.
- **Document Outline Navigator (TOC)**: Panel navigasi hierarki heading dokumen (`H1` s.d. `H6`).
- **Pro Shell Integration**: Integrasi dasar *billing/pro* pada *shell* aplikasi CAMark.

---

## [v1.0.1] - 2026-10-08
*(Inisialisasi Rilis v1.0)*
### Added
- **Standar Rilis Desktop v1.0**: Penyesuaian nama binary mandiri `camark` dan pembersihan IPC shell.
- **Inisialisasi Peluncuran**: Pengaturan pratinjau dokumen default dan optimasi IPC single-instance.

---

## [v0.1.3] - 2026-10-08
*(Prestasi & Fitur)*
### Added
- **CATerm Pro Modern Visual Standard**: Penyegaran UI CAMark untuk mengikuti standar visual `Dark Obsidian`, aksen *Cyan*, serta *ambient RGB glow*.
- **Synchronous Live-Scroll Engine**: Sinkronisasi per-pixel antara Editor CodeMirror & Live Preview 60 FPS.
- **Table Auto-Formatter & Smart Paste**: Tooling untuk konversi data clipboard (Excel / HTML) menjadi tabel GFM Markdown.
- **Native PDF Print Export & Contextual AI Copilot**: Integrasi *printing* PDF & AI Contextual *action-bar* langsung di dokumen.

---

## [v0.1.2] - 2026-10-08
*(Prestasi & Fitur)*
### Added
- **Synchronous Live-Scroll Engine & Native PDF Print**.

---

## [v0.1.0-dev] - 2026-10-04
### Added
- Arsitektur multi-profil dengan *PIN unlock* & RBAC.
- *Sovereign Zero-Knowledge* Storage via SQLCipher 4 + Argon2id (key-wrapping 2 lapis, BIP-39 recovery 24 kata).
- UI Shell Svelte 5 Runes (dark/light theme, i18n ID/EN, command palette).
- Generik AI context scrubber & panel asisten.
- Sinkronisasi struktur tabel dengan UUIDv7 & audit `change_log` otomatis.
- `caf-xtask` developer tooling (`codegen`, `guard`, `new-app`).
