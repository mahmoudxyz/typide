// CodeMirror 6 setup + a lightweight Typst stream highlighter.
// NOTE: the real Typide indexes with `typst-syntax`, never regex (ARCHITECTURE.md
// §7.1). This client-side highlighter is presentation-only for the preview UI.
import { EditorState, StateEffect, StateField, RangeSetBuilder, type Extension } from "@codemirror/state";
import {
  EditorView,
  lineNumbers,
  highlightActiveLine,
  highlightActiveLineGutter,
  drawSelection,
  keymap,
  gutter,
  hoverTooltip,
  Decoration,
  type DecorationSet,
  ViewPlugin,
  type ViewUpdate,
} from "@codemirror/view";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { searchKeymap, highlightSelectionMatches } from "@codemirror/search";
import {
  autocompletion,
  completionKeymap,
  type CompletionContext,
  type CompletionResult,
} from "@codemirror/autocomplete";
import {
  HighlightStyle,
  syntaxHighlighting,
  StreamLanguage,
  bracketMatching,
} from "@codemirror/language";
import { tags as t } from "@lezer/highlight";

// ── Typst stream tokenizer ────────────────────────────────────────────────
const KEYWORDS = new Set([
  "let", "set", "show", "import", "include", "if", "else", "for", "while",
  "return", "none", "auto", "true", "false", "in", "as", "context",
]);

