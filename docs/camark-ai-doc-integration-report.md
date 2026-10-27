# Analisis & Usulan Arsitektur: Integrasi AI dengan Active Document di CAMark

## 1. Root Cause — Mengapa Hana AI Tidak Bisa Melihat Dokumen

### 1.1 Alur Saat Ini (as-is)

```
User → AiChatPanel.svelte
       → chatWithAi(prompt)            // frontend/src/lib/api/ai.ts
       → Tauri IPC: ai_chat(prompt)    // crates/caf-app/src/commands.rs
       → ai::chat(session, prompt, privacy_level, consent_given)
       → build_context(...)             // caf-core/src/ai.rs
       → compose_prompt(guardrails, tone, lang, custom, context, user_prompt)
       → HTTP POST {base_url}/chat/completions
```

### 1.2 Masalah 1: Context Tidak Terhubung ke Editor

`build_context()` di `caf-core/src/ai.rs` **hanya** menghasilkan context dari SQLite DB (jumlah notes, profiles, atau isi notes DB) dengan `PrivacyLevel::Summary / Detailed / Full`.

Dokumen yang sedang dibuka di editor CodeMirror — yang hidup di **state frontend** (`rawMarkdown`, `currentFilePath`, `activeTab`) — **tidak pernah dikirim** ke Rust. Frontend `aiChat.svelte.ts` hanya menyimpan open/closed state. `AiChatPanel.svelte` mengirim `prompt` saja tanpa `activeDocPath`, `activeDocContent`, atau `selectionRange`.

**Tidak ada IPC channel** untuk mengirim isi dokumen aktif dari frontend ke backend. Satu-satunya `Tauri event` yang ada adalah `open-file` (Rust → frontend, satu arah).

### 1.3 Masalah 2: Tidak Ada State Sharing untuk Editor

`docTabs.svelte.ts` menyimpan `id`, `path`, `title`, `isVault`, `isDirty` — **tidak menyimpan content** dokumen. Content hanya hidup di `rawMarkdown` state di dalam `+page.svelte` (CodeMirror 6). Tidak ada mekanisme publik (publik export dari store) yang memungkinkan `AiChatPanel` (yang di-mount di `+layout.svelte`) untuk mengakses `rawMarkdown` saat ini.

`AiChatPanel` di-mount di level `+layout.svelte` (layout root), sementara `+page.svelte` (editor utama) di-render sebagai `<slot>` di dalam layout. Kedua komponen **saling terpisah scope**, tidak ada shared store untuk content editor.

### 1.4 Masalah 3: Tidak Ada Tool Calling / Auto-typing

AI menggunakan `stream: false` — satu request, satu response, diblok. Tidak ada mekanisme tool-calling atau streaming ke editor. Untuk auto-typing, yang dibutuhkan:
1. Stream token-by-token dari API (SSE)
2. Bridge dari Tauri event → CodeMirror `view.dispatch()`
3. Atau cara sederhana: backend mengirim hasil penuh, frontend dispatch ke CodeMirror dengan progress indicator

---

## 2. Usulan Arsitektur

### 2.1 Layer 1 — Frontend: Document Context Store

Buat store baru untuk menjembatani state editor ke AI:

**File baru: `frontend/src/lib/stores/documentContext.svelte.ts`**

```ts
// frontend/src/lib/stores/documentContext.svelte.ts
import { getActiveTab, getDocTabs, getActiveTabId, type DocTab } from '$lib/stores/docTabs.svelte';

interface DocContextSnapshot {
  activeDocId: string | null;
  activeDocPath: string | null;
  activeDocTitle: string | null;
  activeDocContent: string;   // full text (truncated to token budget)
  selectedText: string;       // current selection, if any
  isVaultDoc: boolean;
  viewMode: string;
  wordCount: number;
  lineCount: number;
}

const state = $state<DocContextSnapshot>({
  activeDocId: null,
  activeDocPath: null,
  activeDocTitle: null,
  activeDocContent: '',
  selectedText: '',
  isVaultDoc: false,
  viewMode: 'split',
  wordCount: 0,
  lineCount: 0,
});

// Dipanggil dari +page.svelte setiap kali editorView state berubah
export function updateDocumentContext(partial: Partial<DocContextSnapshot>) {
  Object.assign(state, partial);
}

export function getDocumentContextSnapshot(): DocContextSnapshot {
  return { ...state };
}

export function getDocumentContextState() {
  return state;
}
```

