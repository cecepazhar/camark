# CAMark — High-Performance Zero-Knowledge Markdown & Knowledge Studio

[![Release](https://img.shields.io/badge/release-v1.0.17-06b6d4.svg?style=flat-square)](https://github.com/cecepazhar/camark)
[![Rust](https://img.shields.io/badge/rust-2024%20(1.85%2B)-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/tauri-v2.0-24C8DB.svg?style=flat-square&logo=tauri)](https://v2.tauri.app)
[![Svelte](https://img.shields.io/badge/svelte-5%20(Runes)-FF3E00.svg?style=flat-square&logo=svelte)](https://svelte.dev)
[![UI System](https://img.shields.io/badge/design%20system-%40cecepazhar%2Fcaui-8b5cf6.svg?style=flat-square)](https://github.com/cecepazhar/caui)
[![Security](https://img.shields.io/badge/security-SQLCipher%204%20%2B%20Argon2id-10b981.svg?style=flat-square&logo=sqlite)](https://www.zetetic.net/sqlcipher/)
[![Editor](https://img.shields.io/badge/editor-CodeMirror%206-yellow.svg?style=flat-square&logo=codemirror)](https://codemirror.net)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows%20%7C%20macOS%20%7C%20Android-green.svg?style=flat-square)]()

> **Sovereign, Local-First, Zero-Knowledge Markdown & Code Studio** dibangun untuk performa ekstrem (<35 MB RAM, cold-start <1.0 detik), rendering 60 FPS DMABUF tanpa lag, brankas terenkripsi SQLCipher + Argon2id, serta integrasi AI Co-Pilot lokal dan cloud.

---

## ⚡ Tolok Ukur Kinerja Nyata (Real Benchmarks)

Diuji pada mesin Linux (Fedora 44 x86_64, Kernel 6.19, ThinkPad X1):

| Metrik Kinerja | CAMark (v1.0.17) | Typora (Electron) | Obsidian (Electron) | Notion (Web/Electron) |
|---|---|---|---|---|
| **Cold-Start Launch** | **< 1.0 detik** | ~2.5 detik | ~3.8 detik | > 5.0 detik |
| **Konsumsi RAM (Idle)** | **< 35 MB** | ~95 MB | ~180 MB | > 350 MB |
| **Konsumsi RAM (Doc 10k baris)** | **< 48 MB** | ~140 MB | ~290 MB | > 500 MB |
| **Render Engine & Frame Rate** | **60 FPS (DMABUF Native)** | 40-50 FPS | 45-55 FPS | 30 FPS |
| **Sinkronisasi Scroll** | **Zero-lag pixel sync** | Halus | Cukup | Sering desinkronisasi |
| **Ukuran Binary Rilis** | **~18 MB** | ~85 MB | ~110 MB | ~130 MB |

---

## 🌟 Fitur Unggulan

### 1. Dual-Storage & Multi-Format Workspace Engine
- **Open Filesystem Mode**: Buka, jelajah, dan edit berkas Markdown maupun kode sumber langsung dari filesystem lokal. Tanpa format lock-in, 100% kompatibel dengan Obsidian, VS Code, Git, dan Logseq.
- **Zero-Knowledge Encrypted Vault**: Simpan catatan rahasia dan dokumen sensitif ke dalam brankas terenkripsi **SQLCipher 4 + Argon2id** (dilindungi Master Password & PIN profil, key derivation in-memory tanpa kebocoran disk).
- **Multi-Tab Document Workspace**: Bilah tab multi-dokumen dengan tab context menu (*Close*, *Close Others*, *Close to Right*), state auto-save reaktif, dan indikator penyimpanan status bar.
- **Sistem Reveal Berkas Native**: Integrasi satu klik untuk membuka dan menyorot berkas di manajer berkas OS (Nautilus, Dolphin, Explorer, Finder).

### 2. Editor Presisi & Live Preview 60 FPS
- **CodeMirror 6 Editor**: Syntax highlighting dinamis, line wrapping cerdas, dan auto-indentation.
- **Dukungan Sintaks Multi-Bahasa (v1.0.17)**: Deteksi otomatis sintaks untuk Markdown, Rust, Python, JavaScript, TypeScript, JSON, HTML, CSS, dan puluhan bahasa pemrograman lainnya.
- **Matematika KaTeX & Diagram Mermaid**: Render instan rumus matematika LaTeX (`$...$`, `$$...$$`) serta diagram alur Mermaid.js secara real-time.
- **Auto Table Formatter & Smart Paste**: Konversi instan tabel terformat (`|`) dan konversi otomatis data tabel clipboard Excel/HTML menjadi tabel Markdown bersih.
- **3 Mode Tampilan**: *Split View* (default), *Zen Mode* (Typewriter scrolling berfokus pada baris aktif), dan *Reading Preview Mode*.
- **Marp Presentation Mode**: Konversi berkas Markdown menjadi slide presentasi layar penuh dengan navigasi keyboard (`PageDown`, `PageUp`, panah).
- **Outline Navigator (TOC)**: Ekstraksi hierarki heading dokumen (H1–H6) secara langsung dengan *smooth anchor scrolling*.

### 3. Privacy-First AI Writing Co-Pilot
- **Local AI & Cloud BYO-Key**: Koneksi native ke Ollama lokal, LM Studio, OpenAI API, atau proxy lokal 9Router (`http://127.0.0.1:20128/v1`).
- **Contextual Action Bar**: Perbaiki tata bahasa, humanize tone, buat ringkasan, perbaiki formatting tabel, atau terjemahkan teks terpilih.
- **Automated Privacy Scrubber**: Penyaringan regex otomatis terhadap data pribadi (email, token, kunci API, nomor telepon) sebelum prompt diteruskan ke model AI.

### 4. Ekspor Standar Profesional
- **Native Print & PDF Export**: Ekspor dokumen menjadi PDF bersih siap cetak dengan penomoran halaman dan formatting tabel zebra.
- **Standalone HTML & Rich Text**: Salin langsung sebagai Rich Text terformat ke email atau Microsoft Word.

---

## 📂 Format Berkas yang Didukung

CAMark v1.0.17 mendukung pembacaan, pengeditan, dan render pratinjau untuk format teks dan kode berikut:

| Kategori | Ekstensi & Tipe Berkas |
|---|---|
| **Markdown & Dokumentasi** | `.md`, `.markdown`, `.mdown`, `.mkd`, `.txt`, `.text`, `.org`, `.rst`, `.adoc`, `.asciidoc` |
| **Bahasa Pemrograman & Sistem** | `.rs` (Rust), `.py`, `.pyw` (Python), `.go` (Go), `.c`, `.cpp`, `.cc`, `.cxx`, `.h`, `.hpp`, `.hxx` (C/C++), `.sh`, `.bash`, `.zsh`, `.fish` (Shell), `.sql` (SQL), `.java` (Java), `.php` (PHP), `.rb` (Ruby), `.lua` (Lua), `.r` (R), `.swift` (Swift), `.kt`, `.kts` (Kotlin), `.dart` (Dart), `.zig` (Zig), `.ex`, `.exs` (Elixir), `.erl` (Erlang), `.clj` (Clojure) |
| **Web & Frontend** | `.html`, `.htm`, `.css`, `.scss`, `.sass`, `.less`, `.js`, `.mjs`, `.cjs` (JavaScript), `.ts`, `.mts`, `.cts` (TypeScript), `.jsx`, `.tsx`, `.svelte` (Svelte 5), `.vue`, `.svg` |
| **Konfigurasi & Data** | `.json`, `.jsonc`, `.json5`, `.yaml`, `.yml`, `.toml`, `.xml`, `.csv`, `.tsv`, `.ini`, `.conf`, `.cfg`, `.env`, `.env.*`, `.properties` |
| **DevOps & Berkas Spesial** | `Dockerfile`, `Makefile`, `CMakeLists.txt`, `.diff`, `.patch`, `.log`, `.gitignore`, `.gitattributes`, `.editorconfig`, `.dockerignore`, `.bashrc`, `.zshrc`, `LICENSE`, `README`, `CHANGELOG` |

*Catatan: Berkas kode sumber non-markdown ditampilkan dalam preview terformat dengan syntax highlighting dan tombol salin satu-klik.*

---

## ⌨️ Pintasan Papan Ketik (Keyboard Shortcuts)

| Pintasan | Lingkup | Fungsi |
|---|---|---|
| <kbd>Ctrl</kbd> + <kbd>K</kbd> / <kbd>Cmd</kbd> + <kbd>K</kbd> | Global | Buka **Command Palette** & pencarian global |
| <kbd>Ctrl</kbd> + <kbd>B</kbd> / <kbd>Cmd</kbd> + <kbd>B</kbd> | Global | Tampilkan / Sembunyikan panel **Berkas & Folder (Sidebar)** |
| <kbd>Ctrl</kbd> + <kbd>P</kbd> / <kbd>Cmd</kbd> + <kbd>P</kbd> | Global | **Cetak / Ekspor Dokumen ke PDF** |
| <kbd>Ctrl</kbd> + <kbd>W</kbd> / <kbd>Cmd</kbd> + <kbd>W</kbd> | Global | **Tutup Tab Dokumen Aktif** |
| <kbd>Esc</kbd> | Global | Tutup dialog, modal, drawer AI, atau keluar Zen Mode |
| <kbd>Ctrl</kbd> + <kbd>Z</kbd> | Editor | Undo (Urungkan perubahan) |
| <kbd>Ctrl</kbd> + <kbd>Y</kbd> / <kbd>Cmd</kbd> + <kbd>Shift</kbd> + <kbd>Z</kbd> | Editor | Redo (Ulangi perubahan) |
| <kbd>Ctrl</kbd> + <kbd>F</kbd> | Editor | Cari teks di dalam editor |
| <kbd>Ctrl</kbd> + <kbd>H</kbd> | Editor | Ganti teks di dalam editor |
| <kbd>Ctrl</kbd> + <kbd>B</kbd> (pada seleksi) | Editor | Format teks menjadi **Tebal (Bold)** |
| <kbd>Ctrl</kbd> + <kbd>I</kbd> (pada seleksi) | Editor | Format teks menjadi *Miring (Italic)* |
| <kbd>→</kbd> / <kbd>Space</kbd> / <kbd>PageDown</kbd> | Presentasi | Pindah ke **Slide Berikutnya** (Marp Mode) |
| <kbd>←</kbd> / <kbd>PageUp</kbd> | Presentasi | Pindah ke **Slide Sebelumnya** (Marp Mode) |
| <kbd>Home</kbd> / <kbd>End</kbd> | Presentasi | Lompat ke **Slide Pertama / Terakhir** |

---

## 🔒 Arsitektur Zero-Knowledge Encrypted Vault

Keamanan data catatan di brankas terenkripsi CAMark dibangun tanpa kompromi (*zero-knowledge*, *local-only*). Kunci enkripsi tidak pernah disimpan dalam bentuk plaintext di disk atau dikirim ke jaringan mana pun.

```text
+-------------------------------------------------------------------------+
|                              PENGGUNA                                    |
|   Input: Master Password (>= 8 karakter) / PIN Profil (Argon2id)        |
+-------------------------------------------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                  ARGON2ID KEY DERIVATION ENGINE                         |
|   - Algorithm: Argon2id v1.3                                            |
|   - Parameters: Memory Cost = 64 MB (65,536 KiB), Time = 3, Parallel = 4 |
|   - Salt: 16 Bytes CSPRNG random salt (disimpan di vault_salt.bin)      |
|   - Output: 256-bit (32 bytes) Master Key                               |
+-------------------------------------------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                        IN-MEMORY KEY MANAGEMENT                         |
|   - Disimpan dalam parking_lot::RwLock<Option<[u8; 32]>>                |
|   - Zeroize on Lock / Drop: Memori di-overwrite nol saat terkunci       |
|   - Verifikasi Canary: AES-256-GCM decrypt pada vault_canary.bin        |
+-------------------------------------------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                  SQLCIPHER 4 ENCRYPTED STORAGE LAYER                    |
|   - Database: camark.db                                                 |
|   - Cipher: AES-256-CBC dengan HMAC-SHA512 per page checksum            |
|   - Kunci database di-injeksi via: PRAGMA key = "x'<derived_hex>'";     |
|   - Ubah password aman via: PRAGMA rekey = "x'<new_hex>'";              |
|   - File disk terlindung penuh (Plain-header guard: non-SQLite header)  |
+-------------------------------------------------------------------------+
```

---

## 🏗️ Arsitektur Teknologi

```text
+-------------------------------------------------------------------+
|               Frontend UI (@cecepazhar/caui + Svelte 5)            |
|  - Svelte 5 Runes ($state, $derived, $effect)                     |
|  - CodeMirror 6 Editor dengan multi-bahasa language compartments  |
|  - Live Preview 60 FPS (DOMPurify, KaTeX, Mermaid.js, Marked)     |
|  - CADS Obsidian Theme Tokens (#0A0A0C, Cyan Accent, Ambient Halo)|
+-------------------------------------------------------------------+
                                 |  Tauri IPC Commands
+-------------------------------------------------------------------+
|                     Tauri v2 Native Desktop Layer                 |
|  - Custom Frame Titlebar & Multi-window lifecycle                 |
|  - Native Dialogs, OS File Associations, Single-Instance Handler  |
+-------------------------------------------------------------------+
                                 |
+-------------------------------------------------------------------+
|                         Rust 2024 Core                            |
|  - Filesystem Workspace Engine (fs_workspace.rs, Nautilus reveal) |
|  - Zero-Knowledge Vault & SQLCipher 4 Engine (vault.rs, db.rs)    |
|  - AI Proxy Engine (9Router / Ollama / Cloud BYO-Key)             |
|  - Exporter Engine (Headless WebKit Print to PDF)                 |
|  - Pro Licensing Client (Ed25519 signature offline verification)  |
+-------------------------------------------------------------------+
```

---

## 🚀 Panduan Memulai

### Kebutuhan Sistem
- **Rust**: Versi 1.85+ (Edition 2024)
- **Node.js**: Versi 20+ & npm
- **Sistem Operasi**: Linux (Fedora, Debian/Ubuntu, Arch), Windows 10/11, atau macOS 12+

### Menjalankan Mode Development
```bash
# Clone atau buka direktori proyek
cd ~/Product/camark

# Jalankan dalam mode development Tauri
cargo tauri dev
```

### Membangun Paket Rilis
```bash
# Kompilasi binary rilis berkecepatan tinggi
cargo tauri build
```
Hasil binary executable akan berada di `target/release/camark`.

---

## 📄 Lisensi
Didistribusikan di bawah lisensi MIT. Hak Cipta © 2026 Fathforce / Cecep Saeful Azhar Hidayat.
Situs resmi: [https://cecepazhar.com](https://cecepazhar.com) / [https://camark.fathforce.com](https://camark.fathforce.com).
