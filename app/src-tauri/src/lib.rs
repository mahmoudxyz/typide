//! Typide desktop shell (Tauri v2) — library entry.
//!
//! Command layer over the project on disk (ARCHITECTURE.md §18.2). Commands are
//! named `domain_action` and return `Result<T, IpcError>`. These operate on real
//! files; deeper intelligence (LSP, index) is layered in later milestones.
#![deny(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

// ── Background jobs: progress + cancellation (ARCHITECTURE.md §10.7, P7) ──────

/// Tracks running jobs so they can be cancelled by id.
#[derive(Default)]
struct JobRegistry {
    counter: AtomicU64,
    cancels: Mutex<HashMap<u64, Arc<AtomicBool>>>,
}

impl JobRegistry {
    fn start(&self) -> (u64, Arc<AtomicBool>) {
        let id = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        let flag = Arc::new(AtomicBool::new(false));
        self.cancels.lock().unwrap().insert(id, flag.clone());
        (id, flag)
    }
    fn cancel(&self, id: u64) {
        if let Some(f) = self.cancels.lock().unwrap().get(&id) {
            f.store(true, Ordering::Relaxed);
        }
    }
    fn finish(&self, id: u64) {
        self.cancels.lock().unwrap().remove(&id);
    }
}

#[derive(Clone, Serialize)]
struct JobStarted {
    job: u64,
    title: String,
}
#[derive(Clone, Serialize)]
struct JobProgress {
    job: u64,
    done: usize,
    total: usize,
    message: String,
}
#[derive(Clone, Serialize)]
struct JobFinished {
    job: u64,
    ok: bool,
    message: String,
}

/// A progress sink that emits Tauri events and reads a cancellation flag.
struct EventProgress {
    app: AppHandle,
    job: u64,
    cancel: Arc<AtomicBool>,
}
impl typide_packages::ProgressSink for EventProgress {
    fn step(&self, done: usize, total: usize, message: &str) {
        let _ = self.app.emit(
            "job://progress",
            JobProgress {
                job: self.job,
                done,
                total,
                message: message.to_string(),
            },
        );
    }
    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
}

/// Cancel a running job by id.
#[tauri::command]
fn job_cancel(jobs: State<'_, Arc<JobRegistry>>, job: u64) -> IpcResult<()> {
    jobs.cancel(job);
    Ok(())
}

// ── Toolchains (ARCHITECTURE.md §9) ──────────────────────────────────────────

/// List installed/detected Typst toolchains.
#[tauri::command]
fn toolchain_list() -> IpcResult<Vec<typide_toolchain::Toolchain>> {
    Ok(typide_toolchain::list_installed())
}

/// The effective institutional policy (admin-controlled, §8.5).
#[tauri::command]
fn policy_get() -> IpcResult<typide_policy::Policy> {
    Ok(typide_policy::load())
}

/// Whether a local Zotero (+ Better BibTeX) is reachable (§15.2).
#[tauri::command]
fn refs_zotero_status() -> IpcResult<bool> {
    Ok(typide_refs::zotero_status())
}

/// Import the Zotero library into `<root>/zotero.bib`; returns the entry count.
#[tauri::command]
fn refs_zotero_import(root: String) -> IpcResult<usize> {
    let bib = typide_refs::zotero_export_biblatex().map_err(|e| IpcError::new("refs.zotero", e))?;
    let count = typide_refs::count_entries(&bib);
    std::fs::write(PathBuf::from(&root).join("zotero.bib"), &bib)
        .map_err(|e| IpcError::new("refs.write", format!("{e}")))?;
    Ok(count)
}

// ── Snapshots (ARCHITECTURE.md §16) ──────────────────────────────────────────

/// Create a snapshot (commit) of the project; returns the short hash.
#[tauri::command]
fn vcs_snapshot(root: String, message: String) -> IpcResult<String> {
    typide_vcs::snapshot(Path::new(&root), &message).map_err(|e| IpcError::new("vcs.snapshot", e))
}

/// The snapshot timeline (most recent first).
#[tauri::command]
fn vcs_timeline(root: String) -> IpcResult<Vec<typide_vcs::Snapshot>> {
    typide_vcs::timeline(Path::new(&root)).map_err(|e| IpcError::new("vcs.timeline", e))
}

/// Restore the working tree to a given snapshot (files only).
#[tauri::command]
fn vcs_restore(root: String, commit: String) -> IpcResult<()> {
    typide_vcs::restore(Path::new(&root), &commit).map_err(|e| IpcError::new("vcs.restore", e))
}

/// The configured `origin` remote URL, if any.
#[tauri::command]
fn vcs_get_remote(root: String) -> IpcResult<Option<String>> {
    Ok(typide_vcs::get_remote(Path::new(&root)))
}

/// Set the `origin` remote URL.
#[tauri::command]
fn vcs_set_remote(root: String, url: String) -> IpcResult<()> {
    typide_vcs::set_remote(Path::new(&root), &url).map_err(|e| IpcError::new("vcs.remote", e))
}

/// Sync with the remote (fetch → merge → push) — a cancellable job.
#[tauri::command]
async fn vcs_sync(
    app: AppHandle,
    jobs: State<'_, Arc<JobRegistry>>,
    root: String,
) -> IpcResult<String> {
    let root_path = PathBuf::from(&root);
    let reg = jobs.inner().clone();
    let (job, _cancel) = reg.start();
    let _ = app.emit(
        "job://started",
        JobStarted {
            job,
            title: "Sync with remote".into(),
        },
    );
    let _ = app.emit(
        "job://progress",
        JobProgress {
            job,
            done: 0,
            total: 1,
            message: "fetch · merge · push".into(),
        },
    );

    let res = tauri::async_runtime::spawn_blocking(move || typide_vcs::sync(&root_path))
        .await
        .map_err(|e| IpcError::new("job.join", e.to_string()))?;

    reg.finish(job);
    let ok = res.is_ok();
    let message = match &res {
        Ok(m) => m.clone(),
        Err(e) => e.clone(),
    };
    let _ = app.emit("job://finished", JobFinished { job, ok, message });
    res.map_err(|e| IpcError::new("vcs.sync", e))
}

