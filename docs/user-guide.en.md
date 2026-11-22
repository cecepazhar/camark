# CAMark User Guide (Official Documentation)

Welcome to **CAMark**, a high-performance desktop Markdown editor powered by Rust and Tauri v2, engineered for blistering speed, academic writing reliability, and 100% local privacy.

---

## 1. Feature Matrix & Tiers

CAMark provides two operational tiers: **Free** and **Pro**.

| Feature | Free | Pro |
|---|:---:|:---:|
| Fast GFM Markdown Editing | ✅ | ✅ |
| Math Rendering (KaTeX) & Diagrams (Mermaid) | ✅ | ✅ |
| Callouts, Footnotes, & Interactive Tables | ✅ | ✅ |
| Quick Switcher (`Ctrl+O`) | ✅ | ✅ |
| Direct Workspace Full-Text Search (`Ctrl+Shift+F`) | ✅ | ✅ |
| Backlinks, Unlinked Mentions, & Graph View | ✅ | ✅ |
| Conflict Detection & Atomic File Saves | ✅ | ✅ |
| Standalone HTML Export | ✅ | ✅ |
| **Native PDF Export (A4 & Academic Standards)** | ❌ | ✅ |
| **Indonesian Thesis / Skripsi Template (4-4-3-3 Margins)** | ❌ | ✅ |
| **Distraction-Free Zen Mode** | ❌ | ✅ |
| **Fullscreen Presentation Slides Mode** | ❌ | ✅ |
| **AI Copilot (BYOK - Summaries, TOC, Polishing)** | ❌ | ✅ |
| **Custom Pro Folder Formations** | ❌ | ✅ |

---

## 2. Getting Started

### Opening a Workspace
1. Click **Open Folder** in the sidebar or press `Ctrl+\` to toggle the file tree sidebar.
2. Select your local folder containing your Markdown notes.
3. All `.md` documents are indexed instantly with zero cloud dependency.

### Navigation Shortcuts
- **Quick Switcher (`Ctrl+O`):** Type parts of the filename or directory path to jump across documents instantly. Recently opened files are always sorted at the top.
- **Workspace Search (`Ctrl+Shift+F`):** Search for keywords or phrases across thousands of notes instantly using high-speed direct scanning.

---

## 3. Formatting & Markdown Syntax

CAMark supports complete GitHub Flavored Markdown (GFM) along with academic extensions:
- **Math Formulas:** Write inline math with `$E = mc^2$` or display math blocks:
  ```latex
  $$\int_{-\infty}^{\infty} e^{-x^2} dx = \sqrt{\pi}$$
  ```
- **Mermaid Diagrams:** Use code blocks tagged with `mermaid`:
  ````markdown
  ```mermaid
  graph TD;
    A[Design] --> B[Implementation];
    B --> C[Validation];
  ```
  ````
- **Callout Blocks:**
  ```markdown
  > [!NOTE]
  > Important note or highlight.
  ```
  Supported types: `[!NOTE]`, `[!TIP]`, `[!WARNING]`, `[!DANGER]`, `[!IMPORTANT]`.

---

## 4. Pro Features: PDF Export & Academic Templates

### Export to PDF via WebView Print (`Ctrl+P`)
Pro users can print documents or save directly to PDF without watermarks. The print pipeline automatically hides all editor chrome, formats headings, isolates code and diagrams from page breaks, and applies pristine typography.

### Academic Thesis Template (4-4-3-3 Margins)
Includes a preconfigured academic layout tailored for Indonesian academic standards:
- **Paper Size:** A4
- **Margins:** Top 4 cm, Left 4 cm, Bottom 3 cm, Right 3 cm
- **Typography:** Times New Roman 12pt, Line Spacing 1.5
- **Structure:** Cover, Approval Page, Abstract, Table of Contents, and Chapter I.

---

## 5. Licensing & Offline Grace Period

- Activate your license key under the **Account / License** menu.
- Once activated, CAMark operates completely offline for up to 7 consecutive days without requiring an active internet connection.

---
*Created by the CAMark Team — www.cecepazhar.com*
