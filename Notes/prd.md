# CAMark — Product Requirements Document (PRD) & Business Plan

| Metadata | Keterangan |
|---|---|
| **Produk** | **CAMark** (High-Performance Zero-Knowledge Markdown & Knowledge Studio) |
| **Owner / Founder** | Fathforce / Cecep Saeful Azhar Hidayat, ST (`hi@cecepazhar.com`) |
| **Versi Dokumen** | **v2.0 (Comprehensive Release v1.0.17)** — Diperbarui 15 Oktober 2026 |
| **Status Produk** | Active Production (v1.0.17 Released) |
| **Repositori & Lisensi** | `~/Product/camark` · Lisensi MIT (Core) & Pro License (Commercial) |

---

## 1. Executive Summary & Visi Produk

**CAMark** adalah *high-performance, sovereign, local-first, zero-knowledge Markdown & Code Studio* yang menggabungkan kecepatan pengeditan instan ala Typora, fleksibilitas manajemen direktori ala Obsidian dan VS Code, serta keamanan brankas terenkripsi (*Encrypted Vault*) setingkat industri perbankan berbasis **SQLCipher 4** dan **Argon2id**.

Dibangun dengan arsitektur modern **Rust 2024**, **Tauri v2**, **Svelte 5 (Runes)**, dan design system **@cecepazhar/caui**, CAMark dirancang khusus untuk mengatasi masalah utama aplikasi berbasis Electron yang lambat, rakus memori (>200 MB), serta ketergantungan wajib pada server cloud yang mengorbankan privasi pengguna.

### Nilai Inti & Filosofi (Sovereign Local-First)
1. **Kedaulatan Data Penuh (Data Sovereignty)**: Seluruh dokumen Markdown dan kode sumber disimpan murni di filesystem lokal pengguna dalam format CommonMark / GFM standar tanpa format lock-in atau enkripsi vendor rahasia. Pengguna memiliki data mereka selamanya.
2. **Zero-Knowledge Architecture**: Dokumen rahasia disimpan dalam brankas lokal SQLCipher terenkripsi AES-256 dengan kunci yang diderivasi dari Master Password via Argon2id. Tidak ada pintu belakang (*backdoor*), dan kunci didekripsi murni di memori RAM yang langsung di-overwrite nol (*zeroize*) saat dikunci.
3. **Ekstrem Ringan & Cepat**: Cold-start <1.0 detik, konsumsi RAM <35 MB, dan rendering tampilan 60 FPS hardware-accelerated melalui pipeline DMABUF Linux/Wayland/X11, Windows WebView2, dan macOS WebKit.
4. **Privasi AI Tanpa Kompromi**: Integrasi AI penulisan lokal (Ollama / proxy lokal 9Router di `http://127.0.0.1:20128/v1`) atau Cloud Bring-Your-Own-Key (BYO-Key) yang dilengkapi *automated PII/regex scrubber* untuk menyaring data sensitif sebelum dikirim ke model bahasa.

---

## 2. Analisis Kompetitor & Value Proposition

### Matriks Perbandingan Kompetitor

| Parameter / Fitur | CAMark (v1.0.17) | Typora | Obsidian | Notion |
|---|---|---|---|---|
| **Model Penyimpanan** | **Open Filesystem + Brankas SQLCipher** | Open Filesystem murni | Open Filesystem murni | Cloud Database Proprietary |
| **Enkripsi Data Sensitif** | **SQLCipher 4 + Argon2id (Bawaan)** | Tidak ada | Berbayar (Obsidian Sync) | Cloud DB (Bukan E2EE murni) |
| **Konsumsi RAM (Idle / Working)**| **< 35 MB / < 48 MB** (Rust + Tauri v2) | ~80-120 MB (Electron) | ~150-250 MB (Electron) | > 300-600 MB (Electron/Web) |
| **Cold-Start Launch Time** | **< 1.0 detik** | ~2.5 detik | ~3.8 detik | > 5.0 detik |
| **Rendering & Frame Rate** | **60 FPS DMABUF Native** | ~45 FPS | ~50 FPS | ~30 FPS |
| **Dukungan File Non-Markdown** | **40+ Bahasa Kode & Format Konfigurasi** | Terbatas | Terbatas (Plugin) | Blok kode dalam database |
| **AI Writing Assistant** | **Local Ollama / 9Router / BYO-Key** | Tidak ada | Komunitas Plugin (Tidak aman) | Cloud Berbayar ($10/bulan) |
| **Mode Presentasi Layar Penuh** | **Marp Engine Terintegrasi** | Tidak ada | Perlu plugin pihak ketiga | Tidak ada |
| **Kedaulatan & Akses Offline** | **100% Offline & Sovereign** | Offline lokal | Offline lokal | Sangat terbatas / Terkunci Cloud |
| **Skema Lisensi** | **Core MIT Gratis / $29 Lifetime Pro** | $14.99 (Per 3 perangkat) | Gratis Personal / $50 Komersial | Berlangganan bulanan |