/// List Typst versions available to install from GitHub releases.
#[tauri::command]
fn toolchain_available(online: bool) -> IpcResult<Vec<String>> {
    typide_toolchain::list_available(online).map_err(|e| IpcError::new("toolchain.available", e))
}

/// Remove an installed toolchain.
#[tauri::command]
fn toolchain_remove(version: String) -> IpcResult<()> {
    typide_toolchain::remove(&version).map_err(|e| IpcError::new("toolchain.remove", e))
}

/// Install a Typst toolchain from a local `.tar.xz` archive (offline, §9.4).
#[tauri::command]
fn toolchain_install_from_archive(path: String) -> IpcResult<typide_toolchain::Toolchain> {
    typide_toolchain::install_from_archive(Path::new(&path))
        .map_err(|e| IpcError::new("toolchain.archive", e))
}

/// Download and install a Typst version — a cancellable job with progress.
#[tauri::command]
async fn toolchain_install(
    app: AppHandle,
    jobs: State<'_, Arc<JobRegistry>>,
    version: String,
    online: bool,
) -> IpcResult<typide_toolchain::Toolchain> {
    let reg = jobs.inner().clone();
    let (job, cancel) = reg.start();
    let _ = app.emit(
        "job://started",
        JobStarted {
            job,
            title: format!("Install Typst {version}"),
        },
    );
    let _ = app.emit(
        "job://progress",
        JobProgress {
            job,
            done: 0,
            total: 1,
            message: format!("downloading typst {version}"),
        },
    );

    let app2 = app.clone();
    let ver = version.clone();
    let res = tauri::async_runtime::spawn_blocking(move || {
        let _ = cancel; // install is a single step; cancellation not mid-download yet
        let _ = app2;
        typide_toolchain::install(&ver, online)
    })
    .await
    .map_err(|e| IpcError::new("job.join", e.to_string()))?;

    reg.finish(job);
    let ok = res.is_ok();
    let message = match &res {
        Ok(t) => format!("installed Typst {}", t.version),
        Err(e) => e.clone(),
    };
    let _ = app.emit("job://finished", JobFinished { job, ok, message });
    res.map_err(|e| IpcError::new("toolchain.install", e))
}

/// Uniform error shape returned to the frontend (ARCHITECTURE.md §18.2).
#[derive(Debug, Serialize)]
pub struct IpcError {
    /// Stable machine-readable code, e.g. `project.not_found`.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Optional structured details.
    pub details: Option<serde_json::Value>,
}

impl IpcError {
    fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }
}

type IpcResult<T> = Result<T, IpcError>;

/// Project summary sent to the frontend on open (ARCHITECTURE.md §6).
#[derive(Debug, Serialize)]
pub struct ProjectInfo {
    name: String,
    kind: String,
    root: String,
    entrypoint: String,
    typst: String,
    tinymist: String,
    branch: String,
}

/// A node in the project file tree.
#[derive(Debug, Serialize)]
pub struct FileNode {
    name: String,
    path: String,
    kind: String,
    children: Option<Vec<FileNode>>,
}

/// One scanned source file (content included) for frontend analysis.
#[derive(Debug, Serialize)]
pub struct ScannedFile {
    path: String,
    rel: String,
    kind: String,
    content: String,
}

/// Parameters for creating a project from the New Project wizard (§6.2, §18.2).
#[derive(Debug, Deserialize)]
pub struct CreateParams {
    /// Absolute destination folder (created if missing).
    pub path: String,
    /// Generator id: thesis | paper | notes | package | generic | template.
    pub template_id: String,
    /// Project name.
    pub name: String,
    /// Typst toolchain version string.
    pub typst: String,
    /// "vendor" | "cache".
    pub packages_mode: String,
    /// Initialize a git repo.
    pub git_init: bool,
    /// Add a bibliography file.
    pub refs: bool,
    /// Create a notes vault.
    pub vault: bool,
    /// For the "template" generator: the package spec to instantiate.
    #[serde(default)]
    pub template_spec: Option<String>,
    /// Whether network downloads are allowed (from the current network mode).
    #[serde(default = "yes")]
    pub online: bool,
}

fn yes() -> bool {
    true
}

const IGNORED_DIRS: &[&str] = &[".git", "target", "node_modules", ".vite", "out"];
const MAX_SCAN_BYTES: u64 = 512 * 1024;

fn ext_kind(p: &Path) -> String {
    match p.extension().and_then(|e| e.to_str()) {
        Some("typ") => "typ",
        Some("bib") => "bib",
        Some("md") => "md",
        Some("toml") => "toml",
        Some("yml") | Some("yaml") => "yml",
        Some("otf") | Some("ttf") | Some("ttc") => "font",
        Some("pdf") => "pdf",
        _ => "other",
    }
    .to_string()
}

fn build_tree(dir: &Path, depth: usize) -> Option<FileNode> {
    let name = dir.file_name()?.to_string_lossy().to_string();
    if IGNORED_DIRS.contains(&name.as_str()) {
        return None;
    }
    let mut children: Vec<FileNode> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        let mut entries: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
        entries.sort_by_key(|p| (!p.is_dir(), p.file_name().map(|n| n.to_ascii_lowercase())));
        for p in entries {
            if p.is_dir() && depth < 12 {
                if let Some(child) = build_tree(&p, depth + 1) {
                    children.push(child);
                }
            } else if p.is_file() {
                if let Some(fname) = p.file_name() {
                    children.push(FileNode {
                        name: fname.to_string_lossy().to_string(),
                        path: p.to_string_lossy().to_string(),
                        kind: ext_kind(&p),
                        children: None,
                    });
                }
            }
        }
    }
    Some(FileNode {
        name,
        path: dir.to_string_lossy().to_string(),
        kind: "dir".into(),
        children: Some(children),
    })
}

fn read_field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix(key) {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                return Some(rest.trim().trim_matches('"'));
            }
        }
    }
    None
}

