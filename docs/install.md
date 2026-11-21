# Instal CAMark / Install CAMark

Berlaku untuk `1.1.0-beta.x`. Installer **belum ditandatangani oleh Windows atau Apple**, jadi sistem akan memperingatkan Anda. Itu normal; langkah di bawah menunjukkan cara melanjutkan dan cara memastikan berkas tidak diubah.
*Applies to `1.1.0-beta.x`. Installers are not code-signed by Windows or Apple yet, so the OS will warn you. That is expected; below is how to continue and how to verify the download.*

---

## Bahasa Indonesia

### 1. Unduh
Ambil installer dan berkas `SHA256SUMS-*.txt` dari halaman rilis GitHub: https://github.com/cecepazhar/camark/releases

| Sistem | Berkas |
|---|---|
| Windows | `.exe` (NSIS) |
| Debian / Ubuntu | `.deb` |
| Fedora / openSUSE | `.rpm` |
| Linux lain | `.AppImage` (bila ada di rilis itu) |
| macOS | `.dmg` (bila ada di rilis itu; belum diuji bila tertulis demikian di catatan rilis) |

### 2. Cek keaslian (disarankan)
Bandingkan SHA-256 berkas dengan isi `SHA256SUMS-*.txt` dari rilis yang sama.

```bash
# Linux
sha256sum -c SHA256SUMS* --ignore-missing
# macOS
shasum -a 256 CAMark*.dmg
```
```powershell
# Windows (PowerShell)
Get-FileHash -Algorithm SHA256 .\CAMark*.exe
```
Angka harus sama persis. Bila berbeda, jangan dipasang dan laporkan.

### 3. Pasang
- **Windows:** buka `.exe`. Bila muncul "Windows protected your PC" (SmartScreen), klik **More info** lalu **Run anyway**.
- **Debian / Ubuntu:** `sudo apt install ./CAMark_*.deb`
- **Fedora:** `sudo dnf install ./CAMark-*.rpm`
- **AppImage:** `chmod +x CAMark*.AppImage && ./CAMark*.AppImage`
- **macOS:** buka `.dmg`, seret CAMark ke Applications. Saat pertama kali, **klik kanan aplikasi → Open → Open**. Bila macOS tetap memblokir: `xattr -d com.apple.quarantine /Applications/CAMark.app`

### 4. Dari versi 1.0.x (pasang manual sekali)
Kunci tanda tangan update diganti mulai `1.1.0-beta.1`. Aplikasi 1.0.x **tidak akan menawarkan update otomatis** ke versi ini. Pasang `1.1.0-beta.1` secara manual sesuai langkah di atas. Setelah itu, update berikutnya masuk otomatis.

Sebelum memasang, **buat backup**: pada vault terenkripsi gunakan Export Backup, dan salin folder workspace Anda.

### 5. Masalah umum
- Windows tidak menampilkan "Run anyway": klik **More info** dulu, tombolnya muncul setelah itu.
- Linux: `libwebkit2gtk-4.1` harus terpasang (paket `.deb`/`.rpm` menariknya otomatis).
- Laporkan masalah di https://github.com/cecepazhar/camark/issues (sertakan sistem operasi dan versi).

---

## English

### 1. Download
Get the installer and the `SHA256SUMS-*.txt` file from the GitHub release page: https://github.com/cecepazhar/camark/releases

| System | File |
|---|---|
| Windows | `.exe` (NSIS) |
| Debian / Ubuntu | `.deb` |
| Fedora / openSUSE | `.rpm` |
| Other Linux | `.AppImage` (if present in that release) |
| macOS | `.dmg` (if present; untested if the release notes say so) |

### 2. Verify (recommended)
Compare the file's SHA-256 with `SHA256SUMS-*.txt` from the same release.

```bash
# Linux
sha256sum -c SHA256SUMS* --ignore-missing
# macOS
shasum -a 256 CAMark*.dmg
```
```powershell
# Windows (PowerShell)
Get-FileHash -Algorithm SHA256 .\CAMark*.exe
```
The values must match exactly. If they differ, do not install; report it.

### 3. Install
- **Windows:** run the `.exe`. If "Windows protected your PC" (SmartScreen) appears, click **More info**, then **Run anyway**.
- **Debian / Ubuntu:** `sudo apt install ./CAMark_*.deb`
- **Fedora:** `sudo dnf install ./CAMark-*.rpm`
- **AppImage:** `chmod +x CAMark*.AppImage && ./CAMark*.AppImage`
- **macOS:** open the `.dmg`, drag CAMark to Applications. On first launch **right-click the app → Open → Open**. If macOS still blocks it: `xattr -d com.apple.quarantine /Applications/CAMark.app`

### 4. Coming from 1.0.x (manual install once)
The update-signing key changed in `1.1.0-beta.1`. Installed 1.0.x apps **will not be offered this version automatically**. Install `1.1.0-beta.1` manually as above; later updates then arrive automatically.

**Back up first:** use Export Backup for the encrypted vault and copy your workspace folder.

### 5. Troubleshooting
- No "Run anyway" button on Windows: click **More info** first; the button appears after that.
- Linux: `libwebkit2gtk-4.1` must be installed (the `.deb`/`.rpm` pull it in).
- Report problems at https://github.com/cecepazhar/camark/issues (include your OS and version).
