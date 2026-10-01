# Typide — Architecture & Design Document

> **Working name:** `typide` (rename freely; search-and-replace the crate prefix).
> **Status:** Draft v0.1 · Target Typst version: **0.15.1** · License: **Apache-2.0**
> **Audience:** human contributors and AI coding agents (Claude Code). This document is the source of truth for architecture decisions. When code and this doc disagree, either fix the code or update this doc in the same PR.

---

## 0. How to use this document (for Claude Code)

1. Work **milestone by milestone** (see §17). Do not start a milestone before the previous one's acceptance criteria pass.
2. Each milestone lists **tasks** and **acceptance criteria**. Treat acceptance criteria as tests to write first where possible.
3. Respect the **principles** in §2. If a task seems to require breaking one (e.g. a network call in a core path), stop and leave a `// DESIGN-QUESTION:` comment plus an entry in `docs/open-questions.md` instead of improvising.
4. Items marked **[VERIFY]** are facts about external systems (Typst, Tinymist, Zotero, Tauri) that must be checked against current upstream docs/source before implementation. Record findings in `docs/verified-facts.md`.
5. Keep crates small, with public APIs documented (`#![deny(missing_docs)]` in library crates).
6. Every new crate gets unit tests; every Tauri command gets an integration test in `app/src-tauri/tests/`.

---

## 1. Vision & scope

**Typide is a free, open-source, offline-first IDE for Typst**, in the spirit of IntelliJ: it understands the *whole project* and offers trustworthy navigation, refactoring, inspections, and tooling. Its flagship use case is **academic writing** — theses, papers, and research notes for students and PhD researchers — including institutions with strict data-protection requirements.

### Goals
- Full functionality **without an internet connection** and **without an account**.
- A project is **plain files** (`.typ`, `.md`, `.bib`, `typst.toml`, `typide.toml`); nothing is locked into the app.
- The IDE **manages its own toolchain**: Typst compiler, Tinymist language server, fonts, and packages — installable, versioned, and syncable offline.
- Deep project understanding: index, refactoring, inspections with quick-fixes, style inspector, version migration.
- Academic workflow: notes vault, Zotero integration, snapshots, thesis templates, pre-submission checker, PDF/A + PDF/UA export.

### Non-goals (for now)
- Real-time multi-user collaboration (the official Typst web app does this well).
- A cloud service run by the project. Any server component is **self-hostable** only.
- Reimplementing the Typst compiler or language server.
- Mobile apps.

---

## 2. Principles (hard rules)

| # | Principle | Consequence for code |
|---|-----------|----------------------|
| P1 | **Offline-first** | No core feature may require the network. Network use is isolated in `typide-net` and gated by `NetworkPolicy`. |
| P2 | **No telemetry** | No analytics, crash upload, or phone-home. Logs are local only. Update checks are opt-in and disableable by policy. |
| P3 | **Plain files, no lock-in** | The SQLite index is a cache; deleting it must lose nothing. All user data lives in the project folder or documented app dirs. |
| P4 | **Upstream-first** | Prefer contributing to `typst` / `tinymist` over forking. Wrap external tools behind traits so they can be swapped. |
| P5 | **Reproducible builds of documents** | A project + its lockfile + its toolchain version must produce the same PDF on any machine. |
| P6 | **Admin-controllable** | Institutions can enforce settings (offline mode, mirrors, AI off) via a policy file (§8.5). |
| P7 | **Responsive UI** | Nothing blocking on the UI thread; long work runs as cancellable background jobs that report progress. |

---

## 3. Technology stack

| Layer | Choice | Notes |
|-------|--------|-------|
| App shell | **Tauri v2** | Small binaries, Rust backend, system webview. |
| Backend language | **Rust** (stable, MSRV ≥ 1.92 to match Typst 0.15) | Cargo workspace. |
| Frontend | **TypeScript + Svelte 5** (or SolidJS) + **Vite** | Pick one in M0 and record in `docs/decisions/`. Avoid heavy frameworks. |
| Editor component | **CodeMirror 6** | LSP client via `@codemirror/lsp-client`-style adapter or custom bridge. |
| Compiler | `typst` crates **0.15.1** (`typst`, `typst-pdf`, `typst-svg`, `typst-html`, `typst-render`, `typst-ide`, `typst-kit`) | [VERIFY] exact crate names/APIs; `typst-kit` was reworked in 0.15 to simplify `World` implementations. |
| Language server | **Tinymist** (managed binary, stdio LSP) | Run as subprocess, not linked. Version managed by toolchain manager. |
| Index / cache DB | **SQLite** via `rusqlite` (bundled feature) | FTS5 for notes/full-text search. |
| Version control | `git2` (libgit2) | Snapshots + remote sync. Consider `gix` later. |
| HTTP | `reqwest` (rustls, no native-tls) | Only inside `typide-net`. |
| Archives / hashing | `flate2`, `tar`, `zip`, `sha2` | Package tarballs, toolchain archives. |
| Async runtime | `tokio` | Background jobs. |
| Secrets | `keyring` crate | Git credentials, API tokens; never in plain files. |
| Logging | `tracing` + `tracing-appender` | Rolling local log files. |
| Serialization | `serde`, `toml`, `serde_json` | |
| Errors | `thiserror` (libs), `anyhow` (app/binaries) | |

---

## 4. High-level architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│                          Frontend (WebView)                          │
│  Editor (CM6) │ Preview │ Project tree │ Notes │ Problems │ Jobs     │
│        ▲ LSP messages          ▲ Tauri commands / events             │
└────────┼───────────────────────┼─────────────────────────────────────┘
         │                       │
