// Shared reactive app state (Svelte 5 runes). The backend is the source of
// truth (ARCHITECTURE.md §18.4): projects, trees and file contents come from
// real filesystem commands via `api`, and analysis is recomputed live on edit.
import {
  api,
  isTauri,
  setupJobEvents,
  type CreateParams,
  type CompileResult,
  type FontFamily,
  type OfflineReport,
  type Toolchain,
  type Policy,
} from "./api";
import {
  computeStructure,
  computeDiagnostics,
  collectPackages,
  parseBib,
  markCited,
  type ScannedFile,
} from "./analysis";
import type {
  Citation,
  Diagnostic,
  EditorFile,
  ExportProfile,
  FileNode,
  Job,
  Note,
  PackageInfo,
  ProjectInfo,
  Snapshot,
  StructureNode,
} from "./types";

export type ToolId = "project" | "structure" | "notes" | "git" | "packages" | "fonts" | "problems";
export type BottomTab = "problems" | "jobs" | "terminal";

function initialTheme(): "dark" | "light" {
  try {
    const saved = localStorage.getItem("typide.theme");
    if (saved === "dark" || saved === "light") return saved;
  } catch {}
  return "dark";
}
function firstRunDone(): boolean {
  try {
    return localStorage.getItem("typide.setupDone") === "1";
  } catch {}
  return false;
}

function langFor(path: string): EditorFile["language"] {
  if (path.endsWith(".bib")) return "bibtex";
  if (path.endsWith(".toml")) return "toml";
  if (path.endsWith(".md")) return "markdown";
  if (path.endsWith(".yml") || path.endsWith(".yaml")) return "yaml";
  return "typst";
}
const baseName = (p: string) => p.split("/").pop() ?? p;

// ── Recent projects (persisted) ──────────────────────────────────────────────
export interface Recent {
  name: string;
  path: string;
  kind: string;
  openedAt: number;
}
function loadRecents(): Recent[] {
  try {
    return JSON.parse(localStorage.getItem("typide.recents") ?? "[]");
  } catch {
    return [];
  }
}
export const recents = $state<Recent[]>(loadRecents());
function addRecent(p: ProjectInfo) {
  const i = recents.findIndex((r) => r.path === p.root);
  if (i !== -1) recents.splice(i, 1);
  recents.unshift({ name: p.name, path: p.root, kind: p.kind, openedAt: Date.now() });
  if (recents.length > 12) recents.length = 12;
  try {
    const plain = recents.map((r) => ({ name: r.name, path: r.path, kind: r.kind, openedAt: r.openedAt }));
    localStorage.setItem("typide.recents", JSON.stringify(plain));
  } catch {}
}
export function relTime(ts: number): string {
  const s = Math.floor((Date.now() - ts) / 1000);
  if (s < 60) return "just now";
  if (s < 3600) return `${Math.floor(s / 60)} min ago`;
  if (s < 86400) return `${Math.floor(s / 3600)} h ago`;
  if (s < 604800) return `${Math.floor(s / 86400)} d ago`;
  return new Date(ts).toLocaleDateString();
}

// ── UI state ────────────────────────────────────────────────────────────────
export const ui = $state({
  theme: initialTheme(),
  view: "launcher" as "launcher" | "workspace",
  showFirstRun: !firstRunDone(),
  wizardOpen: false,
  networkMode: "ask" as "offline" | "ask" | "online",
  activeTool: "project" as ToolId,
  leftVisible: true,
  previewVisible: true,
  bottomVisible: true,
  bottomTab: "problems" as BottomTab,
  openTabs: [] as string[],
  activeTab: "",
  exportProfile: "PDF",
  commandPalette: false,
  searchEverywhere: false,
  packageBrowser: false,
  toolchainManager: false,
  citationPicker: false,
  consent: null as { host: string; reason: string } | null,
  cursor: { line: 1, col: 1 },
  compiling: false,
  loading: false,
  closeGuard: false,
  recovery: null as { path: string; savedAt: number }[] | null,
  toast: null as { message: string; kind: "ok" | "error" } | null,
  leftWidth: loadSize("left", 272),
  previewWidth: loadSize("preview", 420),
  bottomHeight: loadSize("bottom", 220),
});