**Modifikasi di `+page.svelte`:**
- Di dalam `updateListener` CodeMirror, panggil `updateDocumentContext({ activeDocContent: rawMarkdown, ... })`
- Saat selection berubah, update `selectedText`
- Saat `activeTab` berubah, update `activeDocId`, `activeDocPath`, `activeDocTitle`

### 2.2 Layer 2 — Frontend: AiChatPanel Mengirim Context

**Modifikasi di `AiChatPanel.svelte`:**

```ts
import { getDocumentContextSnapshot } from '$lib/stores/documentContext.svelte';

async function handleSend() {
  // ...
  const docCtx = getDocumentContextSnapshot();
  const response = await chatWithAi(
    prompt,
    {
      activeDocPath: docCtx.activeDocPath,
      activeDocTitle: docCtx.activeDocTitle,
      activeDocContent: docCtx.activeDocContent,
      selectedText: docCtx.selectedText,
      isVaultDoc: docCtx.isVaultDoc,
      privacyLevel: 'detailed'  // user memilih privacy level
    }
  );
}
```

**Modifikasi di `ai.ts`:**

```ts
export interface AiDocumentContext {
  activeDocPath: string | null;
  activeDocTitle: string | null;
  activeDocContent: string;
  selectedText: string;
  isVaultDoc: boolean;
  privacyLevel?: PrivacyLevel | null;
}

export async function chatWithAi(
  prompt: string,
  docContext?: AiDocumentContext
): Promise<AiChatResponse> {
  return await cmdAiChat({
    prompt,
    docContext: docContext ?? undefined,
  });
}
```

### 2.3 Layer 3 — Rust: Terima Context dari Frontend

**Modifikasi di `caf-app/src/commands.rs`:**

```rust
#[derive(serde::Deserialize, ts_rs::TS)]
#[ts(export)]
pub struct DocContext {
    pub active_doc_path: Option<String>,
    pub active_doc_title: Option<String>,
    pub active_doc_content: String,
    pub selected_text: String,
    pub is_vault_doc: bool,
}

#[tauri::command]
pub async fn ai_chat(
    prompt: String,
    doc_context: Option<DocContext>,
    privacy_level: Option<ai::PrivacyLevel>,
    consent_given: Option<bool>,
) -> Result<ai::AiChatResponse, CafError> {
    run_blocking(move || {
        let session = caf_core::session::get_current_session()
            .unwrap_or_else(|_| caf_core::session::Session::new("default", "member"));
        ai::chat_with_doc_context(
            &session,
            &prompt,
            doc_context.as_ref(),
            privacy_level,
            consent_given.unwrap_or(false),
        )
    })
    .await
}
```

**Modifikasi di `caf-core/src/ai.rs`:**

