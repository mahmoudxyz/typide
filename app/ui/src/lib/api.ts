// The single seam between the frontend and the Rust backend (ARCHITECTURE.md
// §18.2/§18.4). Inside Tauri, calls hit real filesystem commands; in a plain
// browser (dev) they run against an in-memory demo project so the UI still works.
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { open as tauriOpen } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import { homeDir as tauriHomeDir } from "@tauri-apps/api/path";
import type { FileNode, ProjectInfo } from "./types";
import type { ScannedFile } from "./analysis";

export const isTauri = (): boolean =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

export interface JobStarted { job: number; title: string }
export interface JobProgress { job: number; done: number; total: number; message: string }
export interface JobFinished { job: number; ok: boolean; message: string }

/** Subscribe to background-job events; returns an unlisten function. */
export async function setupJobEvents(h: {
  started: (e: JobStarted) => void;
  progress: (e: JobProgress) => void;
  finished: (e: JobFinished) => void;
}): Promise<() => void> {
  if (!isTauri()) return () => {};
  const u1 = await listen<JobStarted>("job://started", (e) => h.started(e.payload));
  const u2 = await listen<JobProgress>("job://progress", (e) => h.progress(e.payload));
  const u3 = await listen<JobFinished>("job://finished", (e) => h.finished(e.payload));
  return () => { u1(); u2(); u3(); };
}

export interface Toolchain {
  version: string;
  source: string; // bundled | system | installed
  path: string;
}

export interface Policy {
  network_mode?: string | null;
  allow_hosts: string[];
  updates_enabled?: boolean | null;
  ai_enabled?: boolean | null;
  source?: string | null;
}

export interface PackageMeta {
  name: string;
  version: string;
  description: string;
  authors: string[];
  license: string;
  keywords: string[];
  categories: string[];
}
export interface FontFamily {
  name: string;
  variants: number;
  styles: string[];
  variable: boolean;
}
export interface OfflineReport {
  packages: { spec: string; sha256: string; vendored: boolean }[];
  errors: string[];
}

export interface Completion {
  kind: string;
  label: string;
  apply?: string | null;
  detail?: string | null;
}
export interface CompleteResponse {
  from: number;
  items: Completion[];
}
export interface Hover {
  text: string;
  code: boolean;
}
export interface Overlay {
  path: string;
  content: string;
}
export interface Misspelling {
  start: number;
  end: number;
  word: string;
  suggestions: string[];
}
export interface JumpTarget {
  file: string;
  offset: number;
  line: number;
}
export interface PreviewPosition {
  page: number;
  x: number;
  y: number;
}

export interface CompilerDiagnostic {
  severity: string;
  file: string;
  line: number;
  col: number;
  message: string;
  hints: string[];
}
export interface CompileResult {
  pages: string[]; // one SVG per page
  diagnostics: CompilerDiagnostic[];
  page_count: number;
  compile_ms: number;
}

export interface CreateParams {
  path: string;
  templateId: string;
  name: string;
  typst: string;
  packagesMode: "vendor" | "cache";
  gitInit: boolean;
  refs: boolean;
  vault: boolean;
  templateSpec?: string;
  online?: boolean;
}

// ── Browser demo virtual filesystem ────────────────────────────────────────
const DEMO_ROOT = "/demo/sample-paper";
const demoFiles: Record<string, string> = {
  "main.typ": `#set document(title: "A Short Paper", author: "You")
#set page(paper: "a4", margin: 2.2cm, numbering: "1")
#set text(size: 10.5pt, lang: "en")
#set par(justify: true)
#set heading(numbering: "1.")

#align(center)[
  #text(17pt, weight: "bold")[A Short Paper on Typide]
  #v(6pt)
  #text(11pt)[Your Name]
]
#v(10pt)

= Introduction <intro>
Typide is an offline-first IDE for Typst @knuth1984. As shown in
@fig-demo, the workflow is simple. See @results for the outcome.

#figure(
  rect(width: 70%, height: 2.5cm, fill: luma(240)),
  caption: [A demonstration figure.],
) <fig-demo>

= Results <results>
Everything is live and editable — try typing here.

#bibliography("refs.bib", style: "ieee")
`,
  "refs.bib": `@book{knuth1984,
  title  = {The TeXbook},
  author = {Knuth, Donald E.},
  year   = {1984},
  publisher = {Addison-Wesley}
}

@article{unused2020,
  title  = {An Uncited Work},
  author = {Doe, Jane},
  year   = {2020}
}`,
  "typide.toml": `[project]
name = "sample-paper"
kind = "paper"
entrypoint = "main.typ"

[toolchain]
typst = "0.15.1"
`,
};

