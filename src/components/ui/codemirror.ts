import {
  HighlightStyle,
  bracketMatching,
  foldGutter,
  syntaxHighlighting,
} from "@codemirror/language";
import { EditorView, highlightActiveLine, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { highlightSelectionMatches, search, searchKeymap } from "@codemirror/search";
import { tags } from "@lezer/highlight";

const highlight = HighlightStyle.define([
  { tag: tags.propertyName, color: "var(--patch)" },
  { tag: tags.string, color: "var(--get)" },
  { tag: tags.number, color: "var(--post)" },
  { tag: [tags.bool, tags.null], color: "var(--put)" },
  { tag: [tags.punctuation, tags.bracket], color: "var(--fg-muted)" },
]);

const theme = EditorView.theme({
  "&": { height: "100%", fontSize: "12px", backgroundColor: "var(--surface)", color: "var(--fg)" },
  "&.cm-focused": { outline: "none" },
  ".cm-scroller": { fontFamily: "var(--font-code)", lineHeight: "19px" },
  ".cm-gutters": { backgroundColor: "var(--surface)", color: "var(--fg-subtle)", border: "none" },
  ".cm-activeLine, .cm-activeLineGutter": { backgroundColor: "var(--surface-2)" },
  ".cm-cursor": { borderLeftColor: "var(--fg)" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection": {
    backgroundColor: "var(--accent-soft) !important",
  },
  ".cm-panels": { backgroundColor: "var(--surface-2)", color: "var(--fg)" },
  ".cm-searchMatch": { backgroundColor: "var(--accent-soft)" },
});

export const baseExtensions = [
  lineNumbers(),
  foldGutter(),
  history(),
  bracketMatching(),
  highlightActiveLine(),
  highlightSelectionMatches(),
  search({ top: true }),
  syntaxHighlighting(highlight),
  keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap, indentWithTab]),
  EditorView.lineWrapping,
  theme,
];