function loadSize(key: string, fallback: number): number {
  try {
    const v = Number(localStorage.getItem(`typide.size.${key}`));
    return Number.isFinite(v) && v > 0 ? v : fallback;
  } catch {
    return fallback;
  }
}
export function persistSize(key: "left" | "preview" | "bottom", value: number) {
  try {
    localStorage.setItem(`typide.size.${key}`, String(Math.round(value)));
  } catch {}
}

// ── Project data (dynamic) ───────────────────────────────────────────────────
export const data = $state({
  project: null as ProjectInfo | null,
  tree: null as FileNode | null,
  structure: [] as StructureNode[],
  diagnostics: [] as Diagnostic[],
  packages: [] as PackageInfo[],
  citations: [] as Citation[],
  notes: [] as Note[],
  snapshots: [] as Snapshot[],
  jobs: [] as Job[],
  exportProfiles: [
    { name: "PDF", format: "pdf", output: "out/main.pdf" },
    { name: "Submission PDF/A-2b", format: "pdf", output: "out/submission.pdf", standards: ["a-2b"] },
    { name: "Submission PDF/A + UA", format: "pdf", output: "out/submission-ua.pdf", standards: ["a-2b", "ua-1"] },
  ] as ExportProfile[],
  previewPages: [] as { n: number; changed: boolean }[],
  compiled: null as CompileResult | null,
  fonts: [] as FontFamily[],
  offlineReport: null as OfflineReport | null,
  toolchains: [] as Toolchain[],
  toolchainsAvailable: [] as string[],
  policy: null as Policy | null,
  remote: null as string | null,
});

/** Load the institutional policy and enforce any locked settings (§8.5). */
export async function initPolicy() {
  try {
    const p = await api.policyGet();
    data.policy = p;
    if (p.network_mode === "offline" || p.network_mode === "ask" || p.network_mode === "online") {
      ui.networkMode = p.network_mode;
    }
  } catch (e) {
    console.error("policy_get failed", e);
  }
}

/** Whether the network mode is locked by an institutional policy. */
export const networkLocked = () => !!data.policy?.network_mode;

/** Load installed + available Typst toolchains for the manager UI. */
export async function loadToolchains() {
  try {
    data.toolchains = await api.toolchainList();
  } catch (e) {
    console.error("toolchain_list failed", e);
  }
  // Fetch the downloadable list only when network is already permitted; in
  // "Ask" mode the user triggers it via checkToolchainsOnline().
  if (ui.networkMode === "online" || alwaysAllowThisSession) {
    try {
      data.toolchainsAvailable = await api.toolchainAvailable(true);
    } catch (e) {
      console.error("toolchain_available failed", e);
    }
  }
}

/** Explicitly check GitHub for downloadable Typst versions (asks if needed). */
export async function checkToolchainsOnline() {
  if (!(await requestConsent("api.github.com", "list downloadable Typst versions"))) return;
  try {
    data.toolchainsAvailable = await api.toolchainAvailable(true);
  } catch (e) {
    console.error("toolchain_available failed", e);
  }
}

export async function installToolchain(version: string) {
  if (!(await requestConsent("github.com", `download Typst ${version}`))) return;
  try {
    await api.toolchainInstall(version, true);
    showToast("Installed Typst " + version, "ok");
    await loadToolchains();
  } catch (e) {
    showToast("Install failed: " + String((e as any)?.message ?? e), "error");
  }
}

