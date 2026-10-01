// Lightweight, live source analysis for the UI panels (structure, problems,
// packages, citations). This is presentation-layer analysis so the panels
// reflect real file content dynamically; the authoritative index in the Rust
// backend uses `typst-syntax` (ARCHITECTURE.md §7.1).
import type { Citation, Diagnostic, PackageInfo, StructureNode } from "./types";

export interface ScannedFile {
  path: string;
  rel: string;
  kind: string;
  content: string;
}

function lineColAt(content: string, index: number): { line: number; col: number } {
  let line = 1;
  let last = 0;
  for (let i = 0; i < index; i++) {
    if (content[i] === "\n") {
      line++;
      last = i + 1;
    }
  }
  return { line, col: index - last + 1 };
}

const LABEL_RE = /<([A-Za-z][\w-]*(?:[.:][\w-]+)*)>/g;
// A reference name may contain '.', ':' and '-' internally, but must not
// swallow trailing prose punctuation (e.g. the '.' ending a sentence).
const REF_RE = /(?<![A-Za-z0-9_@"/])@([A-Za-z][\w-]*(?:[.:][A-Za-z0-9_-]+)*)/g;
const IMPORT_RE = /@([\w-]+)\/([\w-]+):(\d+\.\d+\.\d+)/g;

export interface Occurrence {
  name: string;
  file: string;
  rel: string;
  line: number;
  col: number;
}

export function collectLabels(files: ScannedFile[]): Set<string> {
  const set = new Set<string>();
  for (const f of files) {
    if (f.kind !== "typ") continue;
    for (const m of f.content.matchAll(LABEL_RE)) set.add(m[1]);
  }
  return set;
}

export function collectRefs(files: ScannedFile[]): Occurrence[] {
  const out: Occurrence[] = [];
  for (const f of files) {
    if (f.kind !== "typ") continue;
    for (const m of f.content.matchAll(REF_RE)) {
      const { line, col } = lineColAt(f.content, m.index ?? 0);
      out.push({ name: m[1], file: f.path, rel: f.rel, line, col });
    }
  }
  return out;
}

export function parseBib(files: ScannedFile[]): Citation[] {
  const cites: Citation[] = [];
  const entryRe = /@(\w+)\s*\{\s*([^,\s]+)\s*,([\s\S]*?)\n\}/g;
  for (const f of files) {
    if (f.kind !== "bib") continue;
    for (const m of f.content.matchAll(entryRe)) {
      const body = m[3];
      const field = (k: string) => {
        const r = new RegExp(k + "\\s*=\\s*[\\{\"]([^\\}\"]*)[\\}\"]", "i");
        return body.match(r)?.[1]?.trim() ?? "";
      };
      const authorsRaw = field("author");
      const authors = authorsRaw
        .split(/\s+and\s+/)
        .map((a) => a.split(",")[0].trim())
        .filter(Boolean)
        .slice(0, 2)
        .join(", ");
      cites.push({
        key: m[2].trim(),
        title: field("title") || "(untitled)",
        authors: authors || "—",
        year: field("year") || "",
        source: "bib",
        cited: false,
      });
    }
  }
  return cites;
}

export function collectPackages(files: ScannedFile[]): PackageInfo[] {
  const seen = new Map<string, PackageInfo>();
  for (const f of files) {
    if (f.kind !== "typ") continue;
    for (const m of f.content.matchAll(IMPORT_RE)) {
      const [spec, ns, name, ver] = [m[0], m[1], m[2], m[3]];
      if (seen.has(spec)) continue;
      seen.set(spec, {
        spec: `@${ns}/${name}:${ver}`,
        name,
        version: ver,
        status: ns === "preview" ? "cached" : "vendored",
        description: ns === "preview" ? "Typst Universe package." : `Package in @${ns} namespace.`,
      });
    }
  }
  return [...seen.values()];
}

/** Structure of a single file: headings with nested figures/labels. */
export function computeStructure(content: string): StructureNode[] {
  const lines = content.split("\n");
  const roots: StructureNode[] = [];
  let current: StructureNode | null = null;

  lines.forEach((raw, i) => {
    const line = i + 1;
    const h = raw.match(/^(=+)\s+(.*)$/);
    if (h) {
      current = { label: h[2].replace(/<[\w:.-]+>\s*$/, "").trim(), kind: "heading", line, level: h[1].length, children: [] };
      roots.push(current);
      return;
    }
    const target = current ?? null;
    if (/#figure\b/.test(raw)) {
      const node: StructureNode = { label: "Figure", kind: "figure", line };
      (target?.children ?? roots).push(node);
    }
    if (/#table\b/.test(raw)) {
      (target?.children ?? roots).push({ label: "Table", kind: "table", line });
    }
    const lbl = raw.match(/<([A-Za-z][\w:.-]*)>/);
    if (lbl) {
      (target?.children ?? roots).push({ label: `<${lbl[1]}>`, kind: "label", line });
    }
  });
  return roots;
}

/** Project-wide diagnostics from real content (ARCHITECTURE.md §14.2). */
export function computeDiagnostics(files: ScannedFile[]): Diagnostic[] {
  const labels = collectLabels(files);
  const bib = parseBib(files);
  const bibKeys = new Set(bib.map((c) => c.key));
  const refs = collectRefs(files);
  const usedKeys = new Set(refs.map((r) => r.name));
  const diags: Diagnostic[] = [];

  for (const r of refs) {
    if (!labels.has(r.name) && !bibKeys.has(r.name)) {
      diags.push({
        id: "broken-ref",
        file: r.file,
        line: r.line,
        col: r.col,
        severity: "error",
        message: `Reference @${r.name} has no matching label or citation in the project.`,
        source: "typide",
        quickFix: "Create label / pick similar",
      });
    }
  }
  for (const c of bib) {
    if (!usedKeys.has(c.key)) {
      const f = files.find((x) => x.kind === "bib");
      diags.push({
        id: "uncited-entry",
        file: f?.path ?? "",
        line: 1,
        col: 1,
        severity: "warning",
        message: `Bibliography entry \`${c.key}\` is never cited.`,
        source: "typide",
        quickFix: "Remove / ignore",
      });
    }
  }
  return diags;
}

/** Mark which citations are actually referenced. */
export function markCited(cites: Citation[], files: ScannedFile[]): Citation[] {
  const used = new Set(collectRefs(files).map((r) => r.name));
  return cites.map((c) => ({ ...c, cited: used.has(c.key) }));
}