### Value Proposition Utama
- **Penulis & Insinyur**: Menulis dokumentasi teknis, melihat diagram Mermaid, rumus KaTeX, dan membuka berkas kode sumber tanpa jeda dan tanpa menghabiskan resource sistem.
- **Akademisi & Peneliti**: Menulis naskah akademik dengan sitasi dan KaTeX, lalu mengekspornya langsung ke PDF berstandar cetak atau Rich Text tanpa rusak format.
- **Eksekutif & Privacy Advocates**: Memiliki brankas catatan terenkripsi kuat yang tidak dapat diakses oleh pihak ketiga atau penyedia cloud mana pun.
- **Presenter & Edukator**: Mengubah catatan naskah Markdown menjadi slide presentasi visual layar penuh (Marp) dalam satu detik.

---

## 3. Target Pasar & Persona Pengguna

1. **Technical Writers, Pengembang Perangkat Lunak & SysAdmin**:
   - Mengelola dokumentasi proyek, catatan arsitektur sistem, skrip DevOps (`Dockerfile`, `Makefile`, `.toml`), dan snippet kode. Membutuhkan editor yang instan, ringan, dan tidak lemot saat membuka direktori proyek besar.
2. **Peneliti, Dosen & Mahasiswa Pascasarjana (S2/S3)**:
   - Menyusun draf tesis, jurnal ilmiah, dan persamaan matematika kompleks menggunakan sintaks LaTeX/KaTeX. Membutuhkan ekspor PDF berkualitas tinggi dan layout bersih.
3. **Eksekutif, Advokat & Praktisi Keamanan (Privacy Advocates)**:
   - Menyimpan strategi rahasia, audit hukum, credential notes, dan jurnal pribadi. Sangat menolak platform catatan cloud komersial yang melakukan telemetri data atau AI scraping pada dokumen pengguna.
4. **Content Creators, Pembicara Publik & Fasilitator**:
   - Menyiapkan materi webinar dan ceramah menggunakan Marp Slideshow bawaan, mode Zen Typewriter scrolling untuk menulis naskah konten tanpa distraksi visual.

---

## 4. Model Monetisasi & Strategi Penetapan Harga (Pricing Strategy)

CAMark menerapkan strategi **Fair Pricing, Freemium & Local-First Forever**: fungsi utama sebagai studio Markdown dan editor kode terbuka selamanya (*Core is Free Forever*). Fitur Pro ditujukan untuk fungsionalitas lanjutan, kustomisasi visual eksklusif, sinkronisasi terenkripsi, dan ekosistem enterprise.

### Tuas Monetisasi (Monetization Levers)

