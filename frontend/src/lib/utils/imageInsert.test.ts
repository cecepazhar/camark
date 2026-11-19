import { describe, it, expect } from 'vitest';
import {
  isAllowedImage,
  MAX_IMAGE_BYTES,
  buildImageMarkdown,
  fileToBase64,
  ALLOWED_IMAGE_MIMES,
} from './imageInsert';

describe('I-01 image insert helpers', () => {
  describe('isAllowedImage', () => {
    it('accepts a whitelisted image under the size cap', () => {
      expect(isAllowedImage('image/png', 1024)).toBe(true);
      expect(isAllowedImage('image/jpeg', 1024)).toBe(true);
      expect(isAllowedImage('image/webp', 1024)).toBe(true);
    });

    it('rejects non-image / unknown types', () => {
      expect(isAllowedImage('application/octet-stream', 1024)).toBe(false);
      expect(isAllowedImage('image/x-exotic', 1024)).toBe(false);
      expect(isAllowedImage('', 1024)).toBe(false);
    });

    it('rejects zero-byte and oversized payloads', () => {
      expect(isAllowedImage('image/png', 0)).toBe(false);
      expect(isAllowedImage('image/png', MAX_IMAGE_BYTES + 1)).toBe(false);
      expect(isAllowedImage('image/png', MAX_IMAGE_BYTES)).toBe(true);
    });

    it('mirrors the backend allow-list exactly', () => {
      // Keep the client gate in lockstep with the Rust allow-list.
      expect(ALLOWED_IMAGE_MIMES).toContain('image/svg+xml');
      expect(ALLOWED_IMAGE_MIMES).toHaveLength(9);
    });
  });

  describe('buildImageMarkdown', () => {
    it('wraps the image in blank lines when caret is mid-paragraph', () => {
      const out = buildImageMarkdown('assets/a.png', 'a.png', 'x', 'y');
      expect(out).toBe('\n\n![a](assets/a.png)\n');
    });

    it('uses a single leading newline when already on its own line', () => {
      const out = buildImageMarkdown('assets/a.png', 'a.png', '\n', 'y');
      expect(out).toBe('\n![a](assets/a.png)\n');
    });

    it('omits the trailing newline when already followed by a newline', () => {
      const out = buildImageMarkdown('assets/a.png', 'a.png', 'x', '\n');
      expect(out).toBe('\n\n![a](assets/a.png)');
    });

    it('strips the extension from the alt text and drops alt for empty name', () => {
      expect(buildImageMarkdown('assets/a.png', 'a.png', ' ', ' ')).toContain('![a]');
      expect(buildImageMarkdown('assets/a.png', '', ' ', ' ')).toBe('\n\n![](assets/a.png)\n');
    });

    it('normalizes a leading ./ prefix on the link', () => {
      expect(buildImageMarkdown('./assets/a.png', 'a.png', ' ', ' ')).toContain('(assets/a.png)');
    });
  });

  describe('fileToBase64', () => {
    it('strips the data: prefix and returns pure base64', async () => {
      // "hello" -> base64 "aGVsbG8="
      const bytes = new Uint8Array([104, 101, 108, 108, 111]);
      const file = new File([bytes], 'hello.txt', { type: 'text/plain' });
      const b64 = await fileToBase64(file);
      expect(b64).toBe('aGVsbG8=');
      expect(b64).not.toContain(':');
    });
  });
});
