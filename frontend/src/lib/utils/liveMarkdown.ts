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
  const tokenHiddenDeco = Decoration.mark({ class: 'cm-live-token-hidden' });
  const tokenActiveDeco = Decoration.mark({ class: 'cm-live-token-active' });
  const boldDeco = Decoration.mark({ class: 'cm-live-bold' });
  const italicDeco = Decoration.mark({ class: 'cm-live-italic' });
  const codeDeco = Decoration.mark({ class: 'cm-live-inlinecode' });
  const strikeDeco = Decoration.mark({ class: 'cm-live-strike' });
  const listMarkDeco = Decoration.mark({ class: 'cm-live-listmark' });
  const linkDeco = Decoration.mark({ class: 'cm-live-link' });

  const h1Deco = Decoration.line({ class: 'cm-live-h1' });
  const h2Deco = Decoration.line({ class: 'cm-live-h2' });
  const h3Deco = Decoration.line({ class: 'cm-live-h3' });
  const h4Deco = Decoration.line({ class: 'cm-live-h4' });
  const h5Deco = Decoration.line({ class: 'cm-live-h5' });
  const h6Deco = Decoration.line({ class: 'cm-live-h6' });
  const quoteDeco = Decoration.line({ class: 'cm-live-quote' });
  const hrDeco = Decoration.line({ class: 'cm-live-hr' });
  const codeBlockLineDeco = Decoration.line({ class: 'cm-live-codeblock' });
  const asciiLineDeco = Decoration.line({ class: 'cm-live-ascii' });
  const listItemDeco = Decoration.line({ class: 'cm-live-list-item' });
  const listItemIndent1Deco = Decoration.line({ class: 'cm-live-list-item-indent-1' });
  const listItemIndent2Deco = Decoration.line({ class: 'cm-live-list-item-indent-2' });
  const bulletGlyphDeco = Decoration.mark({ class: 'cm-live-bullet' });
  const BOX_REGEX = /[\u2500-\u257F\u2580-\u259F┌─┐│└┘├┤┬┴┼═║╔╗╚╝╠╣╦╩╬▲▼◀▶]/;

  function computeDecos(view: EditorView): DecorationSet {
    const doc = view.state.doc;
    const builder = new RangeSetBuilder<Decoration>();
    const tree = syntaxTree(view.state);
    const sel = view.state.selection.main;
    const cursorFrom = sel.from;
    const cursorTo = sel.to;
    const cursorLine = doc.lineAt(Math.min(cursorFrom, doc.length)).number;

    const items: Array<{ from: number; to: number; deco: Decoration; isLine: boolean }> = [];

    if (tree) {
      tree.iterate({
        enter: (node) => {
          if (node.from >= doc.length) return;
          const nodeTo = Math.min(node.to, doc.length);
          const nodeLine = doc.lineAt(node.from).number;
          const isCursorOnLine = nodeLine === cursorLine;
          const isCursorInside = cursorFrom >= node.from && cursorTo <= nodeTo;

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
          } else if (node.name === 'ATXHeading4') {
            const lineStart = doc.lineAt(node.from).from;
            items.push({ from: lineStart, to: lineStart, deco: h4Deco, isLine: true });
          } else if (node.name === 'ATXHeading5') {
            const lineStart = doc.lineAt(node.from).from;
            items.push({ from: lineStart, to: lineStart, deco: h5Deco, isLine: true });
          } else if (node.name === 'ATXHeading6') {
            const lineStart = doc.lineAt(node.from).from;
            items.push({ from: lineStart, to: lineStart, deco: h6Deco, isLine: true });
          } else if (node.name === 'Blockquote') {
            const lineStart = doc.lineAt(node.from).from;
            items.push({ from: lineStart, to: lineStart, deco: quoteDeco, isLine: true });
          } else if (node.name === 'HorizontalRule') {
            const lineStart = doc.lineAt(node.from).from;
            items.push({ from: lineStart, to: lineStart, deco: hrDeco, isLine: true });
          } else if (node.name === 'FencedCode') {
            const startLine = doc.lineAt(node.from).number;
            const endLine = doc.lineAt(nodeTo).number;
            for (let l = startLine; l <= endLine; l++) {
              const lineStart = doc.line(l).from;
              items.push({ from: lineStart, to: lineStart, deco: codeBlockLineDeco, isLine: true });
            }
          } else if (node.name === 'ListItem') {
            const line = doc.lineAt(node.from);
            const lineStart = line.from;
            const leading = line.text.match(/^\s*/)?.[0] || '';
            const indent = leading.replace(/\t/g, '  ').length;
            let deco = listItemDeco;
            if (indent >= 4) {
              deco = listItemIndent2Deco;
            } else if (indent >= 2) {
              deco = listItemIndent1Deco;
            }
            items.push({ from: lineStart, to: lineStart, deco, isLine: true });
          }

          // Header mark and Quote mark
          if (node.name === 'HeaderMark' || node.name === 'QuoteMark') {
            const nextChar = view.state.sliceDoc(nodeTo, nodeTo + 1);
            const to = nextChar === ' ' ? nodeTo + 1 : nodeTo;
            items.push({
              from: node.from,
              to: Math.min(to, doc.length),
              deco: isCursorOnLine ? tokenActiveDeco : tokenHiddenDeco,
              isLine: false,
            });
          }

          // List markers (- , * , 1. )
          if (node.name === 'ListMark') {
            const markText = view.state.sliceDoc(node.from, nodeTo);
            const isBullet = markText.includes('*') || markText.includes('-');
            if (isBullet && !isCursorOnLine) {
              items.push({
                from: node.from,
                to: nodeTo,
                deco: bulletGlyphDeco,
                isLine: false,
              });
            } else {
              items.push({
                from: node.from,
                to: nodeTo,
                deco: listMarkDeco,
                isLine: false,
              });
            }
          }

          // Links: [text](url)
          if (node.name === 'Link' && nodeTo - node.from > 4) {
            if (!isCursorInside) {
              items.push({
                from: node.from,
                to: nodeTo,
                deco: linkDeco,
                isLine: false,
              });
            }
          }

          // Inline styling: hide delimiters and apply rich text
          if (node.name === 'StrongEmphasis' && nodeTo - node.from >= 4) {
            items.push({
              from: node.from,
              to: node.from + 2,
              deco: isCursorInside ? tokenActiveDeco : tokenHiddenDeco,
              isLine: false,
            });
            items.push({
              from: node.from + 2,
              to: nodeTo - 2,
              deco: boldDeco,
              isLine: false,
            });
            items.push({
              from: nodeTo - 2,
              to: nodeTo,
              deco: isCursorInside ? tokenActiveDeco : tokenHiddenDeco,
              isLine: false,
            });
          } else if (node.name === 'Emphasis' && nodeTo - node.from >= 2) {
            items.push({
              from: node.from,
              to: node.from + 1,
              deco: isCursorInside ? tokenActiveDeco : tokenHiddenDeco,
              isLine: false,
            });
            items.push({
              from: node.from + 1,
              to: nodeTo - 1,
              deco: italicDeco,
              isLine: false,
            });
            items.push({
              from: nodeTo - 1,
              to: nodeTo,
              deco: isCursorInside ? tokenActiveDeco : tokenHiddenDeco,
              isLine: false,
            });
          } else if (node.name === 'InlineCode' && nodeTo - node.from >= 2) {
            items.push({
              from: node.from,
              to: node.from + 1,
              deco: isCursorInside ? tokenActiveDeco : tokenHiddenDeco,
              isLine: false,
            });
            items.push({
              from: node.from + 1,
              to: nodeTo - 1,
              deco: codeDeco,
              isLine: false,
            });
            items.push({
              from: nodeTo - 1,
              to: nodeTo,
              deco: isCursorInside ? tokenActiveDeco : tokenHiddenDeco,
              isLine: false,
            });
          } else if (node.name === 'Strikethrough' && nodeTo - node.from >= 4) {
            items.push({
              from: node.from,
              to: node.from + 2,
              deco: isCursorInside ? tokenActiveDeco : tokenHiddenDeco,
              isLine: false,
            });
            items.push({
              from: node.from + 2,
              to: nodeTo - 2,
              deco: strikeDeco,
              isLine: false,
            });
            items.push({
              from: nodeTo - 2,
              to: nodeTo,
              deco: isCursorInside ? tokenActiveDeco : tokenHiddenDeco,
              isLine: false,
            });
          }
        },
      });
    }

    // Detect lines containing ASCII diagram box-drawing characters
    for (let i = 1; i <= doc.lines; i++) {
      const line = doc.line(i);
      if (BOX_REGEX.test(line.text)) {
        items.push({ from: line.from, to: line.from, deco: asciiLineDeco, isLine: true });
      }
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
      if (it.from < 0 || it.to > doc.length || it.from > it.to) continue;
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

    try {
      return builder.finish();
    } catch {
      return Decoration.none;
    }
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
