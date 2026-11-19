// I-01 — pure helpers for pasting / dropping images into the markdown editor.
// Kept free of DOM and component state so the behaviour is unit-testable.

/** Client-side mirror of the backend `MAX_IMAGE_BYTES` (25 MB). */
export const MAX_IMAGE_BYTES = 25 * 1024 * 1024;

/** MIME types the frontend will even hand to the backend. Mirrors the Rust
 *  allow-list; anything else is rejected client-side so we can give a fast,
 *  specific error without a round-trip. */
export const ALLOWED_IMAGE_MIMES = [
  'image/png',
  'image/jpeg',
  'image/jpg',
  'image/webp',
  'image/gif',
  'image/bmp',
  'image/avif',
  'image/svg+xml',
  'image/tiff',
];

/** True when every item is an image the backend will accept. */
export function isAllowedImage(mime: string, size: number): boolean {
  if (!mime || !ALLOWED_IMAGE_MIMES.includes(mime)) return false;
  if (size <= 0) return false;
  if (size > MAX_IMAGE_BYTES) return false;
  return true;
}

/** Strip a common image extension so the alt text is the human part of the name. */
function altFromFileName(name: string): string {
  if (!name) return '';
  return name.replace(/\.(png|jpe?g|webp|gif|bmp|svg|avif|tiff?)$/i, '');
}

/**
 * Build the markdown image snippet to insert at the caret.
 *
 * `before`/`after` are the single character immediately before/after the caret
 * (empty string when the caret is at the document edge) so the snippet gets
 * exactly the right number of blank lines and the image block renders cleanly
 * without collapsing into a neighbouring paragraph.
 */
export function buildImageMarkdown(
  link: string,
  fileName: string,
  before: string,
  after: string,
): string {
  const clean = link.replace(/^\.\//, '');
  const alt = altFromFileName(fileName);
  const core = alt ? `![${alt}](${clean})` : `![](${clean})`;
  const prefix = before === '\n' ? '\n' : '\n\n';
  const suffix = after === '\n' ? '' : '\n';
  return `${prefix}${core}${suffix}`;
}

/** Read a `File` as a base64 payload (no `data:` prefix). */
export function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = String(reader.result ?? '');
      const idx = dataUrl.indexOf(',');
      resolve(idx >= 0 ? dataUrl.slice(idx + 1) : '');
    };
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(file);
  });
}