/** Install a toolchain from a local .tar.xz archive (offline, air-gapped). */
export async function installToolchainFromArchive() {
  const path = await api.pickArchive();
  if (!path) return;
  try {
    const t = await api.toolchainInstallFromArchive(path);
    showToast("Installed Typst " + t.version + " from archive", "ok");
    await loadToolchains();
  } catch (e) {
    showToast("Install failed: " + String((e as any)?.message ?? e), "error");
  }
}

export async function removeToolchain(version: string) {
  try {
    await api.toolchainRemove(version);
    await loadToolchains();
  } catch (e) {
    console.error("toolchain remove failed", e);
  }
}

/** True when the network policy permits downloads at all (not offline). */
export const online = () => ui.networkMode !== "offline";

// ── Network consent (P1 — "Ask each time" is the default mode) ───────────────
let consentResolve: ((v: boolean) => void) | null = null;
let alwaysAllowThisSession = false;

/**
 * Gate a network operation by the current policy. Offline → denied; Online →
 * allowed; Ask → prompt once per session (the user can grant for the session).
 */
export function requestConsent(host: string, reason: string): Promise<boolean> {
  if (ui.networkMode === "offline") return Promise.resolve(false);
  if (ui.networkMode === "online" || alwaysAllowThisSession) return Promise.resolve(true);
  ui.consent = { host, reason };
  return new Promise((res) => {
    consentResolve = res;
  });
}

/** Resolve the pending consent prompt with the user's choice. */
export function resolveConsent(choice: "once" | "always" | "deny") {
  ui.consent = null;
  if (choice === "always") alwaysAllowThisSession = true;
  const res = consentResolve;
  consentResolve = null;
  res?.(choice !== "deny");
}

// Working set. `cache` is reactive so editor content drives live UI (preview,
// dirty markers); `scanned` feeds the analysis recompute.
let scanned: ScannedFile[] = [];
const cache = $state<Record<string, { content: string; original: string }>>({});

// ── Theme ─────────────────────────────────────────────────────────────────
export function applyTheme() {
  document.documentElement.setAttribute("data-theme", ui.theme);
  try {
    localStorage.setItem("typide.theme", ui.theme);
  } catch {}
}
export function toggleTheme() {
  ui.theme = ui.theme === "dark" ? "light" : "dark";
  applyTheme();
}
export function completeFirstRun() {
  ui.showFirstRun = false;
  try {
    localStorage.setItem("typide.setupDone", "1");
  } catch {}
}

// ── Analysis ─────────────────────────────────────────────────────────────────
function deriveNotes(files: ScannedFile[]): Note[] {
  return files
    .filter((f) => f.rel.startsWith("notes/") || (f.kind === "md" && f.rel.includes("note")))
    .map((f) => {
      const fm = f.content.match(/cite-key:\s*([\w:.-]+)/);
      const title =
        f.content.match(/^#*\s*=?\s*(.+)$/m)?.[1]?.replace(/^#\s*/, "").trim() || baseName(f.rel);
      const outgoing = [...f.content.matchAll(/\[\[([^\]]+)\]\]/g)].map((m) => m[1]);
      return { path: f.path, title, citeKey: fm?.[1], backlinks: [], outgoing };
    });
}

function recompute() {
  // Real Typst compiler diagnostics first, then Typide inspections (§12.1).
  const root = data.project?.root ?? "";
  const compilerDiags: Diagnostic[] = (data.compiled?.diagnostics ?? []).map((d) => ({
    id: "typst",
    file: d.file ? `${root}/${d.file}` : "",
    line: d.line,
    col: d.col,
    severity: (d.severity === "error" ? "error" : "warning") as Diagnostic["severity"],
    message: d.message,
    source: "typst",
  }));
  data.diagnostics = [...compilerDiags, ...computeDiagnostics(scanned)];
  data.packages = collectPackages(scanned);
  data.citations = markCited(parseBib(scanned), scanned);
  data.notes = deriveNotes(scanned);
  const active = ui.activeTab ? cache[ui.activeTab]?.content : undefined;
  data.structure = active ? computeStructure(active) : [];
  // Preview pages: a light reflection of the entry document's headings.
  const entry = scanned.find((f) => f.rel === data.project?.entrypoint) ?? scanned[0];
  const headings = entry ? (entry.content.match(/^=+\s+/gm)?.length ?? 0) : 0;
  const pages = Math.max(1, Math.min(24, headings + 1));
  data.previewPages = Array.from({ length: pages }, (_, i) => ({ n: i + 1, changed: false }));
}