```rust
pub struct DocContext {
    pub active_doc_path: Option<String>,
    pub active_doc_title: Option<String>,
    pub active_doc_content: String,
    pub selected_text: String,
    pub is_vault_doc: bool,
}

pub fn chat_with_doc_context(
    session: &Session,
    prompt: &str,
    doc_ctx: Option<&DocContext>,
    privacy_level: Option<PrivacyLevel>,
    consent_given: bool,
) -> Result<AiChatResponse, CatermError> {
    let cfg = get_stored_config()?;
    if cfg.mode == AiMode::Off { return Err(CatermError::Ai(AiError::Disabled)); }

    let level = privacy_level.unwrap_or(cfg.privacy_level_default.clone());
    
    // Build context: combine DB context (existing) + live document context
    let mut context_parts: Vec<String> = Vec::new();
    
    // Bagian 1: DB context (sudah ada)
    let db_ctx = build_context(session, &level, consent_given, cfg.allow_full_detail_for_local, &cfg.base_url)?;
    if !db_ctx.content.is_empty() {
        context_parts.push(format!("DB Context:\n{}", db_ctx.content));
    }
    
    // Bagian 2: Live document context (baru)
    if let Some(ctx) = doc_ctx {
        if !ctx.active_doc_content.is_empty() {
            let (scrubbed, _) = scrub_text(&ctx.active_doc_content);
            let path_hint = ctx.active_doc_path.as_deref().unwrap_or("(untitled)");
            context_parts.push(format!(
                "Live Document ({}):\n{}",
                path_hint, scrubbed
            ));
        }
        if !ctx.selected_text.is_empty() {
            let (scrubbed_sel, _) = scrub_text(&ctx.selected_text);
            context_parts.push(format!("User Selection:\n{}", scrubbed_sel));
        }
    }
    
    let combined_context = context_parts.join("\n\n");
    
    // Scrub prompt
    let (scrubbed_prompt, prompt_redactions) = scrub_text(prompt);
    
    let (system_msg, user_msg) = compose_prompt(
        GUARDRAILS_TEXT,
        &cfg.tone,
        &cfg.response_language,
        &cfg.custom_instructions,
        &combined_context,
        &scrubbed_prompt,
    );
    
    // ... (HTTP call, same as before)
}
```

### 2.4 Layer 4 — Auto-typing / Streaming ke Editor

Terdapat dua pilihan pendekatan:

#### Opsi A: Full-Response + Dispatch (sederhana, tanpa SSE)

Backend tetap `stream: false`. Frontend menerima `AiChatResponse` lengkap, lalu:

```ts
// +page.svelte
function handleApplyAiResult(resultText: string, mode: 'replace' | 'append' | 'insert_at_cursor' = 'replace') {
  if (!editorView) return;
  const doc = editorView.state.doc;
  
  if (mode === 'append') {
    editorView.dispatch({
      changes: { from: doc.length, insert: resultText },
    });
  } else if (mode === 'insert_at_cursor') {
    const sel = editorView.state.selection.main;
    editorView.dispatch({
      changes: { from: sel.from, to: sel.to, insert: resultText },
      selection: { anchor: sel.from + resultText.length },
    });
  } else {
    // replace full document
    editorView.dispatch({
      changes: { from: 0, to: doc.length, insert: resultText },
    });
  }
  editorView.focus();
}
```

Tambahkan tombol di `AiChatPanel`: "✍ Apply to document" → panggil `handleApplyAiResult` via Tauri event `ai-apply-result`.

#### Opsi B: SSE Streaming + Live Typing (advanced)

Backend ubah `stream: true`. Rust emit Tauri event token-by-token:

```rust
// caf-app/src/commands.rs
#[tauri::command]
pub async fn ai_chat_stream(
    app: tauri::AppHandle,
    prompt: String,
    doc_context: Option<DocContext>,
    privacy_level: Option<ai::PrivacyLevel>,
) -> Result<(), CafError> {
    // Stream SSE, emit event "ai-stream-token" per chunk
    // Emit "ai-stream-done" saat selesai
}
```

```ts
// +page.svelte
let aiStreamingText = $state('');
let isAiStreaming = $state(false);

listen<string>('ai-stream-token', (e) => {
  aiStreamingText += e.payload;
  // Per-frame dispatch (throttle ~50ms untuk performa)
  if (editorView && !throttleRef.value) {
    const cursor = editorView.state.doc.length;
    editorView.dispatch({
      changes: { from: cursor, insert: e.payload },
      selection: { anchor: cursor + e.payload.length },
    });
    throttleRef.value = true;
    setTimeout(() => throttleRef.value = false, 50);
  }
});

listen('ai-stream-done', () => {
  isAiStreaming = false;
});
```