┌────────┼───────────────────────┼─────────────────────────────────────┐
│        │         Rust backend (app/src-tauri)                        │
│  ┌─────┴──────┐  ┌────────────┴───────────┐  ┌────────────────────┐  │
│  │ typide-lsp │  │   typide-core (facade) │  │ typide-jobs        │  │
│  │ (Tinymist  │  │ Workspace / Project    │  │ (background tasks, │  │
│  │  bridge)   │  │ model, events          │  │  progress, cancel) │  │
│  └─────┬──────┘  └──┬───────┬───────┬─────┘  └────────────────────┘  │
│        │            │       │       │                                │
│  ┌─────┴─────┐ ┌────┴───┐ ┌─┴─────┐ ┌┴──────────┐ ┌───────────────┐  │
│  │ Tinymist  │ │ world  │ │ index │ │ export /  │ │ check         │  │
│  │ process   │ │(Typst  │ │(SQLite│ │ preview   │ │ (inspections, │  │
│  │ (stdio)   │ │ World) │ │ FTS5) │ │           │ │ quick-fixes)  │  │
│  └───────────┘ └───┬────┘ └───────┘ └───────────┘ └───────────────┘  │
│                    │                                                 │
│  ┌──────────────┐ ┌┴────────────┐ ┌───────────┐ ┌──────────────────┐ │
│  │ toolchain    │ │ packages    │ │ fonts     │ │ notes / refs /   │ │
│  │ (typst,      │ │ (resolve,   │ │ (bundled, │ │ vcs / ai         │ │
│  │  tinymist)   │ │  cache,sync)│ │  system)  │ │                  │ │
│  └──────┬───────┘ └──────┬──────┘ └───────────┘ └────────┬─────────┘ │
│         └────────┬───────┴──────────────────────────────┘           │
│            ┌─────┴──────┐        ┌──────────────────┐                │
│            │ typide-net │◄───────┤ NetworkPolicy    │◄── policy.toml │
│            │ (ONLY place│        │ (offline switch, │    user prefs  │
│            │  with HTTP)│        │  mirrors, allow) │                │
│            └────────────┘        └──────────────────┘                │
└──────────────────────────────────────────────────────────────────────┘
```

### Key flows
- **Typing:** Editor → LSP (Tinymist) for completion/diagnostics/hover. Editor → `core` (debounced, ~150 ms) → `world` incremental compile → preview update + our own inspections.
- **Opening a project:** `core` loads `typide.toml` → `toolchain` ensures versions → `packages` resolves imports (offline-first) → `index` scans files → LSP started for the project root.
- **Export:** User picks an export profile → job → `export` compiles with profile options → writes files → `check` runs post-export validations (e.g. PDF standard).

### Why two compilation paths (Tinymist + embedded)?
Tinymist gives mature editor intelligence; our embedded `World` gives full control for preview, export profiles, inspections, and the style/context debugger. Cost: some duplicated CPU. Mitigation: debounce, share the package cache and font set, and revisit in M5 (possible direction: link `tinymist-query`/`tinymist-project` crates directly — [VERIFY] availability and API stability).

---

## 5. Repository layout

```
typide/
├── Cargo.toml                # workspace
├── rust-toolchain.toml
├── ARCHITECTURE.md           # this file
├── CLAUDE.md                 # agent conventions (see Appendix A)
├── docs/
│   ├── decisions/            # ADRs: 0001-frontend-framework.md, ...
│   ├── open-questions.md
│   └── verified-facts.md
├── crates/
│   ├── typide-core/          # workspace/project model, config, event bus, facade
│   ├── typide-world/         # Typst World impl (files, packages, fonts, time)
│   ├── typide-toolchain/     # install/manage typst CLI + tinymist versions
│   ├── typide-packages/      # package resolution, cache, lockfile, vendoring, mirrors
│   ├── typide-fonts/         # bundled + system + project fonts, font DB
│   ├── typide-net/           # the ONLY crate doing network I/O; NetworkPolicy
│   ├── typide-lsp/           # Tinymist process mgmt + LSP message bridge
│   ├── typide-index/         # SQLite index: symbols, labels, citations, notes (FTS5)
│   ├── typide-export/        # preview rendering + export profiles (PDF/HTML/SVG/PNG/bundle)
│   ├── typide-check/         # inspections, quick-fixes, migrations, submission checker
│   ├── typide-notes/         # notes vault, backlinks, note→thesis embedding
│   ├── typide-refs/          # bibliography: .bib/.yml, Zotero local API
│   ├── typide-vcs/           # snapshots + git remote sync
│   ├── typide-jobs/          # background job runner, progress, cancellation
│   ├── typide-policy/        # policy.toml + user settings merge
│   └── typide-ai/            # (M6) provider abstraction, local-model first, AI-use log
├── app/
│   ├── src-tauri/            # Tauri binary: commands, events, window mgmt
│   └── ui/                   # frontend (TS)
├── resources/
│   ├── fonts/                # bundled fonts (license files alongside)
│   ├── templates/            # built-in project templates
│   └── toolchains/           # (build-time) bundled typst + tinymist per target
├── xtask/                    # cargo xtask: fetch bundled toolchains, release, etc.
└── tests/
    └── fixtures/             # sample projects (thesis, notes, broken refs, old syntax)
```

Dependency rule: `typide-net` is depended on **only** by `toolchain`, `packages`, `refs`, `vcs`, `ai`. CI enforces this with a script (`xtask check-deps`).

---

## 6. Core domain model

### 6.1 Workspace & project
```rust
pub struct Workspace { projects: Vec<ProjectHandle>, settings: EffectiveSettings }

pub struct Project {
    pub root: PathBuf,              // Typst --root
    pub config: ProjectConfig,      // from typide.toml
    pub lock: Option<LockFile>,     // from typide.lock
    pub kind: ProjectKind,          // Thesis | Paper | Notes | Package | Generic
}
```
A folder becomes a Typide project when it contains `typide.toml`. A folder with only `.typ` files can be opened; the IDE offers to create `typide.toml` with inferred defaults.

### 6.2 `typide.toml` (project config, committed to git)
```toml
[project]
name = "phd-thesis"
kind = "thesis"                 # thesis | paper | notes | package | generic
entrypoint = "main.typ"