// ── Compilation ──────────────────────────────────────────────────────────────
let compileTimer: ReturnType<typeof setTimeout> | undefined;
let autoFetchTried = false;

function currentOverlays() {
  return Object.entries(cache).map(([path, c]) => ({ path: relPath(path), content: c.content }));
}

/** Compile the project's entrypoint and refresh the preview + diagnostics. */
export async function runCompile() {
  if (!data.project) return;
  ui.compiling = true;
  try {
    // Send current editor buffers as overlays so the preview is live without
    // writing to disk (no autosave → no file-watcher churn).
    const root = data.project.root;
    const entry = data.project.entrypoint;
    data.compiled = await api.compile(root, entry, currentOverlays());

    // If a package is missing and we're allowed online, fetch it and retry once.
    const needsPkg = (data.compiled.diagnostics ?? []).some((d) =>
      /package|network is disabled/i.test(d.message)
    );
    if (needsPkg && online() && !autoFetchTried) {
      autoFetchTried = true;
      if (await requestConsent("packages.typst.org", "download missing packages for this document")) {
        try {
          await api.packagesFetch(root, true);
          data.compiled = await api.compile(root, entry, currentOverlays());
        } catch (e) {
          console.error("package fetch failed", e);
        }
      }
    }
    recompute();
  } catch (e) {
    console.error("compile failed", e);
  } finally {
    ui.compiling = false;
  }
}

/** Download every Universe package the project needs, then recompile. */
export async function fetchPackages() {
  if (!data.project) return;
  if (!(await requestConsent("packages.typst.org", "download this project's packages"))) return;
  ui.compiling = true;
  try {
    await api.packagesFetch(data.project.root, true);
    autoFetchTried = false;
    await runCompile();
  } catch (e) {
    console.error("package fetch failed", e);
  } finally {
    ui.compiling = false;
  }
}

/** Resolve + vendor + lock every package so the project compiles years later. */
export async function makeOfflineReady() {
  if (!data.project) return;
  if (!(await requestConsent("packages.typst.org", "download + vendor all packages"))) return;
  ui.compiling = true;
  try {
    data.offlineReport = await api.packagesMakeOfflineReady(data.project.root, true, true);
    data.tree = await api.readTree(data.project.root); // vendor/ now exists
    autoFetchTried = false;
    await runCompile();
  } catch (e) {
    console.error("make offline-ready failed", e);
  } finally {
    ui.compiling = false;
  }
}

/** Build a USB/intranet mirror of this project's packages into a chosen folder. */
export async function mirrorPackages() {
  if (!data.project) return;
  if (!(await requestConsent("packages.typst.org", "download packages to build a mirror"))) return;
  const out = await api.pickFolder();
  if (!out) return;
  ui.compiling = true;
  try {
    const mirrored = await api.packagesMirrorCreate(data.project.root, out, true);
    showToast(`Mirrored ${mirrored.length} package(s) → ${out}`, "ok");
  } catch (e) {
    showToast("Mirror failed: " + String((e as any)?.message ?? e), "error");
  } finally {
    ui.compiling = false;
  }
}

// ── Snapshots (version history) ──────────────────────────────────────────────
export async function loadTimeline() {
  if (!data.project) return;
  try {
    data.snapshots = await api.vcsTimeline(data.project.root);
  } catch (e) {
    console.error("vcs_timeline failed", e);
  }
}