```text
+-----------------------------------------------------------------------------------------+
|                                TIER & SKEMA HARGA CAMARK                                |
+-----------------------------------------------------------------------------------------+
|                                                                                         |
|  1. FREE CORE TIER ($0 / Selamanya Bebas Biaya)                                         |
|     - Open filesystem workspace mode (tanpa batas file/folder).                         |
|     - CodeMirror 6 editor dengan dynamic syntax highlighting (Rust, Python, JS, dll).   |
|     - 60 FPS live-preview, KaTeX math rendering, & Mermaid.js diagram.                  |
|     - Table auto-align & Smart Paste (Excel/HTML to Markdown).                          |
|     - Marp presentation mode dasar.                                                     |
|     - Local AI via Ollama / 9Router (BYO API Key).                                      |
|     - Ekspor Standalone HTML & Rich Text formatting.                                    |
|                                                                                         |
|  2. PRO FOUNDER LIFETIME LICENSE ($29 Lifetime / Rp 299.000 — TUAS UTAMA)               |
|     * Tuas monetisasi primer: Lisensi sekali bayar seumur hidup, tanpa langganan wajib. *|
|     - Lisensi Pro Founder Lifetime berlaku seumur hidup untuk 3 perangkat.              |
|     - Akses penuh ke seluruh update versi mayor dan minor di masa depan.               |
|     - Multi-vault SQLCipher terenkripsi tanpa batas.                                    |
|     - Ekspor PDF Standar Cetak Akademik resolusi tinggi & styling kustom.               |
|     - Pro Theme Studio, Ambient Glow Engine (RGB Chroma & Work Area Outer Glow).        |
|     - Pro Avatar Profile Halos & Founder Patron Badge.                                  |
|     - Akses awal rilis beta dan prioritas voting roadmap fitur baru.                    |
|                                                                                         |
|  3. PRO SUBSCRIPTION (Opsional: Layanan Cloud-Managed Sync & AI Quota)                  |
|     - $5 / bulan atau $49 / tahun (Diskon Regional IDR: Rp 49.000/bln / Rp 490.000/thn). |
|     - Menyediakan kuota server AI cloud-managed (tanpa perlu setup model sendiri).      |
|     - Cloud Relay Relay E2EE Sync antar-perangkat (Linux, Windows, macOS, Android).     |
|                                                                                         |
+-----------------------------------------------------------------------------------------+
```

### Mekanisme Validasi Lisensi (Offline-First License Key)
- Pembayaran diproses melalui **GCC (Global Control Center)** yang terintegrasi dengan gateway pembayaran lokal (**Mayar.id**) untuk QRIS/Virtual Account dan kartu internasional (**Stripe**).
- Lisensi diterbitkan dalam bentuk token kriptografi yang ditandatangani menggunakan kunci **Ed25519**.
- Klien desktop CAMark memverifikasi keabsahan lisensi secara **100% offline** menggunakan public key yang tersemat pada binary kompilasi. Tidak diperlukan koneksi internet konstan untuk menggunakan fitur Pro.

---

## 5. Kebutuhan Fungsional (Functional Requirements)

### FR-1: Multi-Tab & Workspace Studio Layout (v1.0.17)
- Bilah tab multi-dokumen (`DocumentTabBar`) dengan dukungan tab aktif, indikator modifikasi (*dirty indicator*), dan penutupan tab via shortcut <kbd>Ctrl+W</kbd>.
- Context menu tab kustom: *Close*, *Close Others*, *Close to the Right*.
- Sinkronisasi instan saat berganti tab (*auto-flush in-memory vault docs*).
- Status bar bawah yang memuat indikator penyimpanan (`Saved` / `Unsaved`), jumlah kata/karakter/baris, estimasi waktu baca (*read time*), dan pemilih mode editor (*Editor*, *Split*, *Preview*).

### FR-2: Multi-Format Code & Text Files Support (v1.0.17)
- Mendukung penjelajahan direktori dan pengeditan langsung berkas di luar `.md`:
  - **Dokumentasi**: `.md`, `.markdown`, `.txt`, `.org`, `.rst`, `.adoc`
  - **Bahasa Pemrograman**: `.rs`, `.py`, `.go`, `.c`, `.cpp`, `.h`, `.hpp`, `.sh`, `.bash`, `.sql`, `.java`, `.php`, `.rb`, `.lua`, `.swift`, `.kt`, `.dart`, `.zig`, `.ex`
  - **Web Technologies**: `.html`, `.css`, `.scss`, `.js`, `.ts`, `.jsx`, `.tsx`, `.svelte`, `.vue`, `.svg`
  - **Konfigurasi & DevOps**: `.json`, `.yaml`, `.toml`, `.xml`, `.csv`, `.ini`, `.env`, `Dockerfile`, `Makefile`