**Rekomendasi: Opsi A dulu** (implementasi lebih cepat, UX sudah cukup baik). Opsi B menyusul sebagai enhancement setelah Opsi A stabil.

### 2.5 Layer 5 — Tauri Event Bridge

```rust
// caf-app/src/lib.rs — register event emitter
#[tauri::command]
pub fn emit_ai_apply(text: String, mode: String) -> Result<(), String> {
    // Ini untuk frontend → Rust (sudah cukup pakai invoke langsung)
    // Event Rust → frontend hanya untuk streaming
}
```

Untuk Opsi A cukup pakai `invoke('ai_apply_result', { text, mode })` — tidak perlu event. Untuk Opsi B perlu `tauri::Emitter::emit`.

---

## 3. Perubahan per File (Ringkasan)

| File | Aksi | Detail |
|---|---|---|
| `frontend/src/lib/stores/documentContext.svelte.ts` | **BARU** | Snapshot store untuk editor context |
| `frontend/src/lib/stores/aiChat.svelte.ts` | Modifikasi | Tambah `attachDocContext()` jika perlu |
| `frontend/src/lib/api/ai.ts` | Modifikasi | Tambah `AiDocumentContext` parameter ke `chatWithAi()` |
| `frontend/src/lib/generated/commands.ts` | Regenerasi | Run `tauri codegen` / regen bindings |
| `frontend/src/routes/+page.svelte` | Modifikasi | Emit context ke `documentContext`, terima event apply |
| `frontend/src/lib/components/AiChatPanel.svelte` | Modifikasi | Kirim context, tambah tombol "Apply" |
| `crates/caf-core/src/ai.rs` | Modifikasi | Tambah `DocContext` struct + `chat_with_doc_context()` |
| `crates/caf-app/src/commands.rs` | Modifikasi | Terima `doc_context` di `ai_chat` command |
| `crates/caf-app/src/lib.rs` | Regenerasi | Register command baru / update binding |
| `crates/caf-app/tauri.conf.json` | Verifikasi | Pastikan permissions ada |

---

## 4. Estimasi Upaya

| Fase | Task | Estimasi |
|---|---|---|
| Fase 1 | Document context store + wiring dari `+page.svelte` | 2–4 jam |
| Fase 2 | Rust: terima `DocContext`, update `chat_with_doc_context()` | 3–5 jam |
| Fase 3 | UI: tombol "Apply to Document" + dispatch ke CodeMirror | 2–3 jam |
| Fase 4 | Codegen + test end-to-end | 1–2 jam |
| **Total (Opsi A)** | | **~8–14 jam** |
| Fase 5 (Opsi B) | SSE streaming + live typing + throttle | 6–10 jam tambahan |

---

## 5. Hal yang Perlu Diperhatikan

1. **Privacy scrubbing** — Content dokumen dikirim via IPC (in-memory), tidak pernah keluar dari proses. PiiScrubber tetap aktif. Jangan pernah log content mentah ke file.
2. **Token budget** — Dokumen besar (mis. 100K kata) akan membludak context. Batasi `activeDocContent` yang dikirim ke max ~4096 karakter untuk awal; tambahkan toggle "Full document" untuk user yang sengaja.
3. **Vault docs** — Jika `isVaultDoc == true`, dokumen berasal dari SQLCipher, bukan filesystem. Scrub sama, tapi jangan pernah simpan path file.
4. **Streaming throttle** — Untuk Opsi B, throttle ~50ms per batch token agar CodeMirror tidak re-render per karakter (performa di dokumen besar bisa drop).
5. **Permissions Tauri** — `ai_chat` command perlu `allow-ai-chat` di `tauri.conf.json`. Sudah ada, tidak perlu perubahan.