fn git_branch(root: &Path) -> String {
    std::fs::read_to_string(root.join(".git/HEAD"))
        .ok()
        .and_then(|h| h.trim().rsplit('/').next().map(|s| s.to_string()))
        .unwrap_or_else(|| "—".into())
}

fn load_project(root: &Path) -> ProjectInfo {
    let default_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "project".into());
    let cfg = std::fs::read_to_string(root.join("typide.toml")).unwrap_or_default();
    let name = read_field(&cfg, "name")
        .unwrap_or(&default_name)
        .to_string();
    let kind = read_field(&cfg, "kind").unwrap_or("generic").to_string();
    let mut entrypoint = read_field(&cfg, "entrypoint")
        .unwrap_or("main.typ")
        .to_string();
    if !root.join(&entrypoint).exists() {
        // Fall back to the first .typ at the root.
        if let Ok(rd) = std::fs::read_dir(root) {
            if let Some(first) = rd
                .flatten()
                .map(|e| e.path())
                .find(|p| p.extension().and_then(|e| e.to_str()) == Some("typ"))
            {
                entrypoint = first.file_name().unwrap().to_string_lossy().to_string();
            }
        }
    }
    let typst = read_field(&cfg, "typst").unwrap_or("0.15.1").to_string();
    ProjectInfo {
        name,
        kind,
        root: root.to_string_lossy().to_string(),
        entrypoint,
        typst,
        tinymist: "0.13.10".into(),
        branch: git_branch(root),
    }
}

// ── Commands (ARCHITECTURE.md §18.2) ───────────────────────────────────────

/// Resolve the user's home directory from the environment (cross-platform).
fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
}

/// Expand a leading `~`, `~/` or `~\` to the user's home directory.
///
/// Paths that don't start with `~` are returned unchanged. This guards against
/// a literal `~` folder being created on disk when the UI passes an unexpanded
/// home path (e.g. the wizard's default `~/research`). If the home directory
/// can't be resolved, the input is used verbatim.
fn expand_tilde(input: &str) -> PathBuf {
    let trimmed = input.trim();
    if trimmed == "~" {
        if let Some(home) = home_dir() {
            return home;
        }
    } else if let Some(rest) = trimmed
        .strip_prefix("~/")
        .or_else(|| trimmed.strip_prefix("~\\"))
    {
        if let Some(home) = home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(trimmed)
}

/// Open a folder as a project; infers config from `typide.toml` when present.
#[tauri::command]
fn workspace_open(path: String) -> IpcResult<ProjectInfo> {
    let root = expand_tilde(&path);
    if !root.is_dir() {
        return Err(IpcError::new(
            "path.not_dir",
            format!("Not a folder: {path}"),
        ));
    }
    Ok(load_project(&root))
}

/// Return the recursive file tree for a project root.
#[tauri::command]
fn read_tree(path: String) -> IpcResult<FileNode> {
    let root = PathBuf::from(&path);
    build_tree(&root, 0).ok_or_else(|| IpcError::new("tree.failed", "Could not read folder"))
}

/// Scan `.typ` / `.bib` / `.md` sources (content included) for analysis.
#[tauri::command]
fn scan_project(path: String) -> IpcResult<Vec<ScannedFile>> {
    let root = PathBuf::from(&path);
    let mut out = Vec::new();
    fn walk(dir: &Path, root: &Path, out: &mut Vec<ScannedFile>) {
        if out.len() > 2000 {
            return;
        }
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in rd.flatten() {
            let p = entry.path();
            let name = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if p.is_dir() {
                if !IGNORED_DIRS.contains(&name.as_str()) && !name.starts_with('.') {
                    walk(&p, root, out);
                }
            } else if matches!(
                p.extension().and_then(|e| e.to_str()),
                Some("typ") | Some("bib") | Some("md")
            ) {
                let too_big = std::fs::metadata(&p)
                    .map(|m| m.len() > MAX_SCAN_BYTES)
                    .unwrap_or(true);
                if too_big {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&p) {
                    let rel = p
                        .strip_prefix(root)
                        .unwrap_or(&p)
                        .to_string_lossy()
                        .replace('\\', "/");
                    out.push(ScannedFile {
                        path: p.to_string_lossy().to_string(),
                        rel,
                        kind: ext_kind(&p),
                        content,
                    });
                }
            }
        }
    }
    walk(&root, &root, &mut out);
    Ok(out)
}

/// Read a file's UTF-8 content.
/// Compile a project with the embedded Typst compiler, returning per-page SVG
/// and diagnostics (ARCHITECTURE.md §13.1). `overlays` carry unsaved editor
/// buffers so the preview is live without writing to disk.
#[tauri::command]
fn compile(
    root: String,
    entrypoint: String,
    overlays: Vec<typide_world::Overlay>,
) -> IpcResult<typide_world::CompileResult> {
    let root_path = std::path::PathBuf::from(&root);
    if !root_path.is_dir() {
        return Err(IpcError::new(
            "path.not_dir",
            format!("Not a folder: {root}"),
        ));
    }
    Ok(typide_world::compile(&root_path, &entrypoint, &overlays))
}

#[tauri::command]
fn file_read(path: String) -> IpcResult<String> {
    std::fs::read_to_string(&path).map_err(|e| IpcError::new("file.read", format!("{path}: {e}")))
}

/// Write UTF-8 content to a file.
#[tauri::command]
fn file_write(path: String, content: String) -> IpcResult<()> {
    if let Some(parent) = Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, content).map_err(|e| IpcError::new("file.write", format!("{path}: {e}")))
}

// ── Project file management (tree: new / rename / delete / paste) ─────────────

/// Create an empty file (and any missing parent dirs). Errors if it exists.
#[tauri::command]
fn fs_create_file(path: String) -> IpcResult<()> {
    let p = Path::new(&path);
    if p.exists() {
        return Err(IpcError::new(
            "fs.exists",
            format!("Already exists: {path}"),
        ));
    }
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent).map_err(|e| IpcError::new("fs.mkdir", e.to_string()))?;
    }
    std::fs::write(p, "").map_err(|e| IpcError::new("fs.create", format!("{path}: {e}")))
}

