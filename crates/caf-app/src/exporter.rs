//! Native Markdown Multi-Format Exporter for CAMark (P5).

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub target_path: String,
    pub message: String,
}

pub async fn export_html(
    html_content: String,
    doc_title: String,
    target_path: String,
) -> Result<ExportResult, String> {
    tokio::task::spawn_blocking(move || {
        export_html_sync(&html_content, &doc_title, &target_path, false)
    })
    .await
    .map_err(|e| e.to_string())?
}

pub async fn export_pdf(
    html_content: String,
    doc_title: String,
    target_path: String,
) -> Result<ExportResult, String> {
    tokio::task::spawn_blocking(move || {
        export_html_sync(&html_content, &doc_title, &target_path, true)
    })
    .await
    .map_err(|e| e.to_string())?
}

fn export_html_sync(
    html_content: &str,
    doc_title: &str,
    target_path: &str,
    for_print_pdf: bool,
) -> Result<ExportResult, String> {
    let is_skripsi = html_content.contains("thesis-cover")
        || html_content.contains("thesis-document")
        || html_content.contains("skripsi_standar_4433")
        || html_content.contains("margins_mm");

    let print_styles = if for_print_pdf {
        if is_skripsi {
            r#"
  @page {
    size: A4;
    margin: 40mm 30mm 30mm 40mm;
  }
  @media print {
    body {
      background: #ffffff !important;
      color: #000000 !important;
      font-family: "Times New Roman", Times, serif !important;
      font-size: 12pt !important;
      line-height: 1.5 !important;
      max-width: 100% !important;
      margin: 0 !important;
      padding: 0 !important;
      text-align: justify !important;
      -webkit-print-color-adjust: exact !important;
      print-color-adjust: exact !important;
    }
    h1, h2, h3, h4, h5, h6 {
      page-break-after: avoid;
      break-after: avoid;
      color: #000000 !important;
      font-family: "Times New Roman", Times, serif !important;
      font-weight: bold !important;
    }
    h1 {
      font-size: 14pt !important;
      text-align: center !important;
    }
    h2 {
      font-size: 12pt !important;
    }
    p {
      text-indent: 1.25cm;
      margin-bottom: 6pt;
    }
    pre, blockquote, table, figure, .callout, .footnotes, .mermaid-container, .mermaid-block, .katex-display, .katex-block {
      page-break-inside: avoid;
      break-inside: avoid;
    }
    .footnote-back, .code-copy-btn, .no-print {
      display: none !important;
    }
  }
"#
        } else {
            r#"
  @page {
    size: A4;
    margin: 20mm 15mm 20mm 15mm;
  }
  @media print {
    body {
      background: #ffffff !important;
      color: #111827 !important;
      font-size: 11pt !important;
      line-height: 1.5 !important;
      max-width: 100% !important;
      margin: 0 !important;
      padding: 0 !important;
      -webkit-print-color-adjust: exact !important;
      print-color-adjust: exact !important;
    }
    h1, h2, h3, h4, h5, h6 {
      page-break-after: avoid;
      break-after: avoid;
      color: #000000 !important;
    }
    pre, blockquote, table, figure, .callout, .footnotes, .mermaid-container, .mermaid-block, .katex-display, .katex-block {
      page-break-inside: avoid;
      break-inside: avoid;
    }
    .footnote-back, .code-copy-btn, .no-print {
      display: none !important;
    }
  }
"#
        }
    } else {
        ""
    };

    let base_font_family = if is_skripsi {
        r#""Times New Roman", Times, serif"#
    } else {
        r#"-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif"#
    };

    let full_html = format!(
        r#"<!DOCTYPE html>
<html lang="id">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{}</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/katex@0.16.11/dist/katex.min.css">
  <style>
    body {{
      font-family: {};
      line-height: 1.6;
      color: #1a202c;
      max-width: 860px;
      margin: 40px auto;
      padding: 0 20px;
      background: #ffffff;
    }}
    table {{ width: 100%; border-collapse: collapse; margin: 20px 0; }}
    th, td {{ border: 1px solid #cbd5e0; padding: 8px 12px; text-align: left; }}
    th {{ background-color: #f7fafc; }}
    code {{ background: #edf2f7; padding: 2px 6px; border-radius: 4px; font-family: monospace; font-size: 0.9em; }}
    pre code {{ display: block; padding: 16px; overflow-x: auto; background: #1e293b; color: #f8fafc; border-radius: 6px; }}
    blockquote {{ border-left: 4px solid #06b6d4; margin: 0; padding-left: 16px; color: #4a5568; font-style: italic; }}
    img {{ max-width: 100%; height: auto; }}

    /* Callouts */
    .callout {{
      margin: 1.25em 0;
      padding: 0.75rem 1rem;
      border-left: 4px solid #06b6d4;
      border-radius: 0.375rem;
      background: rgba(6, 182, 212, 0.08);
    }}
    .callout-title {{ font-weight: 700; margin-bottom: 0.25rem; }}
    .callout-body > :first-child {{ margin-top: 0; }}
    .callout-body > :last-child {{ margin-bottom: 0; }}
    .callout-warning, .callout-caution {{ border-left-color: #d97706; background: rgba(217, 119, 6, 0.08); }}
    .callout-danger, .callout-failure, .callout-bug {{ border-left-color: #dc2626; background: rgba(220, 38, 38, 0.08); }}
    .callout-success, .callout-tip {{ border-left-color: #059669; background: rgba(5, 150, 105, 0.08); }}
    .callout-important {{ border-left-color: #9333ea; background: rgba(147, 51, 234, 0.08); }}

    /* Footnotes */
    .footnotes {{ font-size: 0.875em; border-top: 1px solid #e2e8f0; margin-top: 2rem; padding-top: 1rem; color: #475569; }}
    .footnote-ref {{ font-size: 0.75em; vertical-align: super; text-decoration: none; color: #0284c7; }}
    .footnote-back {{ text-decoration: none; margin-left: 0.25rem; }}

    /* Math & Mermaid */
    .katex-display, .katex-block {{ overflow-x: auto; margin: 1em 0; }}
    .mermaid-container, .mermaid-block {{ margin: 1.5em 0; text-align: center; }}
    .mermaid-container svg, .mermaid-block svg {{ max-width: 100%; height: auto; }}
    {}
  </style>
</head>
<body>
  <h1>{}</h1>
  <hr style="border: 0; border-top: 1px solid #e2e8f0; margin: 20px 0;">
  <main>
    {}
  </main>
</body>
</html>"#,
        doc_title, base_font_family, print_styles, doc_title, html_content
    );

    let path = Path::new(target_path);
    crate::fs_workspace::write_file_atomic(path, full_html.as_bytes())
        .map_err(|e| format!("Failed to export: {e}"))?;

    Ok(ExportResult {
        success: true,
        target_path: target_path.to_string(),
        message: if for_print_pdf {
            if is_skripsi {
                "Dokumen Skripsi (4-4-3-3) siap cetak/PDF berhasil diekspor".to_string()
            } else {
                "Dokumen siap cetak/PDF berhasil diekspor".to_string()
            }
        } else {
            "Dokumen HTML berhasil diekspor".to_string()
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_html_sync_structure() {
        let temp_dir = std::env::temp_dir();
        let target_file = temp_dir.join(format!("camark_test_export_{}.html", std::process::id()));
        let target_path = target_file.to_string_lossy().to_string();

        let sample_html = r#"<p>Sample text with <span class="katex">E=mc^2</span></p><div class="callout callout-warning"><div class="callout-title">Peringatan</div><div class="callout-body">Isi callout</div></div>"#;
        let res = export_html_sync(sample_html, "Judul Catatan", &target_path, false);
        assert!(res.is_ok(), "export_html_sync should succeed");

        let content = std::fs::read_to_string(&target_file).unwrap_or_default();
        let _ = std::fs::remove_file(&target_file);

        assert!(content.contains("<title>Judul Catatan</title>"));
        assert!(content.contains("<h1>Judul Catatan</h1>"));
        assert!(content.contains("callout-warning"));
        assert!(content.contains(".callout {"));
        assert!(content.contains(".footnotes {"));
        assert!(!content.contains("@media print"));
    }

    #[test]
    fn test_export_pdf_sync_structure_includes_print_media() {
        let temp_dir = std::env::temp_dir();
        let target_file = temp_dir.join(format!("camark_test_pdf_{}.html", std::process::id()));
        let target_path = target_file.to_string_lossy().to_string();

        let sample_html =
            r#"<div class="mermaid-container"><svg viewBox="0 0 100 100"></svg></div>"#;
        let res = export_html_sync(sample_html, "Dokumen Cetak", &target_path, true);
        assert!(res.is_ok(), "export_html_sync for print pdf should succeed");

        let content = std::fs::read_to_string(&target_file).unwrap_or_default();
        let _ = std::fs::remove_file(&target_file);

        assert!(content.contains("@media print"));
        assert!(content.contains("page-break-inside: avoid;"));
        assert!(content.contains("break-inside: avoid;"));
        assert!(content.contains("size: A4;"));
    }

    #[test]
    fn test_export_pdf_skripsi_4433_margins_and_times_font() {
        let temp_dir = std::env::temp_dir();
        let target_file = temp_dir.join(format!("camark_test_skripsi_{}.html", std::process::id()));
        let target_path = target_file.to_string_lossy().to_string();

        let sample_skripsi = r#"<div class="thesis-cover"><h1>JUDUL SKRIPSI</h1></div><div class="thesis-document"><p>Bab 1 Pendahuluan</p></div>"#;
        let res = export_html_sync(sample_skripsi, "Skripsi Uji", &target_path, true);
        assert!(
            res.is_ok(),
            "export_html_sync with thesis layout should succeed"
        );

        let content = std::fs::read_to_string(&target_file).unwrap_or_default();
        let _ = std::fs::remove_file(&target_file);

        assert!(content.contains("margin: 40mm 30mm 30mm 40mm;"));
        assert!(content.contains("Times New Roman"));
        assert!(content.contains("font-size: 12pt"));
        assert!(content.contains("line-height: 1.5"));
    }
}