export async function createSnapshot(message: string) {
  if (!data.project) return;
  const msg = message.trim() || "Snapshot " + new Date().toLocaleString();
  try {
    await saveAllDirty(); // commit the current work, not stale disk content
    await api.vcsSnapshot(data.project.root, msg);
    await loadTimeline();
    showToast("Snapshot created", "ok");
  } catch (e) {
    showToast("Snapshot failed: " + String((e as any)?.message ?? e), "error");
  }
}

export async function loadRemote() {
  if (!data.project) return;
  try {
    data.remote = await api.vcsGetRemote(data.project.root);
  } catch {
    data.remote = null;
  }
}

export async function setRemote(url: string) {
  if (!data.project || !url.trim()) return;
  try {
    await api.vcsSetRemote(data.project.root, url.trim());
    await loadRemote();
    showToast("Remote set", "ok");
  } catch (e) {
    showToast("Failed: " + String((e as any)?.message ?? e), "error");
  }
}

export async function syncRemote() {
  if (!data.project || !data.remote) return;
  if (!(await requestConsent(data.remote, "sync with your git remote (fetch · merge · push)"))) return;
  try {
    await saveAllDirty();
    const msg = await api.vcsSync(data.project.root);
    await loadTimeline();
    showToast(msg, "ok");
  } catch (e) {
    showToast("Sync failed: " + String((e as any)?.message ?? e), "error");
  }
}

/** Import the Zotero library into the project and refresh citations (§15.2). */
export async function importZotero() {
  if (!data.project) return;
  if (!(await requestConsent("127.0.0.1:23119 (Zotero)", "read your local Zotero library"))) return;
  try {
    const n = await api.refsZoteroImport(data.project.root);
    scanned = await api.scanProject(data.project.root);
    data.tree = await api.readTree(data.project.root);
    recompute();
    showToast(`Imported ${n} entries from Zotero → zotero.bib`, "ok");
  } catch (e) {
    showToast("Zotero import failed: " + String((e as any)?.message ?? e), "error");
  }
}

export async function restoreSnapshot(commit: string) {
  if (!data.project) return;
  try {
    await api.vcsRestore(data.project.root, commit);
    // Reload open buffers from disk, refresh the live editor, recompile.
    for (const path of Object.keys(cache)) {
      try {
        const content = await api.fileRead(path);
        cache[path] = { content, original: content };
      } catch {}
    }
    reloadActiveEditor();
    data.tree = await api.readTree(data.project.root);
    await loadTimeline();
    await runCompile();
    showToast("Restored to " + commit, "ok");
  } catch (e) {
    showToast("Restore failed: " + String((e as any)?.message ?? e), "error");
  }
}

/** Load the font database for the Fonts panel. */
export async function loadFonts() {
  if (data.fonts.length) return;
  try {
    data.fonts = await api.fontsList();
  } catch (e) {
    console.error("fonts_list failed", e);
  }
}

export async function searchPackages(query: string) {
  // The cached index may already satisfy this offline; only a *fetch* needs
  // consent. Try with the current policy; if the backend reports it needs the
  // network, ask and retry once.
  try {
    return await api.packagesSearch(query, online());
  } catch (e) {
    if (online() && (await requestConsent("packages.typst.org", "download the package index"))) {
      return api.packagesSearch(query, true);
    }
    throw e;
  }
}

// ── Background jobs (live, from backend events) ──────────────────────────────
let jobsInited = false;
export function initJobs() {
  if (jobsInited) return;
  jobsInited = true;
  setupJobEvents({
    started: (e) => {
      data.jobs = [
        { id: e.job, title: e.title, fraction: 0, message: "starting…", state: "running" as const },
        ...data.jobs.filter((j) => j.state === "running"),
      ].slice(0, 12);
    },
    progress: (e) => {
      const j = data.jobs.find((x) => x.id === e.job);
      if (j) {
        j.fraction = e.total ? e.done / e.total : 0;
        j.message = e.message;
      }
    },
    finished: (e) => {
      const j = data.jobs.find((x) => x.id === e.job);
      if (j) {
        j.state = e.ok ? "done" : "failed";
        j.fraction = 1;
        j.message = e.message;
      }
    },
  });
}
export function cancelJob(id: number) {
  api.jobCancel(id);
  const j = data.jobs.find((x) => x.id === id);
  if (j) j.message = "cancelling…";
}

