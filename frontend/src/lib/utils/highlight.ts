/**
 * Lightweight syntax highlighting for the preview using the Lezer parsers that CodeMirror
 * already ships (no extra dependency). Output is built only from escaped text + <span>.
 * `@lezer/highlight` is a transitive dependency of @codemirror/language.
 */
import { highlightTree, classHighlighter } from '@lezer/highlight';
import { escapeHtml } from './markdownRender';

type HighlightableTree = Parameters<typeof highlightTree>[0];

interface HighlightParser {
  parse(input: string): HighlightableTree;
}

async function parserFor(lang: string): Promise<HighlightParser | null> {
  switch (lang.toLowerCase()) {
    case 'js': case 'javascript': case 'jsx': case 'mjs': case 'cjs':
    case 'ts': case 'typescript': case 'tsx': {
      const m = await import('@codemirror/lang-javascript');
      return m.javascript({ jsx: true, typescript: true }).language.parser as unknown as HighlightParser;
    }
    case 'json': case 'jsonc':
      return (await import('@codemirror/lang-json')).json().language.parser as unknown as HighlightParser;
    case 'css':
      return (await import('@codemirror/lang-css')).css().language.parser as unknown as HighlightParser;
    case 'html': case 'htm': case 'svelte': case 'vue':
      return (await import('@codemirror/lang-html')).html().language.parser as unknown as HighlightParser;
    case 'py': case 'python':
      return (await import('@codemirror/lang-python')).python().language.parser as unknown as HighlightParser;
    case 'rs': case 'rust':
      return (await import('@codemirror/lang-rust')).rust().language.parser as unknown as HighlightParser;
    case 'go':
      return (await import('@codemirror/lang-go')).go().language.parser as unknown as HighlightParser;
    case 'md': case 'markdown':
      return (await import('@codemirror/lang-markdown')).markdown().language.parser as unknown as HighlightParser;
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
