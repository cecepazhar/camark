import { describe, it, expect } from 'vitest';
import {
  generateSkripsiMarkdown,
  getSkripsiPrintCss,
  DEFAULT_SKRIPSI_PRESET,
  type ThesisMetadata,
} from './academicTemplate';

describe('Academic Thesis Template (F3-04 / X-02)', () => {
  const sampleMeta: ThesisMetadata = {
    title: 'Analisis Performa Sistem Berbasis Rust dan Tauri v2',
    author: 'Cecep Saeful Azhar Hidayat',
    nim: '1234567890',
    advisor1: 'Dr. Pembimbing Utama, M.Kom.',
    advisor2: 'Dr. Pembimbing Pendamping, M.T.',
    department: 'Teknik Informatika',
    faculty: 'Fakultas Sains dan Teknologi',
    university: 'Universitas Indonesia',
    city: 'Bandung',
    year: 2026,
  };

  it('generates standard frontmatter with 4-4-3-3 margins and Times New Roman font', () => {
    const md = generateSkripsiMarkdown(sampleMeta);
    expect(md).toContain('title: "Analisis Performa Sistem Berbasis Rust dan Tauri v2"');
    expect(md).toContain('author: "Cecep Saeful Azhar Hidayat"');
    expect(md).toContain('nim: "1234567890"');
    expect(md).toContain('paper: "A4"');
    expect(md).toContain('margins_mm: [40, 30, 30, 40]');
    expect(md).toContain('font_size_pt: 12');
    expect(md).toContain('line_spacing: 1.5');
  });

  it('includes cover, approval, abstract, and Chapter I sections', () => {
    const md = generateSkripsiMarkdown(sampleMeta);
    expect(md).toContain('### SKRIPSI');
    expect(md).toContain('LEMBAR PENGESAHAN');
    expect(md).toContain('ABSTRAK');
    expect(md).toContain('## BAB I');
    expect(md).toContain('## PENDAHULUAN');
    expect(md).toContain('1.1 Latar Belakang Masalah');
  });

  it('generates corresponding print CSS with correct 4-4-3-3 margin rules', () => {
    const css = getSkripsiPrintCss(DEFAULT_SKRIPSI_PRESET);
    expect(css).toContain('size: A4;');
    expect(css).toContain('margin: 40mm 30mm 30mm 40mm;');
    expect(css).toContain('font-size: 12pt !important;');
    expect(css).toContain('line-height: 1.5 !important;');
  });
});