- Ikon penanda berkas dinamis pada File Tree dan Tab Dokumen berdasarkan ekstensi bahasa.
- Tampilan pratinjau terformat dengan syntax highlighting dan tombol salin kode satu-klik untuk berkas kode non-markdown.

### FR-3: CodeMirror 6 Editor & Dynamic Syntax Highlighting
- Menggunakan arsitektur CodeMirror 6 dengan sistem compartment bahasa dinamis (`languageCompartment`).
- Tema gelap terintegrasi *One Dark* yang selaras dengan palet Obsidian.
- Indentasi otomatis, pencarian/penggantian teks (<kbd>Ctrl+F</kbd>, <kbd>Ctrl+H</kbd>), dan pelipatan baris teks (*line wrapping*).

### FR-4: Zero-Knowledge Encrypted Vault (SQLCipher 4 + Argon2id)
- Brankas catatan terenkripsi kuat menggunakan database SQLCipher 4 (`camark.db`).
- Derivasi kunci Master Password menggunakan **Argon2id v1.3**:
  - Memory cost: 64 MB (65.536 KiB)
  - Time cost (iterations): 3
  - Parallelism: 4 threads
  - Salt: 16 bytes acak CSPRNG dari `OsRng` (disimpan di `vault_salt.bin`)
- Kunci didekripsi dan disimpan pada memori volatile menggunakan `parking_lot::RwLock<Option<[u8; 32]>>`.
- Mekanisme **zeroize** otomatis saat brankas dikunci atau aplikasi ditutup.
- Verifikasi kata sandi aman via file canary terenkripsi (`vault_canary.bin`) dengan AES-256-GCM.
- Dukungan penggantian kata sandi aman (*rekeying*) via `PRAGMA rekey` tanpa risiko data terhapus.

### FR-5: 60 FPS Sync-Scroll & Dual-Pane Live Preview
- Sinkronisasi kursor editor CodeMirror 6 dengan posisi scroll pratinjau live secara presisi tinggi tanpa delay (*pixel-perfect sync-scroll*).
- Pipeline rendering 60 FPS menggunakan akselerasi perangkat keras WebKit/Blink DMABUF.

### FR-6: Matematika KaTeX & Diagram Mermaid.js
- Rendering rumus matematika in-line (`$...$`) dan blok tampilan (`$$...$$`) menggunakan KaTeX berkecepatan tinggi tanpa kompilasi server.
- Rendering diagram alur, grafik status, sequence diagram, dan arsitektur menggunakan Mermaid.js langsung di jendela pratinjau.

### FR-7: Table Auto-Formatter & Smart Paste
- Pemformatan otomatis kolom tabel Markdown (`|`) agar rapi saat mengetik.
- Smart clipboard paste: menyalin tabel dari Excel, Google Sheets, atau tabel HTML web langsung dikonversi menjadi sintaks tabel GFM rapi.

### FR-8: Marp Fullscreen Presentation Mode
- Memisahkan slide presentasi menggunakan pembatas standar `---`.
- Mode presentasi layar penuh dengan navigasi keyboard (<kbd>→</kbd>, <kbd>Space</kbd>, <kbd>PageDown</kbd>, <kbd>Home</kbd>, <kbd>End</kbd>).
- Dukungan KaTeX math dan Markdown formatting pada tiap slide.

### FR-9: Outline Drawer (Table of Contents)
- Ekstraksi hierarki judul dokumen (`H1` hingga `H6`) secara otomatis dan real-time.
- Navigasi cepat ke bagian dokumen menggunakan *smooth anchor scroll*.

### FR-10: Privacy-Scrubbed AI Co-Pilot & Local 9Router Proxy
- Kompatibel dengan API OpenAI standar.
- Default routing ke proxy lokal **9Router** (`http://127.0.0.1:20128/v1`) atau model lokal Ollama (`http://localhost:11434/v1`).
- Fitur aksi kontekstual: Periksa tata bahasa, sederhanakan kalimat, ubah tone, buat ringkasan, atau susun tabel.
- Privacy scrubber regex otomatis: menghapus alamat email, API keys, password, nomor telepon, dan nomor kartu kredit sebelum dikirim ke model AI.