/// Create a directory (and any missing parents).
#[tauri::command]
fn fs_create_dir(path: String) -> IpcResult<()> {
    let p = Path::new(&path);
    if p.exists() {
        return Err(IpcError::new(
            "fs.exists",
            format!("Already exists: {path}"),
        ));
    }
    std::fs::create_dir_all(p).map_err(|e| IpcError::new("fs.mkdir", format!("{path}: {e}")))
}

/// Rename/move a file or folder.
#[tauri::command]
fn fs_rename(from: String, to: String) -> IpcResult<()> {
    let dst = Path::new(&to);
    if dst.exists() {
        return Err(IpcError::new("fs.exists", format!("Target exists: {to}")));
    }
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent).map_err(|e| IpcError::new("fs.mkdir", e.to_string()))?;
    }
    std::fs::rename(&from, &to)
        .map_err(|e| IpcError::new("fs.rename", format!("{from} → {to}: {e}")))
}

/// Delete a file or folder (recursive for folders).
#[tauri::command]
fn fs_delete(path: String) -> IpcResult<()> {
    let p = Path::new(&path);
    let res = if p.is_dir() {
        std::fs::remove_dir_all(p)
    } else {
        std::fs::remove_file(p)
    };
    res.map_err(|e| IpcError::new("fs.delete", format!("{path}: {e}")))
}

/// Write raw bytes to a file (for pasted/imported images and binaries).
#[tauri::command]
fn fs_write_bytes(path: String, data: Vec<u8>) -> IpcResult<()> {
    if let Some(parent) = Path::new(&path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&path, data).map_err(|e| IpcError::new("fs.write_bytes", format!("{path}: {e}")))
}

/// Copy OS files into `dest_dir` (for pasting files from the file manager).
/// Returns the base names actually written (deduplicated on collision).
#[tauri::command]
fn fs_import(dest_dir: String, sources: Vec<String>) -> IpcResult<Vec<String>> {
    let dir = Path::new(&dest_dir);
    std::fs::create_dir_all(dir).map_err(|e| IpcError::new("fs.mkdir", e.to_string()))?;
    let mut written = Vec::new();
    for src in &sources {
        // Accept file:// URIs as well as plain paths.
        let clean = src.strip_prefix("file://").unwrap_or(src);
        let clean = percent_decode(clean);
        let sp = Path::new(&clean);
        let Some(name) = sp.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let dest = unique_path(dir, name);
        if sp.is_dir() {
            continue; // skip folders for paste-import
        }
        if std::fs::copy(sp, &dest).is_ok() {
            if let Some(n) = dest.file_name().and_then(|n| n.to_str()) {
                written.push(n.to_string());
            }
        }
    }
    Ok(written)
}

/// Open a file's containing folder (or the folder itself) in the OS file
/// manager. Args are passed as an array — never through a shell (§20).
#[tauri::command]
fn fs_reveal(path: String) -> IpcResult<()> {
    let p = Path::new(&path);
    let target = if p.is_dir() {
        p.to_path_buf()
    } else {
        p.parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| p.to_path_buf())
    };
    #[cfg(target_os = "linux")]
    let program = "xdg-open";
    #[cfg(target_os = "macos")]
    let program = "open";
    #[cfg(target_os = "windows")]
    let program = "explorer";
    std::process::Command::new(program)
        .arg(target.as_os_str())
        .spawn()
        .map_err(|e| IpcError::new("fs.reveal", format!("{}: {e}", target.display())))?;
    Ok(())
}

/// Minimal `%XX` percent-decoding for `file://` URIs (spaces etc.).
fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Pick a non-colliding path in `dir` for `name` (adds ` (2)`, ` (3)`, …).
fn unique_path(dir: &Path, name: &str) -> PathBuf {
    let candidate = dir.join(name);
    if !candidate.exists() {
        return candidate;
    }
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) => (s.to_string(), format!(".{e}")),
        None => (name.to_string(), String::new()),
    };
    for n in 2..10_000 {
        let c = dir.join(format!("{stem} ({n}){ext}"));
        if !c.exists() {
            return c;
        }
    }
    dir.join(name)
}

/// Create a project on disk from a wizard generator, then return its info.
#[tauri::command]
fn workspace_create(params: CreateParams) -> IpcResult<ProjectInfo> {
    if params.name.trim().is_empty() {
        return Err(IpcError::new("name.empty", "Project name is required"));
    }
    let root = expand_tilde(&params.path);
    std::fs::create_dir_all(&root)
        .map_err(|e| IpcError::new("create.mkdir", format!("{}: {e}", params.path)))?;

    // From a Typst template package: download it and lay down its files.
    if params.template_id == "template" {
        let spec = params
            .template_spec
            .as_deref()
            .ok_or_else(|| IpcError::new("template.missing", "No template selected"))?;
        let entrypoint = typide_packages::init_template(spec, &root, params.online)
            .map_err(|e| IpcError::new("template.init", e))?;
        write_template_config(&root, &params, &entrypoint)
            .map_err(|e| IpcError::new("create.config", e.to_string()))?;
    } else {
        scaffold(&root, &params).map_err(|e| IpcError::new("create.scaffold", e.to_string()))?;
        // Best-effort: pre-fetch any Universe packages so the first compile works.
        let _ = typide_packages::fetch_all(&root, params.online);
    }

    if params.git_init {
        let _ = std::process::Command::new("git")
            .arg("init")
            .arg("-q")
            .current_dir(&root)
            .status();
    }
    Ok(load_project(&root))
}

/// Write a `typide.toml` for a project created from a template.
fn write_template_config(root: &Path, p: &CreateParams, entrypoint: &str) -> std::io::Result<()> {
    let toml = format!(
        "[project]\nname = \"{}\"\nkind = \"paper\"\nentrypoint = \"{entrypoint}\"\n\n\
         [toolchain]\ntypst = \"{}\"\ntinymist = \"auto\"\n\n\
         [packages]\nmode = \"cache\"\n\n\
         [fonts]\nuse-system-fonts = true\n\n\
         [[export]]\nname = \"PDF\"\nformat = \"pdf\"\noutput = \"out/main.pdf\"\n",
        p.name, p.typst
    );
    write(root, "typide.toml", &toml)?;
    write(root, ".gitignore", "/out\n/target\n*.pdf\n")?;
    Ok(())
}