function demoProject(): ProjectInfo {
  return {
    name: "sample-paper",
    kind: "paper",
    root: DEMO_ROOT,
    entrypoint: "main.typ",
    typst: "0.15.1",
    tinymist: "0.13.10",
    branch: "main",
  };
}
function demoTree(): FileNode {
  const abs = (rel: string) => `${DEMO_ROOT}/${rel}`;
  return {
    name: "sample-paper",
    path: DEMO_ROOT,
    kind: "dir",
    children: [
      { name: "main.typ", path: abs("main.typ"), kind: "typ", badge: "entry" },
      { name: "refs.bib", path: abs("refs.bib"), kind: "bib" },
      { name: "typide.toml", path: abs("typide.toml"), kind: "toml" },
    ],
  };
}
function demoScan(): ScannedFile[] {
  return Object.entries(demoFiles).map(([rel, content]) => ({
    path: `${DEMO_ROOT}/${rel}`,
    rel,
    kind: rel.endsWith(".bib") ? "bib" : rel.endsWith(".md") ? "md" : "typ",
    content,
  }));
}
const relOf = (path: string) => path.replace(`${DEMO_ROOT}/`, "");

function demoPackages(query: string): PackageMeta[] {
  const all: PackageMeta[] = [
    { name: "cetz", version: "0.4.2", description: "Draw with a TikZ-like API.", authors: ["johannes-wolf"], license: "MIT", keywords: ["draw", "tikz"], categories: ["visualization"] },
    { name: "fletcher", version: "0.5.8", description: "Diagrams with nodes and arrows.", authors: ["Jollywatt"], license: "MIT", keywords: ["diagram", "arrows"], categories: ["visualization"] },
    { name: "tablex", version: "0.0.9", description: "More powerful tables.", authors: ["PgBiel"], license: "MIT", keywords: ["table"], categories: ["layout"] },
    { name: "glossarium", version: "0.5.4", description: "Glossaries and acronyms.", authors: ["Duskflower"], license: "MIT", keywords: ["glossary"], categories: ["utility"] },
    { name: "lovelace", version: "0.3.0", description: "Pseudocode / algorithms.", authors: ["andreasKroepelin"], license: "MIT", keywords: ["algorithm"], categories: ["scripting"] },
  ];
  const q = query.trim().toLowerCase();
  return q ? all.filter((p) => p.name.includes(q) || p.description.toLowerCase().includes(q)) : all;
}
function demoFonts(): FontFamily[] {
  return [
    { name: "New Computer Modern", variants: 8, styles: ["Regular", "Italic"], variable: false },
    { name: "New Computer Modern Math", variants: 2, styles: ["Regular"], variable: false },
    { name: "Libertinus Serif", variants: 6, styles: ["Regular", "Italic"], variable: false },
    { name: "DejaVu Sans Mono", variants: 4, styles: ["Regular", "Oblique"], variable: false },
  ];
}

