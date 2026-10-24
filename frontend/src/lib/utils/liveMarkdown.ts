import {
  EditorView,
  ViewPlugin,
  type ViewUpdate,
  Decoration,
  type DecorationSet,
} from '@codemirror/view';
import { RangeSetBuilder } from '@codemirror/state';
import { syntaxTree } from '@codemirror/language';

export function createLiveMarkdownPlugin() {
  const replaceDeco = Decoration.replace({});
  const boldDeco = Decoration.mark({ class: 'cm-live-bold' });
  const italicDeco = Decoration.mark({ class: 'cm-live-italic' });
  const codeDeco = Decoration.mark({ class: 'cm-live-inlinecode' });
  const strikeDeco = Decoration.mark({ class: 'cm-live-strike' });

  const h1Deco = Decoration.line({ class: 'cm-live-h1' });
  const h2Deco = Decoration.line({ class: 'cm-live-h2' });
  const h3Deco = Decoration.line({ class: 'cm-live-h3' });
  const quoteDeco = Decoration.line({ class: 'cm-live-quote' });
  const hrDeco = Decoration.line({ class: 'cm-live-hr' });

  function computeDecos(view: EditorView): DecorationSet {
    const doc = view.state.doc;
    const builder = new RangeSetBuilder<Decoration>();
    const tree = syntaxTree(view.state);
    const sel = view.state.selection.main;
    const cursorFrom = sel.from;
    const cursorTo = sel.to;
    const cursorLine = doc.lineAt(cursorFrom).number;

    const items: Array<{ from: number; to: number; deco: Decoration; isLine: boolean }> = [];

    if (tree) {
      tree.iterate({
        enter: (node) => {
          const nodeLine = doc.lineAt(node.from).number;
          const isCursorOnLine = nodeLine === cursorLine;
          const isCursorInside = cursorFrom >= node.from && cursorTo <= node.to;

          // Headings lines
          if (node.name === 'ATXHeading1') {
            const lineStart = doc.lineAt(node.from).from;
            items.push({ from: lineStart, to: lineStart, deco: h1Deco, isLine: true });
          } else if (node.name === 'ATXHeading2') {
            const lineStart = doc.lineAt(node.from).from;
            items.push({ from: lineStart, to: lineStart, deco: h2Deco, isLine: true });
          } else if (node.name === 'ATXHeading3') {
            const lineStart = doc.lineAt(node.from).from;
            items.push({ from: lineStart, to: lineStart, deco: h3Deco, isLine: true });
          } else if (node.name === 'Blockquote') {
            const lineStart = doc.lineAt(node.from).from;
            items.push({ from: lineStart, to: lineStart, deco: quoteDeco, isLine: true });
          } else if (node.name === 'HorizontalRule') {
            const lineStart = doc.lineAt(node.from).from;
            items.push({ from: lineStart, to: lineStart, deco: hrDeco, isLine: true });
          }

          // Hide heading and quote prefix marks when cursor is NOT on that line
          if (node.name === 'HeaderMark' && !isCursorOnLine) {
            const nextChar = view.state.sliceDoc(node.to, node.to + 1);
            const to = nextChar === ' ' ? node.to + 1 : node.to;
            items.push({ from: node.from, to, deco: replaceDeco, isLine: false });
          } else if (node.name === 'QuoteMark' && !isCursorOnLine) {
            const nextChar = view.state.sliceDoc(node.to, node.to + 1);
            const to = nextChar === ' ' ? node.to + 1 : node.to;
            items.push({ from: node.from, to, deco: replaceDeco, isLine: false });
          }

          // Inline styling: hide delimiters and apply rich text when cursor is NOT inside
          if (!isCursorInside) {
            if (node.name === 'StrongEmphasis') {
              if (node.to - node.from > 4) {
                items.push({ from: node.from, to: node.from + 2, deco: replaceDeco, isLine: false });
                items.push({ from: node.from + 2, to: node.to - 2, deco: boldDeco, isLine: false });
                items.push({ from: node.to - 2, to: node.to, deco: replaceDeco, isLine: false });
              }
            } else if (node.name === 'Emphasis') {
              if (node.to - node.from > 2) {
                items.push({ from: node.from, to: node.from + 1, deco: replaceDeco, isLine: false });
                items.push({ from: node.from + 1, to: node.to - 1, deco: italicDeco, isLine: false });
                items.push({ from: node.to - 1, to: node.to, deco: replaceDeco, isLine: false });
              }
            } else if (node.name === 'InlineCode') {
              if (node.to - node.from > 2) {
                items.push({ from: node.from, to: node.from + 1, deco: replaceDeco, isLine: false });
                items.push({ from: node.from + 1, to: node.to - 1, deco: codeDeco, isLine: false });
                items.push({ from: node.to - 1, to: node.to, deco: replaceDeco, isLine: false });
              }
            } else if (node.name === 'Strikethrough') {
              if (node.to - node.from > 4) {
                items.push({ from: node.from, to: node.from + 2, deco: replaceDeco, isLine: false });
                items.push({ from: node.from + 2, to: node.to - 2, deco: strikeDeco, isLine: false });
                items.push({ from: node.to - 2, to: node.to, deco: replaceDeco, isLine: false });
              }
            }
          }
        },
      });
    }

    // Sort order:
    // 1. `from` ascending
    // 2. Line decorations before mark decorations at the same `from`
    // 3. `to` ascending
    items.sort((a, b) => {
      if (a.from !== b.from) return a.from - b.from;
      if (a.isLine !== b.isLine) return a.isLine ? -1 : 1;
      return a.to - b.to;
    });

    const seenLines = new Set<number>();
    for (const it of items) {
      if (it.isLine) {
        if (seenLines.has(it.from)) continue;
        seenLines.add(it.from);
      }
      try {
        builder.add(it.from, it.to, it.deco);
      } catch {
        // Safe guard against out-of-order edges
      }
    }

    return builder.finish();
  }

  return ViewPlugin.fromClass(
    class {
      decorations: DecorationSet;
      constructor(view: EditorView) {
        this.decorations = computeDecos(view);
      }
      update(update: ViewUpdate) {
        if (
          update.docChanged ||
          update.viewportChanged ||
          update.selectionSet ||
          update.focusChanged
        ) {
          this.decorations = computeDecos(update.view);
        }
      }
    },
    {
      decorations: (v) => v.decorations,
    }
  );
}