[toolchain]
typst = "0.15.1"                # exact version; see §9
tinymist = "auto"               # "auto" = latest compatible installed/bundled

[packages]
mode = "vendor"                 # "cache" | "vendor"   (see §10.4)
vendor-dir = "vendor/packages"
mirrors = []                    # extra package mirrors, tried in order before Universe

[fonts]
paths = ["fonts"]               # project-local font dirs
use-system-fonts = true

[notes]
dir = "notes"

[references]
files = ["refs.bib"]
zotero = { enabled = true, collection = "PhD", export = "refs.bib", mode = "local-api" }

[[export]]                       # export profiles ("run configurations")
name = "Submission PDF"
format = "pdf"
output = "out/thesis.pdf"
pdf-standards = ["a-2a", "ua-1"] # Typst 0.15 supports multiple compatible standards [VERIFY identifiers]
pretty = false

[[export]]
name = "Notes website"
format = "html"
input = "notes/index.typ"
output = "out/site"
features = ["html"]

[check]
profile = "thesis-default"      # which inspection set the submission checker uses
```

### 6.3 `typide.lock` (generated, committed)
See §10.3.

### 6.4 Events
`typide-core` exposes a typed event bus (`tokio::sync::broadcast`). The Tauri layer forwards selected events to the frontend.

```rust
pub enum CoreEvent {
    FileChanged { path: PathBuf },
    CompileFinished { project: ProjectId, duration_ms: u64, diagnostics: Vec<Diagnostic> },
    PreviewUpdated { project: ProjectId, pages: Vec<PageDelta> },
    JobProgress { job: JobId, fraction: f32, message: String },
    JobFinished { job: JobId, result: JobResult },
    PackageStateChanged { project: ProjectId },
    ToolchainStateChanged,
    NetworkPolicyChanged { offline: bool },
}
```

---
## 7. App directories & storage

All paths come from the `directories` crate (`ProjectDirs::from("org", "typide", "typide")`). Never hardcode OS paths.

| Purpose | Location (example Linux) | Contents |
|---------|--------------------------|----------|
| Config | `~/.config/typide/` | `settings.toml`, keymaps, UI state |
| Data | `~/.local/share/typide/` | `toolchains/`, user templates, local package namespaces (see below) |
| Cache | `~/.cache/typide/` | index DBs (`index/<project-hash>.sqlite`), preview cache, download temp |
| Logs | `<data>/logs/` | rolling `tracing` logs, 7 days default |
| Shared Typst dirs | Typst's own cache/data dirs | **Package cache is shared with the Typst CLI** so both tools see the same packages. [VERIFY] Typst's default package cache path per OS (e.g. `~/.cache/typst/packages`) and data path for local packages (e.g. `~/.local/share/typst/packages`), plus env vars `TYPST_PACKAGE_CACHE_PATH` / `TYPST_PACKAGE_PATH`. |

**Portable mode:** if a file `portable.flag` sits next to the executable, all of the above live in `./typide-data/` beside it (USB-stick use in labs/exam rooms).

### 7.1 SQLite index schema (cache only — rebuildable)
```sql
CREATE TABLE files     (id INTEGER PRIMARY KEY, path TEXT UNIQUE, hash TEXT, mtime INTEGER, kind TEXT);
CREATE TABLE symbols   (id INTEGER PRIMARY KEY, file_id INTEGER, name TEXT, kind TEXT,  -- let, func, param, import
                        start INTEGER, end INTEGER, scope TEXT);
CREATE TABLE labels    (id INTEGER PRIMARY KEY, file_id INTEGER, name TEXT, start INTEGER, end INTEGER, target_kind TEXT);
CREATE TABLE refs      (id INTEGER PRIMARY KEY, file_id INTEGER, target TEXT, start INTEGER, end INTEGER, kind TEXT); -- @label, cite, link
CREATE TABLE rules     (id INTEGER PRIMARY KEY, file_id INTEGER, kind TEXT, selector TEXT, start INTEGER, end INTEGER); -- set/show
CREATE TABLE imports   (id INTEGER PRIMARY KEY, file_id INTEGER, spec TEXT, start INTEGER, end INTEGER); -- files + packages
CREATE TABLE citations (key TEXT PRIMARY KEY, source TEXT, title TEXT, authors TEXT, year TEXT, raw TEXT);
CREATE TABLE notes     (id INTEGER PRIMARY KEY, file_id INTEGER, title TEXT, cite_key TEXT);
CREATE TABLE links     (src_note INTEGER, dst TEXT);  -- backlinks
CREATE VIRTUAL TABLE fts USING fts5(path, content);
```
Indexing uses the **Typst syntax parser** (`typst-syntax`) — never regex — to extract symbols, labels, refs, rules, and imports. Re-index per file on change (hash-based).

---

## 8. Installation, distribution & updates

### 8.1 Installers
Built in CI (GitHub Actions matrix) with `tauri build`:

| OS | Artifacts |
|----|-----------|
| Windows | `.msi` (for admins / silent install via `msiexec /qn`) and NSIS `.exe` |
| macOS | Universal `.dmg` (signed + notarized when certificates are available) |
| Linux | `.AppImage`, `.deb`, `.rpm`; Flatpak manifest in M5 |

Code signing secrets live only in CI. Unsigned builds must still work (document the OS warnings).

### 8.2 "Batteries included" — works offline from first launch
Each installer **bundles a default toolchain**: one Typst compiler version, one compatible Tinymist version, and the default font set (§11). `xtask fetch-toolchains` downloads these at build time, verifies SHA-256 against a pinned manifest (`resources/toolchains/manifest.toml`), and places them in the bundle resources. On first launch, the bundled toolchain is registered (not copied) as the default.

### 8.3 First-run wizard
1. Language & theme.
2. **Network mode:** *Offline (never use the network)* / *Ask each time* / *Online (allow downloads & update checks)*. Default: *Ask each time*. Locked if policy sets it.
3. Detect existing installs: Typst CLI on `PATH`, Tinymist, Zotero (local API reachable?), Git identity. Offer to use/import them.
4. Optional: import fonts folder, choose default templates (thesis/paper/notes).
5. Create or open a project.

### 8.4 App updates
- Uses `tauri-plugin-updater` with a signed update manifest hosted on GitHub Releases.
- **Opt-in**, never automatic download without consent; disabled entirely in offline mode or by policy.
- Update channel: `stable` / `beta`.
- Toolchain updates are separate from app updates (§9).

### 8.5 Institutional deployment (`policy.toml`)
Read from a system location (Windows: `%ProgramData%\typide\policy.toml`; macOS: `/Library/Application Support/typide/policy.toml`; Linux: `/etc/typide/policy.toml`). Policy values **override** user settings and are shown as locked in the UI.

```toml
[network]
mode = "offline"                 # offline | ask | online  (enforced)
allow-hosts = ["mirror.uni.example"]  # when not offline: only these hosts
package-mirrors = ["https://mirror.uni.example/typst-packages"]
toolchain-mirror = "https://mirror.uni.example/typide-toolchains"