/// Download any Universe packages the project needs — a cancellable job with
/// live progress events (`job://started|progress|finished`).
#[tauri::command]
async fn packages_fetch(
    app: AppHandle,
    jobs: State<'_, Arc<JobRegistry>>,
    root: String,
    online: bool,
) -> IpcResult<Vec<String>> {
    let root_path = PathBuf::from(&root);
    if !root_path.is_dir() {
        return Err(IpcError::new(
            "path.not_dir",
            format!("Not a folder: {root}"),
        ));
    }
    let reg = jobs.inner().clone();
    let (job, cancel) = reg.start();
    let _ = app.emit(
        "job://started",
        JobStarted {
            job,
            title: "Fetch packages".into(),
        },
    );

    let app2 = app.clone();
    let res = tauri::async_runtime::spawn_blocking(move || {
        let sink = EventProgress {
            app: app2,
            job,
            cancel,
        };
        typide_packages::fetch_all_with(&root_path, online, &sink)
    })
    .await
    .map_err(|e| IpcError::new("job.join", e.to_string()))?;

    reg.finish(job);
    let ok = res.is_ok();
    let message = match &res {
        Ok(v) => format!("{} package(s)", v.len()),
        Err(e) => e.clone(),
    };
    let _ = app.emit("job://finished", JobFinished { job, ok, message });
    res.map_err(|e| IpcError::new("packages.fetch", e))
}

/// Resolve, vendor, hash and lock every package — a cancellable job.
#[tauri::command]
async fn packages_make_offline_ready(
    app: AppHandle,
    jobs: State<'_, Arc<JobRegistry>>,
    root: String,
    online: bool,
    vendor: bool,
) -> IpcResult<typide_packages::OfflineReport> {
    let root_path = PathBuf::from(&root);
    if !root_path.is_dir() {
        return Err(IpcError::new(
            "path.not_dir",
            format!("Not a folder: {root}"),
        ));
    }
    let reg = jobs.inner().clone();
    let (job, cancel) = reg.start();
    let _ = app.emit(
        "job://started",
        JobStarted {
            job,
            title: "Make offline-ready".into(),
        },
    );

    let app2 = app.clone();
    let res = tauri::async_runtime::spawn_blocking(move || {
        let sink = EventProgress {
            app: app2,
            job,
            cancel,
        };
        typide_packages::make_offline_ready_with(&root_path, online, vendor, &sink)
    })
    .await
    .map_err(|e| IpcError::new("job.join", e.to_string()))?;

    reg.finish(job);
    let ok = res.is_ok();
    let message = match &res {
        Ok(r) => format!("{} package(s) vendored", r.packages.len()),
        Err(e) => e.clone(),
    };
    let _ = app.emit("job://finished", JobFinished { job, ok, message });
    res.map_err(|e| IpcError::new("packages.offline", e))
}

/// Build a Universe-layout mirror of the project's packages into `out` — a
/// cancellable job (ARCHITECTURE.md §10.5).
#[tauri::command]
async fn packages_mirror_create(
    app: AppHandle,
    jobs: State<'_, Arc<JobRegistry>>,
    root: String,
    out: String,
    online: bool,
) -> IpcResult<Vec<String>> {
    let root_path = PathBuf::from(&root);
    let out_path = PathBuf::from(&out);
    if !root_path.is_dir() {
        return Err(IpcError::new(
            "path.not_dir",
            format!("Not a folder: {root}"),
        ));
    }
    let reg = jobs.inner().clone();
    let (job, cancel) = reg.start();
    let _ = app.emit(
        "job://started",
        JobStarted {
            job,
            title: "Create package mirror".into(),
        },
    );

    let app2 = app.clone();
    let res = tauri::async_runtime::spawn_blocking(move || {
        let sink = EventProgress {
            app: app2,
            job,
            cancel,
        };
        typide_packages::create_mirror(&root_path, &out_path, online, &sink)
    })
    .await
    .map_err(|e| IpcError::new("job.join", e.to_string()))?;

    reg.finish(job);
    let ok = res.is_ok();
    let message = match &res {
        Ok(v) => format!("{} package(s) mirrored", v.len()),
        Err(e) => e.clone(),
    };
    let _ = app.emit("job://finished", JobFinished { job, ok, message });
    res.map_err(|e| IpcError::new("packages.mirror", e))
}

/// Search the Typst Universe index (cached locally; fetched when `online`).
#[tauri::command]
fn packages_search(query: String, online: bool) -> IpcResult<Vec<typide_packages::PackageMeta>> {
    typide_packages::search(&query, online, 60).map_err(|e| IpcError::new("packages.search", e))
}

/// Refresh the cached Universe index. Returns the number of packages.
#[tauri::command]
fn packages_refresh_index(online: bool) -> IpcResult<usize> {
    typide_packages::refresh_index(online).map_err(|e| IpcError::new("packages.index", e))
}

/// List every font family available to the compiler (embedded + system).
#[tauri::command]
fn fonts_list() -> IpcResult<Vec<typide_world::FontFamily>> {
    Ok(typide_world::list_fonts())
}

/// Compile and export the project to a PDF at `output` (relative to root).
/// Returns the absolute path written.
#[tauri::command]
fn export_pdf(
    root: String,
    entrypoint: String,
    output: String,
    standards: Vec<String>,
) -> IpcResult<String> {
    let root_path = PathBuf::from(&root);
    let bytes = typide_world::export_pdf(&root_path, &entrypoint, &standards).map_err(|diags| {
        let first = diags.first().map(|d| d.message.clone()).unwrap_or_default();
        IpcError::new(
            "export.failed",
            format!("{} diagnostic(s) prevented export: {first}", diags.len()),
        )
    })?;
    let out_path = root_path.join(&output);
    if let Some(parent) = out_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(&out_path, bytes).map_err(|e| IpcError::new("export.write", format!("{e}")))?;
    Ok(out_path.to_string_lossy().to_string())
}