let toastTimer: ReturnType<typeof setTimeout> | undefined;
export function showToast(message: string, kind: "ok" | "error" = "ok") {
  ui.toast = { message, kind };
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (ui.toast = null), 4500);
}

// ── Editor intelligence bridges (typst-ide via the backend) ──────────────────
export function completeAt(cursor: number, explicit: boolean) {
  if (!data.project || !ui.activeTab) return Promise.resolve({ from: cursor, items: [] });
  return api.complete(
    data.project.root,
    data.project.entrypoint,
    relPath(ui.activeTab),
    currentOverlays(),
    cursor,
    explicit
  );
}
export function spellCheck(text: string) {
  // Only meaningful for Typst prose; skip other languages.
  const f = getFile(ui.activeTab);
  if (f && f.language !== "typst") return Promise.resolve([]);
  return api.spellcheck(text);
}

/** Move the caret to a UTF-16 offset in the active editor and reveal it. */
function gotoOffset(offset: number) {
  const view = activeView as any;
  if (!view) return;
  const len = view.state.doc.length;
  const pos = Math.max(0, Math.min(offset, len));
  view.dispatch({ selection: { anchor: pos, head: pos }, scrollIntoView: true });
  view.focus?.();
}

/** Preview click → jump to the source location that produced it. */
export async function jumpToClick(page: number, x: number, y: number) {
  if (!data.project) return;
  const target = await api.jumpFromClick(
    data.project.root,
    data.project.entrypoint,
    currentOverlays(),
    page,
    x,
    y
  );
  if (!target) return;
  const abs = `${data.project.root}/${target.file}`;
  await openFile(abs);
  // Let the editor swap documents before moving the caret.
  setTimeout(() => gotoOffset(target.offset), 40);
}

export function hoverAt(cursor: number) {
  if (!data.project || !ui.activeTab) return Promise.resolve(null);
  return api.hover(
    data.project.root,
    data.project.entrypoint,
    relPath(ui.activeTab),
    currentOverlays(),
    cursor
  );
}

/** Run the selected export profile: save, compile, write the PDF. */
export async function exportActiveProfile() {
  if (!data.project) return;
  const profile =
    data.exportProfiles.find((p) => p.name === ui.exportProfile) ?? data.exportProfiles[0];
  ui.compiling = true;
  try {
    await saveActive();
    const path = await api.exportPdf(
      data.project.root,
      data.project.entrypoint,
      profile.output,
      profile.standards ?? []
    );
    showToast("Exported → " + profile.output, "ok");
  } catch (e) {
    showToast("Export failed: " + String((e as any)?.message ?? e), "error");
  } finally {
    ui.compiling = false;
  }
}

// Editor bridge: lets actions insert text into the active editor.
let activeView: { dispatch: (tr: any) => void; state: any } | null = null;
export function setEditorView(view: any | null) {
  activeView = view;
}

/** Insert a package import into the active document (if not already present). */
export function addPackage(spec: string) {
  const view = activeView;
  const f = getFile(ui.activeTab);
  if (!view || !f) return;
  if (f.content.includes(`"${spec}"`)) return; // already imported
  const line = `#import "${spec}": *\n`;
  view.dispatch({ changes: { from: 0, insert: line } });
}

/** Debounced live recompile from the in-memory buffers (no disk write). */
function scheduleCompile() {
  clearTimeout(compileTimer);
  compileTimer = setTimeout(() => {
    runCompile();
  }, 400);
}