[updates]
enabled = false

[ai]
enabled = false                  # or: providers = ["ollama"], endpoint allowlist

[templates]
extra-dirs = ["\\\\fileserver\\thesis-templates"]  # read-only template sources

[telemetry]
# intentionally absent: there is no telemetry to configure.
```

`typide-policy` merges: **defaults < user settings < project settings (only non-security keys) < policy**. Security-relevant keys (network, AI, updates) can never be loosened by project files.

---

## 9. Toolchain management ("SDKs")

Modeled after IntelliJ SDKs / `rustup`. A **toolchain** is a pair *(Typst compiler version, Tinymist version)* plus metadata.

### 9.1 Why manage versions at all?
Typst has frequent breaking changes (e.g. 0.15 removed `path`, `pattern`, `pdf.embed`, `*.decode`; changed baselines). A thesis started on 0.14 must keep compiling identically until the author chooses to migrate (P5).

### 9.2 Components
```
<data>/toolchains/
  typst/0.15.1/typst(.exe)
  typst/0.14.2/typst(.exe)
  tinymist/<ver>/tinymist(.exe)
  registry.toml          # installed versions, source, sha256, install date
```

- **Embedded compiler**: the app links `typst` crates of exactly one version (the "native" version, 0.15.1) used for preview/export/inspections.
- **External compilers**: other Typst versions run as the **CLI subprocess** for compile/export (`typst compile --root ... --package-path ... --font-path ...`). Preview for non-native versions renders via CLI SVG/PNG output (slower; acceptable).
- **Tinymist**: always a subprocess. Each Tinymist release is built against a specific Typst version [VERIFY how to query it, e.g. `tinymist --version` output]; the registry records this so the toolchain manager can pick a compatible pair.

### 9.3 Operations (`typide-toolchain` API)
```rust
trait ToolchainManager {
    fn list_installed(&self) -> Vec<Toolchain>;
    async fn list_available(&self, net: &Net) -> Result<Vec<ReleaseInfo>>;     // GitHub releases or mirror
    async fn install(&self, v: &Version, src: InstallSource, job: JobCtx) -> Result<Toolchain>;
    fn install_from_archive(&self, archive: &Path) -> Result<Toolchain>;     // fully offline install
    fn remove(&self, v: &Version) -> Result<()>;
    fn resolve_for(&self, cfg: &ToolchainConfig) -> Result<ResolvedToolchain>;
    fn detect_system(&self) -> Vec<Toolchain>;                                // PATH detection
}
```
- **Sources:** official GitHub releases (`typst/typst`, `Myriad-Dreamin/tinymist`) [VERIFY asset naming per target triple], an institutional mirror (same file layout), or a local archive file.
- **Integrity:** verify SHA-256; if upstream publishes checksums use them, otherwise pin known hashes in a signed manifest shipped with Typide and fall back to trust-on-first-use with a visible warning.
- **Project binding:** `typide.toml [toolchain] typst = "0.14.2"`. On open, if missing: offer *Install* (online), *Install from file…*, or *Use native 0.15.1 and run migration check* (§14.3).

### 9.4 Offline toolchain bundles
`typide toolchain pack 0.14.2 --out typst-0.14.2-linux-x64.typide-tc` produces a single archive (binaries + manifest + hashes) that can be carried on a USB stick and installed on air-gapped machines.

---

## 10. Package management & sync

Typst packages are imported as `#import "@namespace/name:version": ...` with **exact versions** (no ranges). The public registry is **Typst Universe** (`@preview` namespace). Local/private packages live in other namespaces (e.g. `@local`). [VERIFY] index URL (`https://packages.typst.org/preview/index.json`), tarball URL pattern (`https://packages.typst.org/preview/{name}-{version}.tar.gz`), and the `typst.toml` manifest schema (`[package]`, `[template]`, `compiler` field).

### 10.1 Responsibilities of `typide-packages`
1. **Discover** package imports in project sources (and transitively in package sources) using `typst-syntax`.
2. **Resolve** each `PackageSpec` through the resolution chain (§10.2).
3. **Fetch** missing packages when the network policy allows; otherwise report precisely what is missing.
4. **Lock** resolved packages with hashes (§10.3).
5. **Vendor** or **cache** according to project mode (§10.4).
6. **Browse/search** the Universe index (cached locally for offline browsing).
7. **Update**: detect newer versions and rewrite import strings via a refactoring (§14).
8. **Templates**: create projects from packages that declare `[template]`.
9. **Local/private packages**: manage user namespaces and git-backed packages (§10.6).

