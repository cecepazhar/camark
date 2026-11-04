/**
 * Lightweight syntax highlighting for the preview using the Lezer parsers that CodeMirror
 * already ships (no extra dependency). Output is built only from escaped text + <span>.
 * `@lezer/highlight` is a transitive dependency of @codemirror/language.
 */
import { highlightTree, classHighlighter } from '@lezer/highlight';
import type { Parser } from '@lezer/common';
import { escapeHtml } from './markdownRender';

async function parserFor(lang: string): Promise<Parser | null> {
  switch (lang.toLowerCase()) {
    case 'js': case 'javascript': case 'jsx': case 'mjs': case 'cjs':
    case 'ts': case 'typescript': case 'tsx': {
      const m = await import('@codemirror/lang-javascript');
      return m.javascript({ jsx: true, typescript: true }).language.parser;
    }
    case 'json': case 'jsonc':
      return (await import('@codemirror/lang-json')).json().language.parser;
    case 'css':
      return (await import('@codemirror/lang-css')).css().language.parser;
    case 'html': case 'htm': case 'svelte': case 'vue':
      return (await import('@codemirror/lang-html')).html().language.parser;
    case 'py': case 'python':
      return (await import('@codemirror/lang-python')).python().language.parser;
    case 'rs': case 'rust':
      return (await import('@codemirror/lang-rust')).rust().language.parser;
    case 'md': case 'markdown':
      return (await import('@codemirror/lang-markdown')).markdown().language.parser;
    default:
      return null;
  }
}

/** Returns highlighted HTML (inner HTML of <code>), or null if the language is unsupported. */
export async function highlightToHtml(code: string, lang: string): Promise<string | null> {
  const parser = await parserFor(lang);
  if (!parser) return null;
  const tree = parser.parse(code);
  let out = '';
  let pos = 0;
  highlightTree(tree, classHighlighter, (from, to, classes) => {
    if (from > pos) out += escapeHtml(code.slice(pos, from));
    out += `<span class="${escapeHtml(classes)}">${escapeHtml(code.slice(from, to))}</span>`;
    pos = to;
  });
  if (pos < code.length) out += escapeHtml(code.slice(pos));
  return out;
}