### FR-11: Native Print & Academic PDF Export Engine
- Ekspor dokumen ke berkas PDF beresolusi tinggi dengan styling akademik bersih.
- Dukungan kustom CSS untuk cetakan dokumen resmi.

### FR-12: Desktop OS Integration
- Registrasi MIME type OS untuk membuka berkas Markdown dan file teks secara langsung (*Open With* dan default handler).
- Integrasi *Single-Instance Lock* via IPC event `listen('open-file')` untuk mencegah pembukaan jendela ganda yang boros memori.
- Integrasi manajer berkas native: sorot lokasi berkas langsung di Nautilus (Linux `--select`), Windows Explorer, atau macOS Finder.

### FR-13: UI Design System (@cecepazhar/caui + CADS)
- Palet *Dark Obsidian* (`#0A0A0C`, `#141416`, `#1C1C1F`) dengan aksen utama *Cyan* (`#06b6d4`).
- Komponen form modern dari paket `@cecepazhar/caui` (`CaToggleSwitch`, `CaInputBox`, `CaButton`, `CaModal`).
- Ambient Glow Engine untuk pengguna Pro (RGB Chroma bar & Work Area Outer Glow).

### FR-14: Multi-Profile & RBAC Security Layer
- Dukungan multi-profil pengguna pada satu perangkat dengan kunci PIN profil.
- Role-Based Access Control (RBAC) dengan matriks izin (`notes:*`, `backup:*`, `settings:*`).
- Pencatatan log audit otomatis untuk setiap akses data sensitif (`VAULT_INIT`, `VAULT_UNLOCK`, `VAULT_REKEY`).

### FR-15: Backup & Pemulihan Terjadwal (Mandatory)
- Ekspor dan impor backup data terenkripsi.
- Pengingat backup otomatis berkala untuk mencegah kehilangan data.

### FR-16: Internationalization (100% ID/EN Coverage)
- Dukungan penuh Bahasa Indonesia dan Bahasa Inggris untuk seluruh label antarmuka, pesan error, dialog konfirmasi, dan tooltips.

---

## 6. Kebutuhan Non-Fungsional (Non-Functional Requirements)

| Kategori | Parameter & Target Kualitas |
|---|---|
| **Performa Peluncuran** | Cold-start aplikasi < 1.0 detik pada perangkat target (ThinkPad X1 Yoga / standar laptop modern). |
| **Konsumsi Memori** | Konsumsi RAM idle < 35 MB; saat memuat berkas teks 10.000 baris < 48 MB. |
| **Keamanan Kriptografi** | Enkripsi SQLCipher 4 (AES-256-CBC, HMAC-SHA512). Derivasi kunci Argon2id (64 MB RAM, 3 iterasi, 4 thread). In-memory key zeroization menggunakan crate `zeroize`. File header guard terverifikasi (bebas string SQLite plaintext). |
| **Stabilitas Core Rust** | Kode core non-test bebas dari ekspresi panik (`unwrap()`, `expect()`, `panic!()`). Semua penanganan I/O file bersifat atomik. |
| **Aksesibilitas & UI** | Memenuhi kontras WCAG AA baik pada tema Dark maupun Light; navigasi keyboard penuh untuk semua aksi utama. |
| **Kemandirian Jaringan** | Nol telemetri pihak ketiga; tidak ada panggilan keluar tanpa persetujuan eksplisit pengguna. |

---

## 7. Rencana Distribusi & Go-To-Market (GTM)

1. **Distribusi Komunitas Pengembang & Package Manager**:
   - **Linux**: Rilis paket Arch Linux (AUR), Flathub Flatpak, Debian/Ubuntu `.deb`, Fedora `.rpm`, dan AppImage portable.
   - **macOS**: Distribusi universal binary via Homebrew Cask (`brew install --cask camark`).
   - **Windows**: Distribusi installer `.msi` dan manifest WinGet (`winget install camark`).
   - **Android**: Rilis APK mandiri dan melalui F-Droid repository.