### 10.2 Resolution chain (first hit wins)
```
1. Project vendor dir        <project>/vendor/packages/{ns}/{name}/{ver}/
2. Local namespaces          <typst data>/packages/{ns}/{name}/{ver}/        (non-@preview)
3. Shared package cache      <typst cache>/packages/{ns}/{name}/{ver}/        (shared with Typst CLI)
4. Configured mirrors        policy mirrors, then project mirrors
5. Typst Universe            only if NetworkPolicy allows
```
`typide-world` implements Typst's `World::file`/`source` for package paths using this chain; it never downloads during a compile on the UI path. Missing packages produce a diagnostic with a quick-fix: *Fetch package* / *Install from file…*.

For external CLI compiles (other toolchain versions), pass `--package-path` and `--package-cache-path` [VERIFY flag names] so the CLI sees exactly the same resolution (vendor dir is exposed via a generated overlay directory if needed).

### 10.3 Lockfile `typide.lock`
Typst itself has no lockfile; exact versions are in the source. Typide adds one for **integrity and offline reproducibility**:
```toml
version = 1
typst = "0.15.1"

[[package]]
spec = "@preview/cetz:0.4.2"
source = "universe"               # universe | mirror:<url> | local | git:<url>#<rev> | file
sha256 = "9f2c…"                  # of the canonical tarball, or of a normalized dir tree
dependencies = ["@preview/oxifmt:0.2.1"]
fetched = "2026-09-30T10:12:00Z"
```
- Regenerated on resolve; diffs shown to the user before writing.
- On open, hashes are verified for vendored/cached packages; mismatch → warning + quick-fix *Re-fetch* / *Trust current*.
- Directory hashing: sort entries, hash `path\0bytes` pairs → deterministic across OSes.

### 10.4 Modes
- **cache** (default for notes/generic): packages live in the shared Typst cache; project stays small.
- **vendor** (default for `kind = "thesis"`): packages are copied into `vendor/packages/` and committed. Guarantees the thesis compiles years later without network — ideal for archival submission.
- Command **"Make project offline-ready"**: resolve everything, fetch what's missing, vendor if requested, verify hashes, and report ✅/❌ per package, font, and toolchain.

### 10.5 Mirrors & offline sync
- A **mirror** is a static directory tree (HTTP or file path) with the same layout as Universe: `index.json` + tarballs.
- `typide packages mirror create --from universe --packages list.txt --out ./mirror` builds a partial or full mirror (institutions host it on an intranet; students can carry it on USB).
- `typide packages mirror sync ./mirror` updates it incrementally (only new versions).
- **Index cache:** the Universe `index.json` is cached with an ETag; package search works offline against the last cached index, showing its age.

### 10.6 Local & private packages
- UI to create a package in a local namespace (`@local`, `@lab`, `@uni`…), scaffolding `typst.toml`, `lib.typ`, `README.md`, `LICENSE`.
- **Git-backed packages:** register `git:https://gitlab.uni.example/lab/thesis-tpl.git` pinned to a tag/commit; Typide clones it into the local namespace path `{ns}/{name}/{ver}/`. Sync = fetch + checkout of the pinned rev; updates require explicit bump.
- **Template sources:** local dirs, git repos, policy-provided network shares — all surfaced in *New Project*.

### 10.7 Package jobs (all cancellable, all progress-reporting)
`ResolveProject`, `FetchPackages`, `VendorPackages`, `VerifyLock`, `RefreshIndex`, `CheckUpdates`, `MirrorCreate`, `MirrorSync`.

---

## 11. Fonts

