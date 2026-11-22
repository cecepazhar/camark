# Panduan Pengguna CAMark (Buku Petunjuk Resmi)

Selamat datang di **CAMark**, aplikasi editor Markdown desktop modern berbasis Rust dan Tauri v2 yang dirancang khusus untuk kecepatan, keandalan penulisan akademik, serta privasi data 100% lokal.

---

## 1. Fitur Utama & Tier Pengguna

CAMark hadir dalam dua mode penggunaan: **Free (Bawaan)** dan **Pro (Profesional)**.

| Fitur | Free | Pro |
|---|:---:|:---:|
| Editor Markdown GFM Cepat | ✅ | ✅ |
| Preview Matematika (KaTeX) & Diagram (Mermaid) | ✅ | ✅ |
| Callouts, Catatan Kaki, & Tabel Interaktif | ✅ | ✅ |
| Quick Switcher Cepat (`Ctrl+O`) | ✅ | ✅ |
| Pencarian Teks Seluruh Workspace (`Ctrl+Shift+F`) | ✅ | ✅ |
| Panel Backlinks, Unlinked Mentions, & Graph View | ✅ | ✅ |
| Deteksi Konflik & Penyimpanan Atomik Aman | ✅ | ✅ |
| Export HTML Mandiri | ✅ | ✅ |
| **Export Cetak PDF (Ukuran A4 & Standar Akademik)** | ❌ | ✅ |
| **Template Skripsi / Tesis Indonesia (Margin 4-4-3-3)** | ❌ | ✅ |
| **Mode Zen Bebas Gangguan (Zen Mode)** | ❌ | ✅ |
| **Mode Presentasi Slide Layar Penuh** | ❌ | ✅ |
| **AI Copilot (BYOK - Ringkasan, TOC, Polishing)** | ❌ | ✅ |
| **Formasi Folder Kustom Pro** | ❌ | ✅ |

---

## 2. Memulai (Getting Started)

### Membuka Workspace / Folder
1. Klik menu **Buka Folder** di bilah sisi atau tekan `Ctrl+\` untuk membuka/menutup panel pohon berkas.
2. Pilih direktori di komputer Anda tempat catatan dan dokumen Markdown disimpan.
3. Seluruh berkas `.md` akan terindeks secara instan.

### Menavigasi Catatan
- **Quick Switcher (`Ctrl+O`):** Ketik sebagian nama atau path untuk beralih antar-catatan secara instan. Berkas yang baru saja dibuka akan otomatis berada di urutan teratas.
- **Pencarian Workspace (`Ctrl+Shift+F`):** Temukan kata atau frasa di ribuan berkas catatan secara langsung (direct scan).

---

## 3. Menulis & Format Markdown

CAMark mendukung standar GitHub Flavored Markdown (GFM) lengkap dengan ekstensi akademik:
- **Matematika:** Tuliskan formula inline `$E = mc^2$` atau blok display:
  ```latex
  $$\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}$$
  ```
- **Diagram Mermaid:** Gunakan blok kode bertipe `mermaid`:
  ````markdown
  ```mermaid
  graph TD;
    A[Rencana] --> B[Implementasi];
    B --> C[Pengujian];
  ```
  ````
- **Callouts:**
  ```markdown
  > [!NOTE]
  > Ini adalah catatan informasi penting.
  ```
  Tersedia tipe: `[!NOTE]`, `[!TIP]`, `[!WARNING]`, `[!DANGER]`, `[!IMPORTANT]`.

---

## 4. Fitur Pro: Cetak PDF & Template Skripsi

### Export PDF / Cetak WebView (`Ctrl+P`)
Pengguna Pro dapat mencetak dokumen atau menyimpannya langsung sebagai file PDF berkualitas tinggi tanpa watermark. Tampilan cetak secara otomatis menyembunyikan panel antarmuka dan memformat tata letak agar rapi dan tidak terpotong di tengah halaman.

### Template Skripsi Indonesia (Margin 4-4-3-3)
CAMark menyediakan template siap pakai untuk penulisan karya ilmiah dan skripsi:
- **Ukuran Kertas:** A4
- **Margin:** Atas 4 cm, Kiri 4 cm, Bawah 3 cm, Kanan 3 cm
- **Tipografi:** Times New Roman 12pt, Spasi 1.5
- **Struktur:** Sampul (Cover), Lembar Pengesahan, Abstrak, Daftar Isi, dan BAB I Pendahuluan.

---

## 5. Lisensi & Aktivasi Offline

- Kunci lisensi Pro dapat diaktifkan melalui menu **Akun / Lisensi**.
- Setelah diaktivasi, CAMark mendukung pemakaian offline hingga 7 hari berturut-turut tanpa memerlukan koneksi internet aktif.

---
*Dibuat oleh Tim CAMark — www.cecepazhar.com*