/// Autocomplete at a UTF-16 cursor offset in `path`.
#[tauri::command]
fn complete(
    root: String,
    entrypoint: String,
    path: String,
    overlays: Vec<typide_world::Overlay>,
    cursor: usize,
    explicit: bool,
) -> IpcResult<typide_world::CompleteResponse> {
    Ok(typide_world::complete(
        &PathBuf::from(&root),
        &entrypoint,
        &path,
        &overlays,
        cursor,
        explicit,
    ))
}

/// Spell-check Typst markup prose, returning misspellings + suggestions.
#[tauri::command]
fn spellcheck(text: String) -> IpcResult<Vec<typide_check::Misspelling>> {
    Ok(typide_check::spellcheck(&text))
}

/// Map a preview click (page, point in pt) back to a source location.
#[tauri::command]
fn jump_from_click(
    root: String,
    entrypoint: String,
    overlays: Vec<typide_world::Overlay>,
    page: usize,
    x: f64,
    y: f64,
) -> IpcResult<Option<typide_world::JumpTarget>> {
    Ok(typide_world::jump_from_click(
        &PathBuf::from(&root),
        &entrypoint,
        &overlays,
        page,
        x,
        y,
    ))
}

/// Map a cursor (UTF-16 offset in `path`) to a location in the rendered
/// document, for editor→preview sync.
#[tauri::command]
fn jump_from_cursor(
    root: String,
    entrypoint: String,
    path: String,
    overlays: Vec<typide_world::Overlay>,
    cursor: usize,
) -> IpcResult<Option<typide_world::PreviewPosition>> {
    Ok(typide_world::jump_from_cursor(
        &PathBuf::from(&root),
        &entrypoint,
        &path,
        &overlays,
        cursor,
    ))
}

/// Render a page (1-based) to a PNG at `pixelPerPt` for a high-fidelity raster
/// preview. Returns the raw bytes (an ArrayBuffer on the JS side).
#[tauri::command]
fn render_png(
    root: String,
    entrypoint: String,
    overlays: Vec<typide_world::Overlay>,
    page: usize,
    pixel_per_pt: f64,
) -> IpcResult<tauri::ipc::Response> {
    let bytes = typide_world::render_page_png(
        &PathBuf::from(&root),
        &entrypoint,
        &overlays,
        page,
        pixel_per_pt,
    )
    .ok_or_else(|| IpcError::new("render.png", "could not render page"))?;
    Ok(tauri::ipc::Response::new(bytes))
}

/// Hover information at a UTF-16 cursor offset in `path`.
#[tauri::command]
fn hover(
    root: String,
    entrypoint: String,
    path: String,
    overlays: Vec<typide_world::Overlay>,
    cursor: usize,
) -> IpcResult<Option<typide_world::Hover>> {
    Ok(typide_world::hover(
        &PathBuf::from(&root),
        &entrypoint,
        &path,
        &overlays,
        cursor,
    ))
}

fn write(root: &Path, rel: &str, body: &str) -> std::io::Result<()> {
    let p = root.join(rel);
    if let Some(parent) = p.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(p, body)
}

fn scaffold(root: &Path, p: &CreateParams) -> std::io::Result<()> {
    let name = &p.name;
    let kind = if p.template_id == "template" {
        "paper"
    } else {
        &p.template_id
    };
    let mode = if p.packages_mode == "vendor" {
        "vendor"
    } else {
        "cache"
    };

    let mut toml = format!(
        "[project]\nname = \"{name}\"\nkind = \"{kind}\"\nentrypoint = \"main.typ\"\n\n\
         [toolchain]\ntypst = \"{}\"\ntinymist = \"auto\"\n\n\
         [packages]\nmode = \"{mode}\"\n",
        p.typst
    );
    if mode == "vendor" {
        toml.push_str("vendor-dir = \"vendor/packages\"\n");
    }
    toml.push_str("\n[fonts]\nuse-system-fonts = true\n");
    if p.refs {
        toml.push_str("\n[references]\nfiles = [\"refs.bib\"]\n");
    }
    toml.push_str(
        "\n[[export]]\nname = \"PDF\"\nformat = \"pdf\"\noutput = \"out/main.pdf\"\n\n\
         [check]\nprofile = \"default\"\n",
    );
    write(root, "typide.toml", &toml)?;

    let title = name.replace(['-', '_'], " ");
    let cite = if p.refs {
        "\n\n#bibliography(\"refs.bib\", style: \"ieee\")\n"
    } else {
        "\n"
    };

    let main = match kind {
        "thesis" => format!(
            "#set document(title: \"{title}\", author: \"\")\n\
             #set page(paper: \"a4\", margin: 2.5cm, numbering: \"1\")\n\
             #set text(font: \"New Computer Modern\", size: 11pt, lang: \"en\")\n\
             #set heading(numbering: \"1.1\")\n\n\
             #align(center)[\n  #v(4cm)\n  #text(20pt, weight: \"bold\")[{title}]\n  \
             #v(1cm)\n  #text(14pt)[Your Name]\n  #v(0.4cm)\n  \
             Doctoral Thesis · #datetime.today().display()\n]\n\n\
             #pagebreak()\n#outline(title: \"Contents\", indent: auto)\n#pagebreak()\n\n\
             = Introduction <intro>\n#include \"chapters/01-introduction.typ\"\n{cite}"
        ),
        "paper" => format!(
            "#set document(title: \"{title}\")\n\
             #set page(paper: \"a4\", margin: 2.2cm, numbering: \"1\")\n\
             #set text(size: 10.5pt, lang: \"en\")\n#set par(justify: true)\n\
             #set heading(numbering: \"1.\")\n\n\
             #align(center)[\n  #text(17pt, weight: \"bold\")[{title}]\n  #v(6pt)\n  \
             #text(11pt)[Your Name]\n]\n#v(10pt)\n\n\
             #align(center)[*Abstract*]\n#pad(x: 1.5cm)[\n  A concise summary of the work.\n]\n\n\
             = Introduction <intro>\nWrite your introduction here.\n{cite}"
        ),
        "notes" => format!(
            "#set page(width: 18cm, height: auto, margin: 1.6cm)\n\
             #set text(size: 11pt)\n\n= {title}\n\nWelcome to your research notebook.\n\n\
             - Create notes under `notes/`.\n- Link between notes with `[[note-name]]`.\n"
        ),
        "package" => format!(
            "// {title} — library entry point.\n\n\
             #let hello(name) = [Hello, #name!]\n"
        ),
        _ => format!("#set page(numbering: \"1\")\n\n= {title}\n\nStart writing.\n{cite}"),
    };
    write(root, "main.typ", &main)?;

    if kind == "thesis" {
        write(
            root,
            "chapters/01-introduction.typ",
            "This is the introduction. Reference a figure with @fig-example.\n\n\
             #figure(\n  rect(width: 80%, height: 3cm, fill: luma(240)),\n  \
             caption: [An example figure.],\n) <fig-example>\n",
        )?;
    }
    if p.refs {
        write(
            root,
            "refs.bib",
            "@article{example2024,\n  title  = {An Example Reference},\n  \
             author = {Doe, Jane},\n  journal = {Journal of Examples},\n  year = {2024}\n}\n",
        )?;
    }
    if p.vault && kind != "package" {
        write(root, "notes/index.typ", "= Notes\n\nYour vault index.\n")?;
    }
    if kind == "package" {
        write(
            root,
            "typst.toml",
            &format!(
                "[package]\nname = \"{}\"\nversion = \"0.1.0\"\nentrypoint = \"main.typ\"\n\
                 authors = [\"you\"]\nlicense = \"MIT\"\n",
                name.to_lowercase().replace([' ', '_'], "-")
            ),
        )?;
        write(
            root,
            "README.md",
            &format!("# {title}\n\nA Typst package.\n"),
        )?;
    }
    write(root, ".gitignore", "/out\n/target\n*.pdf\n")?;
    Ok(())
}