- **Bundled fonts** (in `resources/fonts/`, with license files): Typst's default families (e.g. Libertinus Serif, New Computer Modern + Math, DejaVu Sans Mono) [VERIFY exact set embedded by Typst 0.15 — if `typst-kit` can provide embedded fonts, reuse that], plus a few widely used academic fonts with permissive licenses.
- **Search order:** project `fonts.paths` → user font dirs → system fonts (if enabled) → bundled.
- For reproducibility (P5) the submission checker warns when a document uses a **system** font not present in the project; quick-fix: *Copy font into project* (license permitting — show the font's license metadata).
- Font DB built once per session with `fontdb`/`typst-kit` and shared between preview, export, and checker. Variable fonts supported (Typst 0.15).
- Font panel: list families, variants, variation axes, coverage, source path.

---
## 12. Language intelligence (`typide-lsp` + `typide-index`)

### 12.1 Tinymist bridge
- Spawn Tinymist (from the resolved toolchain) per workspace with stdio transport; restart with backoff on crash (max 3 in 60 s, then show a banner).
- Pass configuration: root, package paths, font paths, export disabled (Typide owns export) [VERIFY Tinymist config keys].
- The frontend speaks LSP through the backend (`lsp_send` command / `lsp://message` event) so the backend can **intercept and augment** responses (e.g. merge Typide inspections into diagnostics, add Typide code actions).

### 12.2 Features owned by Tinymist
Completion, hover, signature help, go-to-definition, document symbols, semantic tokens, formatting (via typstyle), basic diagnostics.

### 12.3 Features owned by Typide (built on the index)
- **Find usages** across the whole project for labels, functions, variables, citation keys.
- **Project-wide rename** of labels, functions, variables, citation keys (updates `.bib` if the key is local) — previewed as a diff before apply.
- **Structure view:** headings tree, figures, tables, equations, labels, show/set rules.
- **Go to file / symbol / label / citation** (fuzzy, `Shift Shift`-style "Search Everywhere").
- **Dependency graph:** file includes/imports and package usage.

Every refactoring produces a `WorkspaceEdit`, shown in a **preview dialog**, applied atomically, and is undoable as one step. Before applying, Typide auto-creates a snapshot (§16).

---

## 13. Preview & export (`typide-export`)

### 13.1 Preview
- Compile incrementally with the embedded compiler (Typst uses `comemo` memoization — keep a long-lived `World` per project to benefit).
- Render **per-page SVG** (`typst-svg`); send only changed pages to the frontend (hash per page).
- Virtualized page list; zoom; dark-mode page inversion option.
- **Source ↔ preview sync:** click in preview → `typst-ide::jump_from_click`; cursor in editor → `jump_from_cursor` to scroll preview [VERIFY function names in typst-ide 0.15].
- Non-native toolchain versions: preview via CLI `typst compile --format svg` into a temp dir, watched.

### 13.2 Export profiles
Defined in `typide.toml [[export]]` (§6.2), editable in a UI. Each profile = format + options + output path. Formats: PDF (standards, pretty, page ranges), HTML (feature-flagged, via embedded crate or CLI), SVG/PNG (per-page, DPI), **Bundle** (experimental; requires `bundle` feature; multi-file output). Export runs as a job and ends with post-export checks.

---

## 14. Inspections, quick-fixes, migrations (`typide-check`)

### 14.1 Inspection framework
```rust
pub trait Inspection: Send + Sync {
    fn id(&self) -> &'static str;                 // "unused-label"
    fn category(&self) -> Category;               // References | Style | Migration | Submission | Accessibility
    fn default_severity(&self) -> Severity;
    fn run(&self, ctx: &InspectionCtx) -> Vec<Finding>;   // ctx: index, syntax trees, compiled doc (optional)
}
pub struct Finding { range: FileRange, message: String, fixes: Vec<QuickFix> }
pub struct QuickFix { title: String, edit: WorkspaceEdit }
```
Inspection profiles (TOML) enable/disable and set severity; the thesis profile is stricter. Findings surface as LSP diagnostics + a Problems panel.

### 14.2 Initial inspections
| ID | What | Quick-fix |
|----|------|-----------|
| `broken-ref` | `@label` with no target | Create label / pick similar label |
| `unused-label` | Label never referenced | Remove |
| `missing-citation` | Cite key not in any bibliography | Search Zotero / add stub entry |
| `uncited-entry` | Bib entry never cited (thesis profile) | Remove / ignore |
| `figure-no-caption` | Figure without caption | Insert caption |
| `image-no-alt` | Image without `alt` (PDF/UA) | Insert `alt:` |
| `system-font` | Uses font not bundled in project | Copy font into project |
| `unpinned-package-hash` | Package not in lockfile | Resolve & lock |
| `backslash-path` | Windows backslash in paths (error in 0.15) | Replace with `/` |

### 14.3 Version migration assistant
A table-driven set of `Migration` rules per Typst release (`migrations/0.15.toml`), each with a syntax-tree matcher and rewrite:
- `path(...)` → `curve(...)` (manual review flagged — argument semantics differ)
- `pattern(...)` → `tiling(...)`
- `pdf.embed` → `pdf.attach`
- `json.decode(x)` / `csv.decode` / … → `json(x)` etc. (pass bytes)
- Renamed citation styles (e.g. `vancouver` → `nlm-citation-sequence`, `mla-8` → `mla`)
- Deprecated/renamed symbols (import list from the codex changelog)
- Informational warnings for behavior changes that can't be rewritten (baselines, `math.class` non-recursive, `lr`/`stretch` sizing, calligraphic letterforms → suggest `stylistic-set: 6`).

Flow: *Migrate project to Typst X.Y* → snapshot → run rules → diff preview grouped by rule → apply → compile with new toolchain → side-by-side **visual diff** of old vs new PDF pages (render both, pixel-diff, highlight changed pages).

### 14.4 Style inspector (M4)
Click an element in preview → show the ordered list of set/show rules that affected it, with source locations. Implementation: map the click to a content element via `jump_from_click`, collect active style chain from the embedded compiler [VERIFY what style introspection is reachable from public APIs; may require an upstream contribution — track in open questions].

### 14.5 Context/introspection debugger (M5)
Show counter/state values at locations, query results, and convergence diagnostics (Typst 0.15 reports detailed convergence issues). Start with a panel listing counters/states and their values per heading/page.

---

## 15. Academic workflow

### 15.1 Notes vault (`typide-notes`)
- Notes are files in `notes/` — `.typ` or `.md` (Markdown notes rendered in-app; `.typ` notes compile like any Typst file).
- Wiki links `[[note-name]]` in Markdown; in Typst notes, a tiny bundled package `@typide/notes` provides `#note-link("name")`.
- Backlinks panel, graph view (M5), daily note command, full-text search (FTS5).
- **Literature notes:** a note can declare `cite-key: smith2024` (front matter). Opening a citation in the editor shows its literature note; "Create literature note" from any cite key.
- **Embed into thesis:** drag a note block into a chapter → inserts the text (copy) or `#include` (for `.typ` notes) at user choice.

### 15.2 References (`typide-refs`)
- Parse `.bib` (biblatex) and Hayagriva `.yml`; index into `citations` table.
- **Zotero integration, offline:** Zotero 7 local API on `localhost` [VERIFY port (commonly 23119), endpoint paths, and that it must be enabled in Zotero settings]; fallback: watch a Better BibTeX auto-exported `.bib` file. No Zotero web API by default (would violate P1); optional, policy-gated.
- Citation picker: fuzzy search by author/title/year → inserts `@key` or `#cite(<key>)`.
- Mendeley: import via exported `.bib` (no live sync in v1).

### 15.3 Thesis templates & submission checker
- Built-in templates: generic thesis, paper, lecture notes, research notebook. University-specific templates are community-contributed packages or template dirs (policy can add institutional ones).
- **Submission checker** = inspection profile `thesis-submission` + post-export checks: PDF standard conformance flags set, fonts embedded, all refs resolved, no uncited entries, page/margin/word-count rules from a university rule file (`rules.toml` shipped with the template), accessibility (alt text, document title/language set).
- Output: a report (HTML/Typst) with ✅/⚠️/❌ that the student can hand to the graduate office.

---

## 16. Version control & sync (`typide-vcs`)

### 16.1 Snapshots (Git for non-Git users)
- Every project gets a git repo on creation (opt-out).
- **Snapshot** = commit with an auto message ("Snapshot before migration to 0.15.1") or user message. Auto-snapshots: before refactorings/migrations, on export of a "submission" profile, optional timer (e.g. hourly if changed).
- UI: timeline, compare two snapshots (text diff + visual PDF diff), restore file/project.
- Advanced users get a normal Git panel (branches, stage hunks, log).

### 16.2 Remote sync (GitHub / GitLab / any Git remote)
- Add remote by URL; auth via SSH agent or HTTPS token stored in OS keychain (`keyring`).
- **Sync** button = fetch → rebase-or-merge (default merge, configurable) → push. Conflicts open a 3-way merge editor for `.typ`/`.md`.
- Works with self-hosted GitLab — the institutional "sync server" is just a Git server (no custom backend needed in v1).
- Network gated by policy (host allowlist).

### 16.3 Supervisor review (M6)
- Comments stored as files in the repo: `.typide/review/<id>.toml` (anchor = file + range + snapshot commit + quoted text for re-anchoring). They travel with Git sync, so review works over any Git host with no extra server.
- Export review comments into the PDF as annotations (optional).

### 16.4 Settings sync
Export/import settings, keymaps, snippets, and inspection profiles as a single `.typide-settings.zip`; optionally keep them in a user-chosen Git repo.

---

## 17. Roadmap & milestones

> Each milestone ends with a tagged pre-release. Acceptance criteria are mandatory.

### M0 — Skeleton (1–2 weeks)
Tasks: Cargo workspace + crates stubs (§5); Tauri app opens a window; frontend framework chosen (ADR 0001); CI builds on Win/macOS/Linux; `xtask check-deps` enforces the `typide-net` rule; `CLAUDE.md` created.
**Accept:** `cargo test --workspace` green; CI produces unsigned installers for 3 OSes; app launches and shows an empty workspace.

### M1 — Editor + preview + project (4–6 weeks)
Tasks: open folder, file tree, CodeMirror with Typst highlighting; `typide-world` with native compiler; per-page SVG preview with incremental updates; click-to-jump both directions; `typide.toml` read/write; export profile "PDF"; background jobs + progress UI; local logging.
**Accept:** fixture `tests/fixtures/thesis-basic` compiles; editing a paragraph updates preview < 300 ms on a 50-page doc (reference machine documented); PDF export byte-for-byte reproducible across two runs.

### M2 — Toolchains, packages, fonts, offline (4–6 weeks)
Tasks: bundled toolchain; Tinymist bridge (completion, hover, diagnostics); `typide-net` + `NetworkPolicy` + offline switch; toolchain install/remove/from-archive; package resolution chain, fetch, lockfile, vendor mode, "Make project offline-ready"; Universe index cache + package browser; mirror create/sync CLI; fonts panel; first-run wizard; `policy.toml`.
**Accept:** with networking disabled at OS level, a fresh install can create a thesis from the built-in template and export PDF; a project importing `@preview/<pkg>` fetched once compiles offline afterwards; vendored project compiles on a second machine with no cache; policy `mode = "offline"` makes every network call fail closed (integration test with a mock server asserting zero requests).

### M3 — Academic workflow (4–6 weeks)
Tasks: notes vault + backlinks + FTS; `.bib`/Hayagriva parsing + citation picker; Zotero local API + BBT fallback; snapshots + timeline + restore; Git remote sync; thesis/paper/notes templates; export PDF/A+PDF/UA profile.
**Accept:** end-to-end scenario test: create thesis → add Zotero citation offline → write literature note → embed into chapter → snapshot → export submission PDF.

### M4 — The IntelliJ layer (6–8 weeks)
Tasks: full index; find usages; project-wide rename; structure view; Search Everywhere; inspection framework + §14.2 inspections; migration assistant for 0.14→0.15 with visual PDF diff; submission checker report; style inspector (prototype).
**Accept:** rename a label used in 30 files in one atomic, undoable step; migrating `tests/fixtures/old-0.14` produces a compiling project and a report of non-rewritable changes.

### M5 — Polish & ecosystem prep
Context/introspection debugger; graph view; Flatpak; performance pass (evaluate linking Tinymist crates); accessibility of the app UI itself (keyboard-only, screen reader labels); i18n (start with English + Italian + German).

### M6 — Review, AI, plugins
Supervisor review; AI (§19); plugin API design (WASM, capability-based, no network by default).

---

## 18. Frontend & IPC contract

### 18.1 Layout
IntelliJ-style: left tool windows (Project, Notes, Structure, Git), center editor tabs, right preview (detachable window), bottom tool windows (Problems, Jobs, Terminal-lite, Search). Command palette (`Ctrl/Cmd+Shift+P`) and Search Everywhere (`Shift Shift`). Keymaps: default + "IntelliJ" + "VS Code" presets.

### 18.2 Tauri commands (request/response)
Naming: `domain_action`. All return `Result<T, IpcError>` where `IpcError { code: String, message: String, details: Option<Value> }`.
```
workspace_open(path) -> ProjectInfo
workspace_create(path, template_id, params) -> ProjectInfo
file_read(path) -> String            file_write(path, content)     file_rename(from, to)
compile_request(project_id)          preview_page(project_id, page) -> Svg
jump_from_click(project_id, page, x, y) -> SourceLocation
export_run(project_id, profile) -> JobId
lsp_send(project_id, message)
index_search(query, kinds) -> Vec<SearchHit>
refactor_rename(project_id, target, new_name) -> WorkspaceEditPreview
edit_apply(preview_id)
check_run(project_id, profile) -> JobId
toolchain_list() / toolchain_install(version, source) -> JobId / toolchain_remove(version)
packages_resolve(project_id) -> JobId / packages_status(project_id) -> PackageStatus
packages_search(query) -> Vec<PackageInfo>  / packages_make_offline_ready(project_id) -> JobId
vcs_snapshot(project_id, message) / vcs_timeline(project_id) / vcs_restore(project_id, commit, paths)
vcs_sync(project_id) -> JobId
refs_search(query) -> Vec<Citation> / refs_zotero_status() -> ZoteroStatus
notes_backlinks(note) -> Vec<Backlink>
settings_get() -> EffectiveSettings  / settings_set(patch)   (policy-locked keys rejected)
network_set_mode(mode)               job_cancel(job_id)
```
### 18.3 Events (backend → frontend)
`core://event` carrying `CoreEvent` (§6.4), `lsp://message`, `job://progress`, `job://finished`.

### 18.4 Frontend state
One store per domain (workspace, editor tabs, preview, jobs, problems, settings). The backend is the source of truth; the frontend never writes files directly.

---

## 19. AI integration (M6, optional, off by default)

- `typide-ai` exposes a `Provider` trait; first implementation: **local models via Ollama-compatible HTTP on localhost**. Remote providers only if policy allows and user adds a key (keychain).
- Features: explain compile error, fix-it suggestions (always shown as a diff), LaTeX→Typst conversion of a pasted snippet, summarize a literature note, ask questions about Typst docs (bundled offline docs index — the Typst docs PDF/source).
- **AI-use log** (academic integrity): every accepted AI edit is appended to `.typide/ai-log.jsonl` (timestamp, file, range, prompt summary, model). Command *Generate AI-use declaration* produces a Typst appendix. Log is part of the project (user can decide to commit or not).
- Never send file contents anywhere without an explicit per-request or per-project consent.

---

## 20. Security & privacy

- `NetworkPolicy` checked in `typide-net` for **every** request: mode, host allowlist, mirror redirection. Offline mode = fail closed.
- All downloads: HTTPS only (except explicitly configured intranet mirrors flagged as such), size limits, SHA-256 verification, extraction guarded against path traversal (`..`, absolute paths, symlinks) — dedicated tests.
- Tauri: minimal capability set; CSP locked; no remote URLs loaded in the webview; filesystem access only through backend commands scoped to open projects + app dirs.
- Subprocesses (typst CLI, tinymist, git): arguments passed as arrays, never through a shell.
- Secrets only in OS keychain. Logs scrub tokens and home paths when exporting a bug report.
- `SECURITY.md` with disclosure process.

---

## 21. Testing, quality & CI

- **Unit tests** per crate; **snapshot tests** (`insta`) for inspection findings, migrations, lockfile output.
- **Fixture projects** in `tests/fixtures/`: `thesis-basic`, `notes-vault`, `broken-refs`, `old-0.14`, `packages-heavy`, `fonts-custom`, `bundle-site`.
- **Visual regression:** render fixture PDFs to PNG and compare against baselines (tolerance-based).
- **Offline tests:** run package/toolchain tests against a local mock HTTP server; a CI job runs the app test suite inside a network-less container.
- **Frontend:** Vitest for logic; Playwright (via `tauri-driver`/WebDriver) for smoke E2E in M3+.
- **CI (GitHub Actions):** fmt, clippy (`-D warnings`), tests, `xtask check-deps`, `cargo deny` (licenses/advisories), build installers on tags, generate SBOM.
- **Performance budget:** keystroke-to-preview < 300 ms on reference doc; cold start < 2 s; idle memory tracked per release.

---

## 22. Open questions (seed list)

1. Link Tinymist crates directly vs. subprocess only? (Decide in M5 based on stability of its crate APIs.)
2. How much style-chain introspection is available via public Typst APIs? Upstream contribution needed for the style inspector?
3. Does Typst Universe publish tarball checksums? If not, propose it upstream; meanwhile TOFU + lockfile.
4. HTML/bundle export are feature-flagged/experimental in 0.15 — how to present them in UI (label "Experimental")?
5. Visual PDF diff: render via `typst-render` (only for native version) or a PDF rasterizer (e.g. pdfium/hayro) for any version?
6. Markdown notes: render with a Typst-based Markdown package or a JS renderer? (Consistency vs. speed.)
7. Naming & branding: must not imply official affiliation with Typst GmbH; check their brand guidelines.

---

## Appendix A — `CLAUDE.md` starter (copy to repo root)

```markdown
# Agent conventions for typide
- Read ARCHITECTURE.md before any task. Follow the milestone order in §17.
- Principles in §2 are hard rules. Network I/O only in crates/typide-net.
- Use typst-syntax for parsing Typst; never regex over Typst source.
- Every public fn in library crates is documented; `cargo clippy -- -D warnings` must pass.
- Errors: thiserror in libs, anyhow in app. No unwrap() outside tests.
- Long operations run as jobs (typide-jobs) with progress + cancellation.
- Every refactoring returns a WorkspaceEdit preview; never write files directly from inspections.
- Tests: add/extend fixtures in tests/fixtures; use insta for snapshot tests.
- When unsure about an external API, mark [VERIFY], check upstream source, record in docs/verified-facts.md.
- Commands: `cargo xtask ci` (full local CI), `cargo tauri dev` (run app).
- Keep PRs scoped to one milestone task; update ARCHITECTURE.md when a design decision changes.
```

## Appendix B — Glossary
- **Toolchain:** pinned pair of Typst compiler + Tinymist versions.
- **Native version:** the Typst version linked into the app (0.15.1).
- **Vendor mode:** packages copied into the project and committed.
- **Mirror:** static copy of (part of) Typst Universe in the same layout.
- **Snapshot:** a Git commit created by Typide, presented as a version.
- **Policy:** admin-controlled settings file that overrides user settings.