// ── Public API ─────────────────────────────────────────────────────────────
export const api = {
  /// The user's home directory (absolute). Falls back to "~" in a plain browser.
  async homeDir(): Promise<string> {
    if (!isTauri()) return "~";
    try {
      return (await tauriHomeDir()).replace(/[/\\]$/, "");
    } catch {
      return "~";
    }
  },

  async pickFolder(): Promise<string | null> {
    if (!isTauri()) return DEMO_ROOT;
    const res = await tauriOpen({ directory: true, multiple: false });
    return (res as string) ?? null;
  },

  async pickFonts(): Promise<string[]> {
    if (!isTauri()) return [];
    const res = await tauriOpen({
      directory: false,
      multiple: true,
      filters: [{ name: "Fonts", extensions: ["ttf", "otf", "ttc", "woff2"] }],
    });
    if (!res) return [];
    return Array.isArray(res) ? (res as string[]) : [res as string];
  },

  async pickArchive(): Promise<string | null> {
    if (!isTauri()) return null;
    const res = await tauriOpen({
      directory: false,
      multiple: false,
      filters: [{ name: "Toolchain archive", extensions: ["xz", "tar.xz"] }],
    });
    return (res as string) ?? null;
  },

  async workspaceOpen(path: string): Promise<ProjectInfo> {
    if (!isTauri()) return demoProject();
    return tauriInvoke<ProjectInfo>("workspace_open", { path });
  },

  async workspaceCreate(p: CreateParams): Promise<ProjectInfo> {
    if (!isTauri()) return { ...demoProject(), name: p.name, kind: p.templateId, root: p.path };
    return tauriInvoke<ProjectInfo>("workspace_create", {
      params: {
        path: p.path,
        template_id: p.templateId,
        name: p.name,
        typst: p.typst,
        packages_mode: p.packagesMode,
        git_init: p.gitInit,
        refs: p.refs,
        vault: p.vault,
        template_spec: p.templateSpec ?? null,
        online: p.online ?? true,
      },
    });
  },

  async packagesFetch(root: string, online: boolean): Promise<string[]> {
    if (!isTauri()) return [];
    return tauriInvoke<string[]>("packages_fetch", { root, online });
  },

  async packagesMakeOfflineReady(root: string, online: boolean, vendor: boolean): Promise<OfflineReport> {
    if (!isTauri()) return { packages: [], errors: [] };
    return tauriInvoke<OfflineReport>("packages_make_offline_ready", { root, online, vendor });
  },

  async packagesMirrorCreate(root: string, out: string, online: boolean): Promise<string[]> {
    if (!isTauri()) return [];
    return tauriInvoke<string[]>("packages_mirror_create", { root, out, online });
  },

  async packagesSearch(query: string, online: boolean): Promise<PackageMeta[]> {
    if (!isTauri()) return demoPackages(query);
    return tauriInvoke<PackageMeta[]>("packages_search", { query, online });
  },

  async packagesRefreshIndex(online: boolean): Promise<number> {
    if (!isTauri()) return 0;
    return tauriInvoke<number>("packages_refresh_index", { online });
  },

  async fontsList(root?: string): Promise<FontFamily[]> {
    if (!isTauri()) return demoFonts();
    return tauriInvoke<FontFamily[]>("fonts_list", { root: root ?? null });
  },
  async fontsSetDirs(dirs: string[]): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("fonts_set_dirs", { dirs });
  },
  async fontsRescan(): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("fonts_rescan", {});
  },

  async readTree(path: string): Promise<FileNode> {
    if (!isTauri()) return demoTree();
    return tauriInvoke<FileNode>("read_tree", { path });
  },

  async scanProject(path: string): Promise<ScannedFile[]> {
    if (!isTauri()) return demoScan();
    return tauriInvoke<ScannedFile[]>("scan_project", { path });
  },

  async compile(
    root: string,
    entrypoint: string,
    overlays: { path: string; content: string }[]
  ): Promise<CompileResult> {
    if (!isTauri()) return { pages: [], diagnostics: [], page_count: 0, compile_ms: 0 };
    return tauriInvoke<CompileResult>("compile", { root, entrypoint, overlays });
  },

  async exportPdf(root: string, entrypoint: string, output: string, standards: string[]): Promise<string> {
    if (!isTauri()) throw new Error("PDF export needs the desktop app");
    return tauriInvoke<string>("export_pdf", { root, entrypoint, output, standards });
  },

  async complete(
    root: string,
    entrypoint: string,
    path: string,
    overlays: Overlay[],
    cursor: number,
    explicit: boolean
  ): Promise<CompleteResponse> {
    if (!isTauri()) return { from: cursor, items: [] };
    return tauriInvoke<CompleteResponse>("complete", { root, entrypoint, path, overlays, cursor, explicit });
  },

  async hover(
    root: string,
    entrypoint: string,
    path: string,
    overlays: Overlay[],
    cursor: number
  ): Promise<Hover | null> {
    if (!isTauri()) return null;
    return tauriInvoke<Hover | null>("hover", { root, entrypoint, path, overlays, cursor });
  },

  async spellcheck(text: string): Promise<Misspelling[]> {
    if (!isTauri()) return [];
    return tauriInvoke<Misspelling[]>("spellcheck", { text });
  },

  async jumpFromClick(
    root: string,
    entrypoint: string,
    overlays: Overlay[],
    page: number,
    x: number,
    y: number
  ): Promise<JumpTarget | null> {
    if (!isTauri()) return null;
    return tauriInvoke<JumpTarget | null>("jump_from_click", { root, entrypoint, overlays, page, x, y });
  },

  async jumpFromCursor(
    root: string,
    entrypoint: string,
    path: string,
    overlays: Overlay[],
    cursor: number
  ): Promise<PreviewPosition | null> {
    if (!isTauri()) return null;
    return tauriInvoke<PreviewPosition | null>("jump_from_cursor", {
      root,
      entrypoint,
      path,
      overlays,
      cursor,
    });
  },

  /// Render a page to PNG bytes at `pixelPerPt` (ArrayBuffer in Tauri).
  async renderPng(
    root: string,
    entrypoint: string,
    overlays: Overlay[],
    page: number,
    pixelPerPt: number
  ): Promise<ArrayBuffer | null> {
    if (!isTauri()) return null;
    return tauriInvoke<ArrayBuffer>("render_png", { root, entrypoint, overlays, page, pixelPerPt });
  },

  async vcsSnapshot(root: string, message: string): Promise<string> {
    if (!isTauri()) return "demo";
    return tauriInvoke<string>("vcs_snapshot", { root, message });
  },
  async vcsTimeline(root: string): Promise<{ id: string; message: string; when: string; auto: boolean }[]> {
    if (!isTauri()) return [];
    return tauriInvoke("vcs_timeline", { root });
  },
  async vcsRestore(root: string, commit: string): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("vcs_restore", { root, commit });
  },
  async vcsGetRemote(root: string): Promise<string | null> {
    if (!isTauri()) return null;
    return tauriInvoke<string | null>("vcs_get_remote", { root });
  },
  async vcsSetRemote(root: string, url: string): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("vcs_set_remote", { root, url });
  },
  async vcsSync(root: string): Promise<string> {
    if (!isTauri()) return "demo";
    return tauriInvoke<string>("vcs_sync", { root });
  },

  async refsZoteroStatus(): Promise<boolean> {
    if (!isTauri()) return false;
    return tauriInvoke<boolean>("refs_zotero_status", {});
  },
  async refsZoteroImport(root: string): Promise<number> {
    if (!isTauri()) return 0;
    return tauriInvoke<number>("refs_zotero_import", { root });
  },

  async jobCancel(job: number): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("job_cancel", { job });
  },

  async policyGet(): Promise<Policy> {
    if (!isTauri()) return { allow_hosts: [] };
    return tauriInvoke<Policy>("policy_get", {});
  },

  async toolchainList(): Promise<Toolchain[]> {
    if (!isTauri()) return [{ version: "0.15.1", source: "bundled", path: "embedded" }];
    return tauriInvoke<Toolchain[]>("toolchain_list", {});
  },
  async toolchainAvailable(online: boolean): Promise<string[]> {
    if (!isTauri()) return ["0.15.1", "0.14.2", "0.13.1", "0.12.0"];
    return tauriInvoke<string[]>("toolchain_available", { online });
  },
  async toolchainInstall(version: string, online: boolean): Promise<Toolchain> {
    if (!isTauri()) throw new Error("toolchain install needs the desktop app");
    return tauriInvoke<Toolchain>("toolchain_install", { version, online });
  },
  async toolchainRemove(version: string): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("toolchain_remove", { version });
  },
  async toolchainInstallFromArchive(path: string): Promise<Toolchain> {
    if (!isTauri()) throw new Error("toolchain install needs the desktop app");
    return tauriInvoke<Toolchain>("toolchain_install_from_archive", { path });
  },

  async fileRead(path: string): Promise<string> {
    if (!isTauri()) return demoFiles[relOf(path)] ?? "";
    return tauriInvoke<string>("file_read", { path });
  },

  async fileWrite(path: string, content: string): Promise<void> {
    if (!isTauri()) {
      demoFiles[relOf(path)] = content;
      return;
    }
    return tauriInvoke<void>("file_write", { path, content });
  },

  // ── Crash recovery (autosaved drafts outside the project) ──────────────────
  async recoverySave(path: string, content: string): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("recovery_save", { path, content });
  },
  async recoveryScan(): Promise<{ path: string; savedAt: number }[]> {
    if (!isTauri()) return [];
    return tauriInvoke<{ path: string; savedAt: number }[]>("recovery_scan");
  },
  async recoveryRead(path: string): Promise<string> {
    if (!isTauri()) return "";
    return tauriInvoke<string>("recovery_read", { path });
  },
  async recoveryDiscard(path: string): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("recovery_discard", { path });
  },

  // ── Project file management ────────────────────────────────────────────────
  async fsCreateFile(path: string): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("fs_create_file", { path });
  },
  async fsCreateDir(path: string): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("fs_create_dir", { path });
  },
  async fsRename(from: string, to: string): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("fs_rename", { from, to });
  },
  async fsDelete(path: string): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("fs_delete", { path });
  },
  async fsWriteBytes(path: string, data: Uint8Array): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("fs_write_bytes", { path, data: Array.from(data) });
  },
  async fsImport(destDir: string, sources: string[]): Promise<string[]> {
    if (!isTauri()) return [];
    return tauriInvoke<string[]>("fs_import", { destDir, sources });
  },
  async fsReveal(path: string): Promise<void> {
    if (!isTauri()) return;
    return tauriInvoke<void>("fs_reveal", { path });
  },

  /// Run a command line in `root`, streaming output via terminal:// events.
  async terminalExec(root: string, line: string): Promise<void> {
    if (!isTauri()) throw new Error("Terminal needs the desktop app");
    return tauriInvoke<void>("terminal_exec", { root, line });
  },
  async onTerminalOutput(cb: (stream: string, line: string) => void): Promise<() => void> {
    if (!isTauri()) return () => {};
    return listen<{ stream: string; line: string }>("terminal://output", (e) =>
      cb(e.payload.stream, e.payload.line)
    );
  },
  async onTerminalExit(cb: (code: number) => void): Promise<() => void> {
    if (!isTauri()) return () => {};
    return listen<{ code: number }>("terminal://exit", (e) => cb(e.payload.code));
  },
};