// ── Crash recovery: autosaved drafts outside the project (§ unsaved-work) ─────
//
// Dirty editor buffers are periodically written here (NOT into the project, so
// there is no autosave-to-disk / file-watcher churn). On reopen, drafts newer
// than the file on disk are offered back to the user. Each draft is one JSON
// file named by a stable hash of the absolute file path.

/// A recoverable draft (metadata only; content is read on demand).
#[derive(Serialize)]
struct RecoveryDraft {
    path: String,
    #[serde(rename = "savedAt")]
    saved_at: u64,
}

#[derive(Serialize, Deserialize)]
struct DraftFile {
    path: String,
    saved_at: u64,
    content: String,
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn mtime_millis(p: &Path) -> Option<u64> {
    std::fs::metadata(p)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
}

/// Stable, filesystem-safe filename for a draft of `abs_path`.
fn draft_name(abs_path: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    abs_path.hash(&mut h);
    format!("{:016x}.json", h.finish())
}

fn recovery_dir(app: &AppHandle) -> Result<PathBuf, IpcError> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| IpcError::new("recovery.dir", format!("no app data dir: {e}")))?
        .join("recovery");
    std::fs::create_dir_all(&dir).map_err(|e| IpcError::new("recovery.dir", e.to_string()))?;
    Ok(dir)
}

/// Autosave a dirty buffer's content to the recovery store.
#[tauri::command]
fn recovery_save(app: AppHandle, path: String, content: String) -> IpcResult<()> {
    let file = recovery_dir(&app)?.join(draft_name(&path));
    let draft = DraftFile {
        path: path.clone(),
        saved_at: now_millis(),
        content,
    };
    let json =
        serde_json::to_vec(&draft).map_err(|e| IpcError::new("recovery.enc", e.to_string()))?;
    std::fs::write(&file, json).map_err(|e| IpcError::new("recovery.write", format!("{path}: {e}")))
}

/// List drafts that are newer than their on-disk file (or whose file is gone)
/// and whose content actually differs — i.e. genuine unsaved work to recover.
#[tauri::command]
fn recovery_scan(app: AppHandle) -> IpcResult<Vec<RecoveryDraft>> {
    let dir = recovery_dir(&app)?;
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(&dir) else {
        return Ok(out);
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Ok(bytes) = std::fs::read(&p) else {
            continue;
        };
        let Ok(draft) = serde_json::from_slice::<DraftFile>(&bytes) else {
            continue;
        };
        let disk = Path::new(&draft.path);
        let recoverable = match (std::fs::read_to_string(disk), mtime_millis(disk)) {
            // File exists: recover only if the draft is newer and differs.
            (Ok(on_disk), Some(mt)) => draft.saved_at > mt && on_disk != draft.content,
            // File missing: the draft is all that's left.
            _ => true,
        };
        if recoverable {
            out.push(RecoveryDraft {
                path: draft.path,
                saved_at: draft.saved_at,
            });
        } else {
            // Stale draft (file was saved since): clean it up.
            let _ = std::fs::remove_file(&p);
        }
    }
    Ok(out)
}

/// Return a draft's saved content.
#[tauri::command]
fn recovery_read(app: AppHandle, path: String) -> IpcResult<String> {
    let file = recovery_dir(&app)?.join(draft_name(&path));
    let bytes =
        std::fs::read(&file).map_err(|e| IpcError::new("recovery.read", format!("{path}: {e}")))?;
    let draft: DraftFile =
        serde_json::from_slice(&bytes).map_err(|e| IpcError::new("recovery.dec", e.to_string()))?;
    Ok(draft.content)
}

