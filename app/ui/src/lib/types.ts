// Domain types mirrored from the Rust backend (ARCHITECTURE.md §6, §18.2).
// The frontend is a read-through view; the backend is the source of truth.

export type Severity = "error" | "warning" | "hint";
export type ProjectKind = "thesis" | "paper" | "notes" | "package" | "generic";
export type NetworkMode = "offline" | "ask" | "online";

export interface FileNode {
  name: string;
  path: string;
  kind: "dir" | "typ" | "bib" | "md" | "toml" | "yml" | "font" | "pdf" | "other";
  children?: FileNode[];
  badge?: string; // e.g. "entry", "vendored"
}

export interface EditorFile {
  path: string;
  name: string;
  language: "typst" | "bibtex" | "toml" | "markdown" | "yaml";
  content: string;
  dirty?: boolean;
}

export interface Diagnostic {
  id: string; // inspection id, e.g. "broken-ref"
  file: string;
  line: number;
  col: number;
  severity: Severity;
  message: string;
  source: "typide" | "tinymist" | "typst";
  quickFix?: string;
}

export interface Job {
  id: number;
  title: string;
  fraction: number; // 0..1
  message: string;
  state: "running" | "done" | "failed" | "cancelled";
}

export interface StructureNode {
  label: string;
  kind: "heading" | "figure" | "table" | "equation" | "label" | "rule";
  line: number;
  level?: number;
  children?: StructureNode[];
}

export interface PackageInfo {
  spec: string; // @preview/cetz:0.4.2
  name: string;
  version: string;
  latest?: string;
  status: "vendored" | "cached" | "missing" | "mirror";
  description: string;
}

export interface Citation {
  key: string;
  title: string;
  authors: string;
  year: string;
  source: "bib" | "zotero";
  cited: boolean;
}

export interface Note {
  path: string;
  title: string;
  citeKey?: string;
  backlinks: string[];
  outgoing: string[];
}

export interface ExportProfile {
  name: string;
  format: "pdf" | "html" | "svg" | "png";
  output: string;
  standards?: string[];
}

export interface Snapshot {
  id: string;
  message: string;
  when: string;
  auto: boolean;
}

export interface PreviewPage {
  n: number;
  changed: boolean;
}

export interface ProjectInfo {
  name: string;
  kind: string; // generator/template id from the backend
  root: string;
  entrypoint: string;
  typst: string;
  tinymist: string;
  branch: string;
}
