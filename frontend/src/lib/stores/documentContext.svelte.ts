export interface DocContextSnapshot {
  active_doc_path: string | null;
  active_doc_title: string | null;
  active_doc_content: string;
  selected_text: string;
  is_vault_doc: boolean;
}

export type EditorApplyMode = 'replace' | 'append' | 'insert';
export type EditorApplyHandler = (text: string, mode: EditorApplyMode) => void;

let docContext = $state<DocContextSnapshot>({
  active_doc_path: null,
  active_doc_title: null,
  active_doc_content: '',
  selected_text: '',
  is_vault_doc: false,
});

let editorApplyHandler: EditorApplyHandler | null = null;
let editorUndoHandler: (() => boolean) | null = null;

export function registerEditorApplyHandler(handler: EditorApplyHandler): () => void {
  editorApplyHandler = handler;
  return () => {
    if (editorApplyHandler === handler) {
      editorApplyHandler = null;
    }
  };
}

export function registerEditorUndoHandler(handler: () => boolean): () => void {
  editorUndoHandler = handler;
  return () => {
    if (editorUndoHandler === handler) {
      editorUndoHandler = null;
    }
  };
}

export function undoEditor(): boolean {
  if (editorUndoHandler) {
    return editorUndoHandler();
  }
  return false;
}

export function applyAiTextToEditor(text: string, mode: EditorApplyMode = 'replace'): boolean {
  if (editorApplyHandler) {
    editorApplyHandler(text, mode);
    return true;
  }
  return false;
}

export function updateDocumentContext(partial: Partial<DocContextSnapshot>): void {
  if (partial.active_doc_path !== undefined) docContext.active_doc_path = partial.active_doc_path;
  if (partial.active_doc_title !== undefined) docContext.active_doc_title = partial.active_doc_title;
  if (partial.active_doc_content !== undefined) docContext.active_doc_content = partial.active_doc_content;
  if (partial.selected_text !== undefined) docContext.selected_text = partial.selected_text;
  if (partial.is_vault_doc !== undefined) docContext.is_vault_doc = partial.is_vault_doc;
}

export function getDocumentContextSnapshot(): DocContextSnapshot {
  return {
    active_doc_path: docContext.active_doc_path,
    active_doc_title: docContext.active_doc_title,
    active_doc_content: docContext.active_doc_content,
    selected_text: docContext.selected_text,
    is_vault_doc: docContext.is_vault_doc,
  };
}

export function getDocumentContextState() {
  return docContext;
}