/// Drop a single draft (after it's saved or explicitly discarded).
#[tauri::command]
fn recovery_discard(app: AppHandle, path: String) -> IpcResult<()> {
    let file = recovery_dir(&app)?.join(draft_name(&path));
    if file.exists() {
        std::fs::remove_file(&file).map_err(|e| IpcError::new("recovery.rm", e.to_string()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> PathBuf {
        let p = std::env::temp_dir()
            .join(format!("typide-test-{}", std::process::id()))
            .join(format!("{:?}", std::time::SystemTime::now()));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn params(root: &Path, kind: &str) -> CreateParams {
        CreateParams {
            path: root.to_string_lossy().to_string(),
            template_id: kind.into(),
            name: "my-thesis".into(),
            typst: "0.15.1".into(),
            packages_mode: "vendor".into(),
            git_init: false,
            refs: true,
            vault: true,
            template_spec: None,
            online: true,
        }
    }

    #[test]
    fn expand_tilde_resolves_home_and_never_yields_a_literal_tilde() {
        // Absolute and plain relative paths pass through untouched.
        let abs = if cfg!(windows) {
            "C:\\tmp\\proj"
        } else {
            "/tmp/proj"
        };
        assert_eq!(expand_tilde(abs), PathBuf::from(abs));
        assert_eq!(
            expand_tilde("research/thesis"),
            PathBuf::from("research/thesis")
        );

        // "~/…" must expand to an absolute home path — never a literal "~".
        if home_dir().is_some() {
            let expanded = expand_tilde("~/research");
            assert!(
                expanded.is_absolute(),
                "expected absolute, got {expanded:?}"
            );
            assert!(expanded.ends_with("research"));
            assert_ne!(expanded, PathBuf::from("~/research"));
            assert_eq!(expand_tilde("~"), home_dir().unwrap());
        }
    }

    #[test]
    fn fs_ops_create_rename_delete() {
        let root = tmp();
        let a = root.join("a.typ");
        fs_create_file(a.to_string_lossy().to_string()).unwrap();
        assert!(a.exists());
        // Creating again errors.
        assert!(fs_create_file(a.to_string_lossy().to_string()).is_err());
        let b = root.join("sub/b.typ");
        fs_rename(
            a.to_string_lossy().to_string(),
            b.to_string_lossy().to_string(),
        )
        .unwrap();
        assert!(!a.exists() && b.exists(), "rename moves + creates parent");
        fs_delete(root.join("sub").to_string_lossy().to_string()).unwrap();
        assert!(!b.exists());
    }

    #[test]
    fn unique_path_avoids_collisions() {
        let root = tmp();
        std::fs::write(root.join("img.png"), b"x").unwrap();
        let u = unique_path(&root, "img.png");
        assert_eq!(u.file_name().unwrap().to_string_lossy(), "img (2).png");
        assert_eq!(percent_decode("a%20b.png"), "a b.png");
    }

    #[test]
    fn draft_name_is_stable_and_distinct() {
        let a = draft_name("/home/u/proj/main.typ");
        assert_eq!(
            a,
            draft_name("/home/u/proj/main.typ"),
            "same path → same name"
        );
        assert_ne!(
            a,
            draft_name("/home/u/proj/chapter.typ"),
            "different path → different name"
        );
        assert!(a.ends_with(".json"));
    }

    #[test]
    fn scaffold_thesis_writes_real_files() {
        let root = tmp();
        scaffold(&root, &params(&root, "thesis")).unwrap();
        assert!(root.join("typide.toml").exists());
        assert!(root.join("main.typ").exists());
        assert!(root.join("chapters/01-introduction.typ").exists());
        assert!(root.join("refs.bib").exists());
        assert!(root.join("notes/index.typ").exists());

        let cfg = std::fs::read_to_string(root.join("typide.toml")).unwrap();
        assert_eq!(read_field(&cfg, "name"), Some("my-thesis"));
        assert_eq!(read_field(&cfg, "kind"), Some("thesis"));
    }

    #[test]
    fn load_project_reads_back_config() {
        let root = tmp();
        scaffold(&root, &params(&root, "paper")).unwrap();
        let info = load_project(&root);
        assert_eq!(info.name, "my-thesis");
        assert_eq!(info.kind, "paper");
        assert_eq!(info.entrypoint, "main.typ");
    }

    #[test]
    #[ignore = "hits the network + heavy compile"]
    fn template_project_instantiates_and_compiles() {
        let dir = std::env::temp_dir().join(format!("typide-tpl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let entry = typide_packages::init_template("@preview/charged-ieee:0.1.3", &dir, true)
            .expect("init template");
        assert!(dir.join(&entry).exists(), "entrypoint {entry} laid down");
        let result = typide_world::compile(&dir, &entry, &[]);
        for d in &result.diagnostics {
            eprintln!("  [{}] {}:{} {}", d.severity, d.file, d.line, d.message);
        }
        assert!(
            result.page_count >= 1,
            "template should compile to >= 1 page"
        );
    }

    #[test]
    fn build_tree_lists_entries_and_skips_ignored() {
        let root = tmp();
        scaffold(&root, &params(&root, "thesis")).unwrap();
        std::fs::create_dir_all(root.join("target/debug")).unwrap();
        let tree = build_tree(&root, 0).unwrap();
        let kids = tree.children.unwrap();
        assert!(kids.iter().any(|n| n.name == "main.typ" && n.kind == "typ"));
        assert!(kids.iter().any(|n| n.name == "chapters" && n.kind == "dir"));
        assert!(!kids.iter().any(|n| n.name == "target"));
    }
}

/// Build and run the Tauri application. Called from `main.rs` on desktop.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "typide=info".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Arc::new(JobRegistry::default()))
        .invoke_handler(tauri::generate_handler![
            workspace_open,
            workspace_create,
            read_tree,
            scan_project,
            compile,
            packages_fetch,
            packages_make_offline_ready,
            packages_mirror_create,
            packages_search,
            packages_refresh_index,
            fonts_list,
            export_pdf,
            complete,
            hover,
            spellcheck,
            jump_from_click,
            jump_from_cursor,
            render_png,
            job_cancel,
            toolchain_list,
            toolchain_available,
            toolchain_install,
            toolchain_install_from_archive,
            toolchain_remove,
            policy_get,
            vcs_snapshot,
            vcs_timeline,
            vcs_restore,
            vcs_get_remote,
            vcs_set_remote,
            vcs_sync,
            refs_zotero_status,
            refs_zotero_import,
            file_read,
            file_write,
            fs_create_file,
            fs_create_dir,
            fs_rename,
            fs_delete,
            fs_write_bytes,
            fs_import,
            fs_reveal,
            recovery_save,
            recovery_scan,
            recovery_read,
            recovery_discard,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Typide");
}