// ── Loading a project ────────────────────────────────────────────────────────
export async function loadProject(path: string) {
  ui.loading = true;
  try {
    const project = await api.workspaceOpen(path);
    data.project = project;
    addRecent(project);
    data.tree = await api.readTree(project.root);
    scanned = await api.scanProject(project.root);
    for (const k of Object.keys(cache)) delete cache[k];
    ui.openTabs = [];
    ui.activeTab = "";
    // Open the entry file.
    const entryPath = `${project.root}/${project.entrypoint}`;
    data.snapshots = [];
    data.remote = null;
    loadTimeline(); // real git history (async, non-blocking)
    loadRemote();
    data.jobs = [];
    data.compiled = null;
    autoFetchTried = false;
    recompute();
    await openFile(entryPath);
    ui.view = "workspace";
    ui.activeTool = "project";
    runCompile(); // initial preview (async, non-blocking)
    scanRecovery(); // offer any unsaved work from a previous session
  } finally {
    ui.loading = false;
  }
}

export async function openFromPicker() {
  const path = await api.pickFolder();
  if (path) await loadProject(path);
}

export async function createProject(p: CreateParams) {
  ui.loading = true;
  try {
    const project = await api.workspaceCreate({ ...p, online: online() });
    ui.wizardOpen = false;
    await loadProject(project.root);
  } finally {
    ui.loading = false;
  }
}

export function closeProject() {
  ui.view = "launcher";
}

// ── Files ────────────────────────────────────────────────────────────────────
export async function openFile(path: string) {
  if (!(path in cache)) {
    const content = await api.fileRead(path);
    cache[path] = { content, original: content };
  }
  if (!ui.openTabs.includes(path)) ui.openTabs.push(path);
  ui.activeTab = path;
  recompute();
}

export function closeTab(path: string) {
  const i = ui.openTabs.indexOf(path);
  if (i === -1) return;
  ui.openTabs.splice(i, 1);
  if (ui.activeTab === path) ui.activeTab = ui.openTabs[Math.max(0, i - 1)] ?? "";
  recompute();
}

export function getFile(path: string): EditorFile | undefined {
  const c = cache[path];
  if (!c) return undefined;
  return {
    path,
    name: baseName(path),
    language: langFor(path),
    content: c.content,
    dirty: c.content !== c.original,
  };
}

export function updateContent(path: string, content: string) {
  const c = cache[path];
  if (!c) return;
  c.content = content;
  // Keep the scanned copy in sync so analysis is live.
  const s = scanned.find((f) => f.path === path);
  if (s) s.content = content;
  recompute();
  scheduleCompile(); // autosave + recompile the preview shortly after typing
}

export async function saveFile(path: string) {
  const c = cache[path];
  if (!c || c.content === c.original) return;
  await api.fileWrite(path, c.content);
  c.original = c.content;
  void api.recoveryDiscard(path); // clean buffer → no draft to recover
}

export async function saveActive() {
  if (ui.activeTab) await saveFile(ui.activeTab);
}

/** Persist every dirty buffer to disk (used before snapshots / git ops). */
export async function saveAllDirty() {
  for (const path of Object.keys(cache)) {
    const c = cache[path];
    if (c && c.content !== c.original) {
      try {
        await api.fileWrite(path, c.content);
        c.original = c.content;
        void api.recoveryDiscard(path);
      } catch (e) {
        console.error("save failed", path, e);
      }
    }
  }
}

/** Replace the live editor document with the active file's (reloaded) content. */
export function reloadActiveEditor() {
  const view = activeView as any;
  const f = getFile(ui.activeTab);
  if (!view || !f) return;
  view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: f.content } });
}

/** Insert text at the current editor caret and keep focus. */
export function insertAtCursor(text: string) {
  const view = activeView as any;
  if (!view) return;
  const pos = view.state.selection.main.head;
  view.dispatch({ changes: { from: pos, insert: text }, selection: { anchor: pos + text.length } });
  view.focus?.();
}

