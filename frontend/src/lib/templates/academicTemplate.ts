/**
 * Academic Thesis / Skripsi Template Generator and Preset Manager (F3-04 / X-02)
 *
 * Implements standard Indonesian Academic Thesis format:
 * - A4 Paper, Margins: Top 40mm, Left 40mm, Bottom 30mm, Right 30mm (4-4-3-3)
 * - Typography: Times New Roman 12pt, Line Spacing 1.5
 * - Standard Frontmatter metadata: title, author, nim, advisor, faculty, university, year
 * - Structured sections: Cover, Abstract, Table of Contents, Chapter I (Pendahuluan)
 * - Configurable preset values with export/import capabilities
 */

export interface ThesisMetadata {
  title: string;
  author: string;
  nim: string;
  advisor1: string;
  advisor2?: string;
  department: string;
  faculty: string;
  university: string;
  city: string;
  year: number;
}

export interface ThesisPreset {
  id: string;
  name: string;
  paperSize: 'A4';
  marginTopMm: number;
  marginLeftMm: number;
  marginBottomMm: number;
  marginRightMm: number;
  fontFamily: string;
  fontSizePt: number;
  lineSpacing: number;
}

export const DEFAULT_SKRIPSI_PRESET: ThesisPreset = {
  id: 'skripsi_standar_4433',
  name: 'Skripsi Standar (4-4-3-3)',
  paperSize: 'A4',
  marginTopMm: 40,
  marginLeftMm: 40,
  marginBottomMm: 30,
  marginRightMm: 30,
  fontFamily: 'Times New Roman, Times, serif',
  fontSizePt: 12,
  lineSpacing: 1.5,
};

/**
 * Generate Markdown content for Indonesian Academic Thesis (Skripsi) template.
 */
export function generateSkripsiMarkdown(meta: ThesisMetadata, preset: ThesisPreset = DEFAULT_SKRIPSI_PRESET): string {
  const frontmatter = `---
title: "${meta.title}"
author: "${meta.author}"
nim: "${meta.nim}"
advisor1: "${meta.advisor1}"
${meta.advisor2 ? `advisor2: "${meta.advisor2}"\n` : ''}department: "${meta.department}"
faculty: "${meta.faculty}"
university: "${meta.university}"
year: ${meta.year}
preset: "${preset.id}"
page_format:
  paper: "${preset.paperSize}"
  margins_mm: [${preset.marginTopMm}, ${preset.marginRightMm}, ${preset.marginBottomMm}, ${preset.marginLeftMm}]
  font: "${preset.fontFamily}"
  font_size_pt: ${preset.fontSizePt}
  line_spacing: ${preset.lineSpacing}
---

<div class="thesis-cover text-center" style="text-align: center; page-break-after: always; padding: 40px 0;">

# ${meta.title.toUpperCase()}

<br><br>

### SKRIPSI

Diajukan untuk Memenuhi Sebagian dari Syarat Memperoleh Gelar Sarjana

<br><br><br>

**Oleh:**  
**${meta.author.toUpperCase()}**  
NIM. ${meta.nim}

<br><br><br><br>

**${meta.department.toUpperCase()}**  
**${meta.faculty.toUpperCase()}**  
**${meta.university.toUpperCase()}**  
**${meta.city.toUpperCase()}**  
**${meta.year}**

</div>

<!-- pagebreak -->

<div class="thesis-approval" style="page-break-after: always;">

## LEMBAR PENGESAHAN

**Judul:** ${meta.title}  
**Nama:** ${meta.author}  
**NIM:** ${meta.nim}  
**Program Studi:** ${meta.department}  

Menyetujui,

<br><br>

| Pembimbing Utama | Pembimbing Pendamping |
| :---: | :---: |
| <br><br><br> | <br><br><br> |
| **${meta.advisor1}** | **${meta.advisor2 || '-'}** |

</div>

<!-- pagebreak -->

<div class="thesis-abstract" style="page-break-after: always;">

## ABSTRAK

Tuliskan ringkasan penelitian Anda di sini (maksimal 250-300 kata). Abstrak mencakup latar belakang masalah, tujuan, metodologi penelitian yang digunakan, hasil temuan utama, dan kesimpulan.

**Kata Kunci:** Kata Kunci 1, Kata Kunci 2, Kata Kunci 3

</div>

<!-- pagebreak -->

## BAB I  
## PENDAHULUAN

### 1.1 Latar Belakang Masalah
Uraikan latar belakang masalah penelitian secara komprehensif di sini dengan menyertakan data, fakta empiris, dan referensi akademik terkait.

### 1.2 Rumusan Masalah
1. Bagaimana analisis masalah pertama?
2. Bagaimana perancangan dan implementasi solusi?
3. Bagaimana evaluasi performa dan efektivitas hasil?

### 1.3 Tujuan Penelitian
1. Menganalisis permasalahan yang terjadi.
2. Membangun sistem yang mampu menjawab kebutuhan.
3. Mengukur efektivitas dan kinerja solusi yang dikembangkan.

### 1.4 Manfaat Penelitian
- **Bagi Akademik:** Memberikan kontribusi pada pengembangan keilmuan.
- **Bagi Praktisi:** Memberikan acuan solusi teknis yang dapat diterapkan secara nyata.

### 1.5 Sistematika Penulisan
- **BAB I PENDAHULUAN**: Berisi latar belakang, rumusan masalah, tujuan, manfaat, dan sistematika.
- **BAB II TINJAUAN PUSTAKA**: Berisi landasan teori, kajian pustaka, dan penelitian terdahulu.
- **BAB III METODOLOGI PENELITIAN**: Berisi tahapan penelitian, arsitektur sistem, dan instrumen pengujian.
- **BAB IV HASIL DAN PEMBAHASAN**: Berisi implementasi dan evaluasi hasil temuan.
- **BAB V KESIMPULAN DAN SARAN**: Berisi kesimpulan akhir dan saran pengembangan ke depan.
`;

  return frontmatter;
}

/**
 * Generate CSS snippet for thesis print preview conforming to 4-4-3-3 margin and Times 12pt 1.5 line spacing.
 */
export function getSkripsiPrintCss(preset: ThesisPreset = DEFAULT_SKRIPSI_PRESET): string {
  return `
@page thesis-page {
  size: ${preset.paperSize};
  margin: ${preset.marginTopMm}mm ${preset.marginRightMm}mm ${preset.marginBottomMm}mm ${preset.marginLeftMm}mm;
}

.thesis-document {
  font-family: ${preset.fontFamily} !important;
  font-size: ${preset.fontSizePt}pt !important;
  line-height: ${preset.lineSpacing} !important;
  color: #000000 !important;
  text-align: justify !important;
}

.thesis-document h1,
.thesis-document h2,
.thesis-document h3 {
  font-family: ${preset.fontFamily} !important;
  color: #000000 !important;
  font-weight: bold !important;
  text-align: center;
  page-break-after: avoid;
}

.thesis-document h1 {
  font-size: 14pt !important;
  margin-top: 24pt;
  margin-bottom: 18pt;
}

.thesis-document h2 {
  font-size: 12pt !important;
  margin-top: 18pt;
  margin-bottom: 12pt;
  text-align: left;
}

.thesis-document p {
  text-indent: 1.25cm;
  margin-bottom: 6pt;
}
`;
}