const typst = StreamLanguage.define<{ inMath: boolean }>({
  startState: () => ({ inMath: false }),
  token(stream, state) {
    // Comments
    if (stream.match("//")) {
      stream.skipToEnd();
      return "comment";
    }
    if (stream.match("/*")) {
      while (!stream.eol()) {
        if (stream.match("*/")) break;
        stream.next();
      }
      return "comment";
    }
    // Math mode
    if (stream.peek() === "$") {
      stream.next();
      state.inMath = !state.inMath;
      return "punctuation";
    }
    if (state.inMath) {
      stream.next();
      return "operator"; // rendered with --syn-math
    }
    // Heading (line starts with one or more '=')
    if (stream.sol() && stream.match(/^\s*=+\s/)) {
      stream.skipToEnd();
      return "heading";
    }
    // Strings
    if (stream.match(/^"(?:[^"\\]|\\.)*"/)) return "string";
    // Label <name> and reference @name
    if (stream.match(/^<[\w-]+>/)) return "labelName";
    if (stream.match(/^@[\w:.-]+/)) return "labelName";
    // Function / code call  #name  or bare name(
    if (stream.match(/^#[a-zA-Z_][\w.-]*/)) {
      const w = stream.current().slice(1);
      return KEYWORDS.has(w) ? "keyword" : "function";
    }
    if (stream.match(/^[a-zA-Z_][\w-]*(?=\()/)) return "function";
    // Numbers with optional units
    if (stream.match(/^-?\d+(\.\d+)?(pt|mm|cm|in|em|fr|%|deg)?/)) return "number";
    // Bare identifiers → keyword check
    if (stream.match(/^[a-zA-Z_][\w-]*/)) {
      const w = stream.current();
      return KEYWORDS.has(w) ? "keyword" : null;
    }
    if (stream.match(/^[{}()\[\],;:#*_]/)) return "punctuation";
    stream.next();
    return null;
  },
  tokenTable: {
    heading: t.heading,
    function: t.function(t.variableName),
    labelName: t.labelName,
    operator: t.operator,
  },
});

// ── Highlight colors bound to CSS variables (adapt to light/dark) ──────────
const typideHighlight = HighlightStyle.define([
  { tag: t.keyword, color: "var(--syn-keyword)", fontWeight: "600" },
  { tag: [t.function(t.variableName), t.function(t.propertyName)], color: "var(--syn-func)" },
  { tag: t.string, color: "var(--syn-string)" },
  { tag: t.number, color: "var(--syn-number)" },
  { tag: t.comment, color: "var(--syn-comment)", fontStyle: "italic" },
  { tag: t.heading, color: "var(--syn-heading)", fontWeight: "700" },
  { tag: t.labelName, color: "var(--syn-label)" },
  { tag: t.operator, color: "var(--syn-math)" },
  { tag: t.punctuation, color: "var(--syn-punct)" },
]);

const typideTheme = EditorView.theme({
  "&": {
    color: "var(--fg-0)",
    backgroundColor: "var(--bg-2)",
    height: "100%",
    fontSize: "13px",
  },
  ".cm-scroller": {
    fontFamily: "'JetBrains Mono', ui-monospace, monospace",
    lineHeight: "1.65",
  },
  ".cm-content": { padding: "10px 0", caretColor: "var(--accent)" },
  "&.cm-focused": { outline: "none" },
  ".cm-gutters": {
    backgroundColor: "var(--bg-2)",
    color: "var(--fg-3)",
    border: "none",
    paddingRight: "6px",
  },
  ".cm-activeLineGutter": { backgroundColor: "transparent", color: "var(--fg-1)" },
  ".cm-activeLine": { backgroundColor: "color-mix(in srgb, var(--bg-4) 45%, transparent)" },
  ".cm-cursor": { borderLeftColor: "var(--accent)", borderLeftWidth: "2px" },
  ".cm-selectionBackground, ::selection": { backgroundColor: "var(--accent-soft) !important" },
  "&.cm-focused .cm-selectionBackground": { backgroundColor: "var(--accent-soft) !important" },
  ".cm-selectionMatch": { backgroundColor: "var(--warn-soft)" },
  ".cm-matchingBracket": {
    backgroundColor: "var(--accent-soft)",
    outline: "1px solid var(--accent)",
  },
  ".cm-tooltip": {
    background: "var(--bg-3)",
    border: "1px solid var(--border-strong)",
    borderRadius: "8px",
    boxShadow: "var(--shadow)",
    overflow: "hidden",
  },
  ".cm-tooltip.cm-tooltip-autocomplete > ul": {
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: "12.5px",
    maxHeight: "16em",
  },
  ".cm-tooltip.cm-tooltip-autocomplete > ul > li": {
    padding: "3px 8px",
    display: "flex",
    alignItems: "center",
    gap: "7px",
  },
  ".cm-tooltip-autocomplete ul li[aria-selected]": {
    background: "var(--accent-soft)",
    color: "var(--fg-0)",
  },
  ".cm-completionLabel": { color: "var(--fg-0)" },
  ".cm-completionDetail": { color: "var(--fg-3)", fontStyle: "normal", marginLeft: "auto" },
  ".cm-completionIcon": { opacity: "0.7", width: "1.1em" },
  ".cm-typide-hover": {
    padding: "7px 10px",
    fontSize: "12.5px",
    maxWidth: "360px",
    color: "var(--fg-1)",
    lineHeight: "1.5",
  },
  ".cm-typide-hover pre": {
    margin: "0",
    fontFamily: "'JetBrains Mono', monospace",
    fontSize: "12px",
    color: "var(--fg-0)",
    whiteSpace: "pre-wrap",
  },
  ".cm-spell-error": {
    textDecoration: "underline wavy var(--warn)",
    textDecorationSkipInk: "none",
    textUnderlineOffset: "2px",
  },
  ".cm-spell-tip": {
    padding: "8px 9px",
    display: "flex",
    flexWrap: "wrap",
    gap: "5px",
    maxWidth: "290px",
    alignItems: "center",
  },
  ".cm-spell-title": {
    width: "100%",
    fontSize: "11px",
    color: "var(--fg-3)",
    marginBottom: "3px",
  },
  ".cm-spell-sug": {
    padding: "3px 9px",
    borderRadius: "6px",
    background: "var(--accent-soft)",
    color: "var(--accent)",
    fontSize: "12px",
    fontFamily: "'JetBrains Mono', monospace",
    cursor: "pointer",
    border: "none",
  },
  ".cm-spell-none": { fontSize: "12px", color: "var(--fg-3)" },
  ".cm-spell-add": {
    width: "100%",
    marginTop: "5px",
    padding: "5px",
    fontSize: "11px",
    color: "var(--fg-2)",
    background: "var(--bg-4)",
    borderRadius: "6px",
    border: "none",
    cursor: "pointer",
  },
});

const langFor: Record<string, Extension> = {
  typst: typst,
  toml: [],
  bibtex: [],
  markdown: [],
  yaml: [],
};

export interface EditorHandle {
  view: EditorView;
  setDoc: (doc: string, lang: string) => void;
  destroy: () => void;
}

interface IdeCompletion {
  from: number;
  items: { kind: string; label: string; apply?: string | null; detail?: string | null }[];
}

const CM_TYPE: Record<string, string> = {
  func: "function",
  param: "property",
  type: "type",
  constant: "constant",
  label: "text",
  package: "namespace",
  symbol: "variable",
  path: "text",
  syntax: "keyword",
  font: "text",
};

// Strip typst-ide `${n:name}` / `${n}` snippet placeholders down to plain text.
function plainApply(it: { apply?: string | null; label: string }): string {
  const raw = it.apply ?? it.label;
  return raw.replace(/\$\{\d+:([^}]*)\}/g, "$1").replace(/\$\{\d+\}/g, "").replace(/\$\{\}/g, "");
}

// ── Spell checking (backend typide-check + a per-viewer user dictionary) ─────
export interface Misspelling {
  start: number;
  end: number;
  word: string;
  suggestions: string[];
}

function loadUserDict(): Set<string> {
  try {
    return new Set(JSON.parse(localStorage.getItem("typide.userdict") ?? "[]"));
  } catch {
    return new Set();
  }
}
const userDict = loadUserDict();
function addToDict(word: string) {
  userDict.add(word.toLowerCase());
  try {
    localStorage.setItem("typide.userdict", JSON.stringify([...userDict]));
  } catch {}
}

const spellMark = Decoration.mark({ class: "cm-spell-error" });
const setSpell = StateEffect.define<DecorationSet>();
const spellField = StateField.define<DecorationSet>({
  create: () => Decoration.none,
  update(deco, tr) {
    deco = deco.map(tr.changes);
    for (const e of tr.effects) if (e.is(setSpell)) deco = e.value;
    return deco;
  },
  provide: (f) => EditorView.decorations.from(f),
});

// The latest misspellings, read by the hover handler to show suggestions.
let currentMisspellings: Misspelling[] = [];

function buildSpellDeco(view: EditorView, ms: Misspelling[]): DecorationSet {
  const len = view.state.doc.length;
  const b = new RangeSetBuilder<Decoration>();
  for (const m of [...ms].sort((a, b) => a.start - b.start)) {
    if (m.start < m.end && m.end <= len) b.add(m.start, m.end, spellMark);
  }
  return b.finish();
}

function spellPlugin(cb: (text: string) => Promise<Misspelling[]>) {
  return ViewPlugin.fromClass(
    class {
      timer: ReturnType<typeof setTimeout> | undefined;
      constructor(view: EditorView) {
        this.schedule(view, 120);
      }
      update(u: ViewUpdate) {
        if (u.docChanged) this.schedule(u.view, 450);
      }
      schedule(view: EditorView, delay: number) {
        clearTimeout(this.timer);
        this.timer = setTimeout(() => this.run(view), delay);
      }
      async run(view: EditorView) {
        try {
          const text = view.state.doc.toString();
          const ms = (await cb(text)).filter((m) => !userDict.has(m.word.toLowerCase()));
          currentMisspellings = ms;
          view.dispatch({ effects: setSpell.of(buildSpellDeco(view, ms)) });
        } catch {
          /* ignore */
        }
      }
      destroy() {
        clearTimeout(this.timer);
      }
    }
  );
}

function spellTooltip(view: EditorView, miss: Misspelling): { dom: HTMLElement } {
  const dom = document.createElement("div");
  dom.className = "cm-spell-tip";
  const title = document.createElement("div");
  title.className = "cm-spell-title";
  title.textContent = `“${miss.word}”`;
  dom.appendChild(title);
  if (miss.suggestions.length) {
    for (const s of miss.suggestions) {
      const btn = document.createElement("button");
      btn.className = "cm-spell-sug";
      btn.textContent = s;
      btn.onmousedown = (e) => {
        e.preventDefault();
        view.dispatch({ changes: { from: miss.start, to: miss.end, insert: s } });
      };
      dom.appendChild(btn);
    }
  } else {
    const none = document.createElement("div");
    none.className = "cm-spell-none";
    none.textContent = "No suggestions";
    dom.appendChild(none);
  }
  const add = document.createElement("button");
  add.className = "cm-spell-add";
  add.textContent = "Add to dictionary";
  add.onmousedown = (e) => {
    e.preventDefault();
    addToDict(miss.word);
    currentMisspellings = currentMisspellings.filter(
      (m) => m.word.toLowerCase() !== miss.word.toLowerCase()
    );
    view.dispatch({ effects: setSpell.of(buildSpellDeco(view, currentMisspellings)) });
  };
  dom.appendChild(add);
  return { dom };
}

export function createEditor(opts: {
  parent: HTMLElement;
  doc: string;
  lang: string;
  onChange?: (doc: string) => void;
  onCursor?: (line: number, col: number) => void;
  complete?: (cursor: number, explicit: boolean) => Promise<IdeCompletion>;
  hover?: (cursor: number) => Promise<{ text: string; code: boolean } | null>;
  spell?: (text: string) => Promise<Misspelling[]>;
}): EditorHandle {
  const listener = EditorView.updateListener.of((u) => {
    if (u.docChanged) opts.onChange?.(u.state.doc.toString());
    if (u.selectionSet || u.docChanged) {
      const pos = u.state.selection.main.head;
      const line = u.state.doc.lineAt(pos);
      opts.onCursor?.(line.number, pos - line.from + 1);
    }
  });

  // Editor intelligence (completion + hover) backed by typst-ide.
  const ide: Extension[] = [];
  if (opts.complete) {
    const source = async (ctx: CompletionContext): Promise<CompletionResult | null> => {
      const before = ctx.matchBefore(/[#@\w.\-]+/);
      if (!ctx.explicit && !before) return null;
      try {
        const res = await opts.complete!(ctx.pos, ctx.explicit);
        if (!res.items.length) return null;
        return {
          from: res.from,
          options: res.items.map((it) => ({
            label: it.label,
            type: CM_TYPE[it.kind] ?? "text",
            detail: it.detail ?? undefined,
            apply: plainApply(it),
          })),
          validFor: /[#@\w.\-]*/,
        };
      } catch {
        return null;
      }
    };
    ide.push(autocompletion({ override: [source], activateOnTyping: true, icons: true }));
  }
  if (opts.spell) {
    ide.push(spellField, spellPlugin(opts.spell));
  }
  if (opts.hover || opts.spell) {
    ide.push(
      hoverTooltip(async (view, pos) => {
        // Spelling suggestions take priority when hovering a flagged word.
        const miss = currentMisspellings.find((m) => pos >= m.start && pos <= m.end);
        if (miss) {
          return { pos: miss.start, end: miss.end, above: true, create: (v) => spellTooltip(v, miss) };
        }
        if (!opts.hover) return null;
        try {
          const h = await opts.hover(pos);
          if (!h || !h.text) return null;
          return {
            pos,
            above: true,
            create: () => {
              const dom = document.createElement("div");
              dom.className = "cm-typide-hover";
              if (h.code) {
                const pre = document.createElement("pre");
                pre.textContent = h.text;
                dom.appendChild(pre);
              } else {
                dom.textContent = h.text;
              }
              return { dom };
            },
          };
        } catch {
          return null;
        }
      })
    );
  }

  const base: Extension[] = [
    lineNumbers(),
    gutter({ class: "cm-typide-gutter" }),
    highlightActiveLine(),
    highlightActiveLineGutter(),
    drawSelection(),
    history(),
    bracketMatching(),
    highlightSelectionMatches(),
    keymap.of([...defaultKeymap, ...historyKeymap, ...searchKeymap, ...completionKeymap, indentWithTab]),
    syntaxHighlighting(typideHighlight),
    typideTheme,
    ...ide,
    listener,
  ];

  let state = EditorState.create({
    doc: opts.doc,
    extensions: [...base, langFor[opts.lang] ?? []],
  });
  const view = new EditorView({ state, parent: opts.parent });

  return {
    view,
    setDoc(doc, lang) {
      state = EditorState.create({ doc, extensions: [...base, langFor[lang] ?? []] });
      view.setState(state);
    },
    destroy: () => view.destroy(),
  };
}