2. **Peluncuran Komunitas Global**:
   - Kampanye Show HN (Hacker News) dengan artikel teknis: *"How we built a <35 MB RAM Markdown studio using Rust, Tauri v2, and Svelte 5"*.
   - Promosi di Reddit (`r/markdown`, `r/privacy`, `r/rust`, `r/obsidianmd`, `r/selfhosted`).
   - Rilis di Product Hunt dan GitHub Trending.
3. **Kampanye Founder Lifetime License ($29 / Rp 299.000)**:
   - Penawaran terbatas untuk 1.000 pengguna pertama dengan harga promosi Rp 299.000 ($29) sekali bayar seumur hidup.
   - Penetrasi pasar kreator konten dan developer di Indonesia melalui kanal media sosial (@cecepazhar).

---

## 8. Proyeksi Finansial (Tahun 1 – Tahun 3)

| Indikator Finansial | Tahun 1 (Validasi & Komunitas) | Tahun 2 (Pertumbuhan Global) | Tahun 3 (Skala Enterprise) |
|---|---|---|---|
| **Total Pengguna Aktif Bulanan (MAU)** | 35.000 | 150.000 | 500.000 |
| **Penjualan Lifetime License ($29 / Rp299k)** | 1.800 lisensi ($52.200) | 6.500 lisensi ($188.500) | 18.000 lisensi ($522.000) |
| **Pelanggan Langganan Cloud Pro ($49/thn)** | 400 pelanggan ($19.600) | 2.200 pelanggan ($107.800) | 8.500 pelanggan ($416.500) |
| **Total Pendapatan Kotor (Gross Revenue)** | **$71.800** (~Rp 1,12 Miliar) | **$296.300** (~Rp 4,62 Miliar) | **$938.500** (~Rp 14,64 Miliar) |
| **Biaya Operasional (Server, Gateway, CI)** | $5.500 | $22.000 | $68.000 |
| **Gross Margin Bersih** | **~92%** | **~92%** | **~93%** |

---

## 9. Roadmap Strategis Produk

```text
2026 Q4 (Rilis v1.0.17 - v1.1.0) [SELESAI & AKTIF]
  [x] Adopsi @cecepazhar/caui & Svelte 5 Runes.
  [x] Dukungan penuh file kode & teks non-markdown dengan dynamic syntax highlighting.
  [x] Eliminasi startup flicker & perbaikan master password lock cycle.
  [x] Integrasi proxy lokal 9Router AI & offline license validation.
  [x] Kampanye peluncuran $29 Lifetime License / Rp 299.000.

2027 Q1 (Ekspansi Mobile & Android)
  [ ] Peluncuran aplikasi native Android tablet & smartphone via Tauri v2.
  [ ] Optimasi layout touch gesture, virtual keyboard handling, dan mobile drawer.
  [ ] Ekspor APK mandiri dan rilis di F-Droid.

2027 Q2 (Integrasi Git & Versioning)
  [ ] Git sync engine tersemat langsung di dalam sidebar UI (Commit, Push, Pull).
  [ ] Visual side-by-side diff viewer untuk riwayat revisi catatan Markdown.

2027 Q3 - Q4 (E2EE Multi-Device Sync & Team Workspaces)
  [ ] Relay server sinkronisasi end-to-end encrypted (E2EE) antar perangkat desktop & ponsel.
  [ ] Brankas tim terenkripsi (*Shared Encrypted Team Vaults*) berbasis otorisasi kriptografi multi-pihak.
```

---

## 10. Persetujuan & Tanda Tangan

Dokumen ini menjadi acuan mutlak pengembangan teknis, tata kelola arsitektur, dan strategi komersialisasi produk CAMark.

*Disetujui dan disahkan oleh:*  
**Cecep Saeful Azhar Hidayat, ST**  
*Lead Architect & Founder Fathforce*  
15 Oktober 2026 · Bandung, Indonesia