/** Explicit save (Ctrl+S / Run): persist and recompile immediately. */
export async function saveAndCompile() {
  await saveActive();
  await runCompile();
}

export function isDirty(path: string): boolean {
  const c = cache[path];
  return !!c && c.content !== c.original;
}

/** How many open buffers have unsaved changes. */
export function dirtyCount(): number {
  return Object.keys(cache).filter((p) => isDirty(p)).length;
}

/** Relative path from the project root, for breadcrumbs. */
export function relPath(path: string): string {
  const root = data.project?.root;
  return root && path.startsWith(root + "/") ? path.slice(root.length + 1) : baseName(path);
}

// ── Unsaved-work safety: autosave recovery + close guard ─────────────────────

let recoveryTimer: ReturnType<typeof setInterval> | null = null;

/** Periodically mirror dirty buffers to the crash-recovery store (not the
 *  project). Cheap: a few small files every few seconds. */
export function initRecoveryAutosave() {
  if (!isTauri() || recoveryTimer) return;
  recoveryTimer = setInterval(() => {
    for (const path of Object.keys(cache)) {
      if (isDirty(path)) void api.recoverySave(path, cache[path].content);
    }
  }, 4000);
}

/** After a project loads, surface any drafts newer than disk for this project. */
export async function scanRecovery() {
  if (!isTauri() || !data.project) return;
  try {
    const all = await api.recoveryScan();
    const root = data.project.root;
    const mine = all.filter((d) => d.path === root || d.path.startsWith(root + "/"));
    ui.recovery = mine.length ? mine : null;
  } catch {
    ui.recovery = null;
  }
}

/** Load each recovered draft into its buffer (and the live editor if open). */
export async function restoreRecovery() {
  const drafts = ui.recovery ?? [];
  for (const d of drafts) {
    try {
      const content = await api.recoveryRead(d.path);
      if (!(d.path in cache)) {
        const original = await api.fileRead(d.path).catch(() => content);
        cache[d.path] = { content, original };
      } else {
        cache[d.path].content = content;
      }
      if (!ui.openTabs.includes(d.path)) ui.openTabs.push(d.path);
    } catch (e) {
      console.error("restore failed", d.path, e);
    }
  }
  if (drafts[0]) ui.activeTab = drafts[0].path;
  ui.recovery = null;
  reloadActiveEditor();
  recompute();
  scheduleCompile();
  showToast(`Recovered ${drafts.length} unsaved file${drafts.length === 1 ? "" : "s"}`);
}

/** Drop all recovered drafts for this project without restoring them. */
export async function discardRecovery() {
  for (const d of ui.recovery ?? []) void api.recoveryDiscard(d.path);
  ui.recovery = null;
}

/** Intercept the window close: if there are unsaved buffers, ask first. */
export async function initCloseGuard() {
  if (!isTauri()) return;
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const win = getCurrentWindow();
    await win.onCloseRequested((event) => {
      if (dirtyCount() > 0) {
        event.preventDefault();
        ui.closeGuard = true;
      }
    });
  } catch (e) {
    console.error("close guard init failed", e);
  }
}

async function forceClose() {
  const { getCurrentWindow } = await import("@tauri-apps/api/window");
  await getCurrentWindow().destroy();
}

/** Close-guard action: save everything, then quit. */
export async function saveAndQuit() {
  await saveAllDirty();
  ui.closeGuard = false;
  await forceClose();
}

/** Close-guard action: discard unsaved work (and its drafts), then quit. */
export async function discardAndQuit() {
  for (const path of Object.keys(cache)) {
    if (isDirty(path)) void api.recoveryDiscard(path);
  }
  ui.closeGuard = false;
  await forceClose();
}

/** Close-guard action: keep editing. */
export function cancelQuit() {
  ui.closeGuard = false;
}
