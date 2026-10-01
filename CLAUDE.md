# Agent conventions for typide

- Read `ARCHITECTURE.md` before any task. Follow the milestone order in §17.
- Principles in §2 are hard rules. **Network I/O only in `crates/typide-net`.**
- Use `typst-syntax` for parsing Typst; never regex over Typst source.
- Every public fn in library crates is documented; `#![deny(missing_docs)]` is on.
- `cargo clippy -- -D warnings` must pass. Errors: `thiserror` in libs, `anyhow`
  in app. No `unwrap()` outside tests.
- Long operations run as jobs (`typide-jobs`) with progress + cancellation.
- Every refactoring returns a `WorkspaceEdit` preview; never write files directly
  from inspections.
- Tests: add/extend fixtures in `tests/fixtures`; use `insta` for snapshot tests.
- When unsure about an external API, mark `// [VERIFY]`, check upstream source,
  record findings in `docs/verified-facts.md`. Design blockers go in
  `docs/open-questions.md` as a `// DESIGN-QUESTION:` comment + an entry there.
- Keep PRs scoped to one milestone task; update `ARCHITECTURE.md` when a design
  decision changes.

## Commands

| What | Command |
|------|---------|
| Full local CI | `cargo xtask ci` |
| Enforce network isolation | `cargo xtask check-deps` |
| Workspace tests | `cargo test --workspace` |
| Run the desktop app | `cargo tauri dev` (needs the frontend + system WebView) |
| Frontend dev server | `cd app/ui && npm run dev` |
| Frontend build | `cd app/ui && npm run build` |

## Workspace note

`app/src-tauri` is **excluded** from the default Cargo workspace because Tauri
needs the platform WebView libraries (e.g. `webkit2gtk-4.1` + `libsoup-3.0` on
Linux). Install those before `cargo tauri dev/build`. `cargo test --workspace`
covers the pure-Rust crates and stays green without them.

## Current status

M0 skeleton + a **working desktop app**. The Tauri backend (`app/src-tauri`)
has real filesystem commands — `workspace_open`, `workspace_create` (scaffolds
projects on disk per generator), `read_tree`, `scan_project`, `file_read`,
`file_write` — covered by unit tests. The Svelte 5 + CodeMirror 6 frontend is
wired to them through `lib/api.ts` (real in Tauri, in-memory demo VFS in a plain
browser):

- Welcome launcher with **persisted recent projects** (`Launcher.svelte`).
- New Project wizard that **actually creates files** (`NewProjectWizard.svelte`).
- First-run setup (`FirstRunSetup.svelte`).
- Real file tree → open → edit → **save** (Ctrl+S); dirty markers.
- **Live analysis** (`lib/analysis.ts`): structure, problems (broken-ref,
  uncited), packages and citations recompute as you type. This is presentation-
  layer only; the authoritative index still belongs in Rust via `typst-syntax`.
- Resizable panels (`Resizer.svelte`), sizes persisted.

Dev tip: append `?demo` to the browser dev URL to open a sample project without
the backend.

### M1 (in progress): real Typst compilation

`typide-world` embeds the **Typst 0.15.1 compiler**: a `World` over the project
(embedded + system fonts, offline package resolution — no network), and
`compile(root, entrypoint, overlays) -> CompileResult` producing **per-page SVG**
and located diagnostics. `overlays` carry unsaved editor buffers, so the preview
compiles the **current buffer without writing to disk** — no autosave, no
file-watcher churn. Compile runs on open, debounced ~400 ms after typing (live),
and on Ctrl+S / Run (which also saves). It is wrapped in `catch_unwind` so a
compiler panic can never crash the window. The frontend renders the SVG in the
Preview and merges compiler diagnostics into Problems. Verified API facts are in
`docs/verified-facts.md`.

The dev CSP allows Tauri IPC, the Vite dev server + HMR socket, `data:`/`asset:`
images (for embedded SVG resources) and Google Fonts — earlier it blocked IPC
and HMR (`app/src-tauri/tauri.conf.json`).

### Packages & templates (network isolated in `typide-net`)

`typide-net::obtain_package` downloads a Universe package tarball into the shared
Typst cache (the only place HTTP happens — P1). `typide-packages` orchestrates:
`fetch_all(root)` scans the project (and packages it pulls in, transitively) for
`@preview/…` specs and obtains each; `init_template(spec, dest)` downloads a
template package, copies its `[template]` files into the project and returns the
entrypoint. Backend commands: `packages_fetch`, and `workspace_create` now
instantiates the wizard's chosen template. The frontend auto-fetches missing
packages on compile when the network mode isn't `offline`, and the Packages panel
has a manual "Fetch / make offline-ready" button. The compiler itself stays fully
offline; it only ever reads from the cache these steps populate.

### M2 (in progress): packages, fonts, offline, network policy

- **Live package browser** — `packages_search(query, online)` searches the Typst
  Universe index (`typide_packages::search`: fetches `index.json` via
  `typide-net`, caches it, keeps the latest version per name). UI:
  `PackageBrowser.svelte` (Cmd-palette "Browse Universe" or the Packages panel),
  one-click **Add** inserts the import into the editor and fetches the package.
- **Make offline-ready** — `packages_make_offline_ready` resolves everything
  (transitively), copies it into `vendor/packages/`, hashes each package
  (SHA-256) and writes `typide.lock`. Wired to the Packages panel.
- **Fonts panel** — `fonts_list` (`typide_world::list_fonts`) returns the real
  embedded+system font families; `FontsPanel.svelte` renders each in its own face.
- **Network policy** — every network call takes an `online` flag derived from
  `ui.networkMode` (`online()` in the store); offline fails closed in
  `typide_net::obtain_package`. `NetworkMode::Ask` currently behaves as online
  (no consent prompt yet — a remaining M2 item).

- **PDF export** — `typide_world::export_pdf` (via `typst-pdf`); the `export_pdf`
  command writes the profile's output path and the Run button / command palette
  run it, with a toast on success/failure (`Toast.svelte`).
- **Editor intelligence via `typst-ide`** (no subprocess): `complete` and `hover`
  commands back a CodeMirror `autocompletion` source and `hoverTooltip`
  (`editor.ts`, bridged through `completeAt`/`hoverAt` in the store). Offsets are
  converted UTF-16 ↔ UTF-8 in `typide-world`. A lighter embedded-fonts-only world
  (`TypideWorld::new_lite`) keeps these interactive queries fast.
- **Spell checking** — `typide-check` extracts prose from markup `Text` nodes with
  `typst-syntax` (never code/math/labels), checks a bundled ~10k-word frequency
  list, and ranks suggestions with **Damerau-Levenshtein** (`spellcheck` command).
  The editor underlines misspellings and shows suggestions + "Add to dictionary"
  on hover (per-viewer dictionary in `localStorage`). Backend offsets are UTF-16.
- **Source↔preview jump (two-way)** — backward: `jump_from_click` (typst-ide)
  maps a preview click (page + point in pt) back to a file + caret offset.
  Forward: `jump_from_cursor` (typst-ide) maps the editor caret (UTF-16 offset)
  to a `PreviewPosition {page,x,y}`; the editor's `onCursor` debounces into
  `syncPreviewToCursor`, which scrolls the Preview to that page/point and pulses
  a highlight bar. A header toggle (`ui.syncScroll`) turns forward sync off; a
  short post-click suppression keeps the two directions from fighting.
- **Preview UX** — **fit-to-width by default**, plus Fit-page / Actual-size /
  50–200% via a zoom-mode menu; per-page vector SVG (crisp: `shape-rendering`/
  `text-rendering: geometricPrecision`, no compositing clip) with page numbers;
  optional dark-invert (a CSS `filter`, so it can soften text on HiDPI — the
  default non-inverted view is fully vector-crisp).

- **Incremental compile** — a per-project `World` is cached (`with_world` in
  `typide-world`); between compiles overlays are swapped and `FileStore::reset()`
  is called so `comemo` reuses unchanged layout, and the font DB loads once.
  compile/complete/hover/jump all share the cached world (panic-guarded).
- **Background jobs** — `JobRegistry` + Tauri events (`job://started|progress|
  finished`); `packages_fetch`, `packages_make_offline_ready` and
  `toolchain_install` are async jobs with live progress + `job_cancel`. Frontend
  wires events in `store.initJobs()`; the Jobs panel shows live bars + cancel.
  `typide_packages` reports via a `ProgressSink`.
- **Toolchain manager** — `typide-toolchain` lists bundled/system/installed Typst
  versions, `list_available` reads GitHub releases (via `typide-net`), and
  `install` downloads + extracts (`tar.xz` via `lzma-rs`) into the data dir. UI:
  `ToolchainManager.svelte` (command palette → "Toolchain: Manage versions…").

- **Network consent ("Ask each time")** — `requestConsent(host, reason)` in the
  store gates every network op: Offline → denied; Online → allowed; Ask → a
  `ConsentDialog.svelte` prompt (Deny / Allow once / Allow this session). All
  download paths (package fetch, make-offline-ready, package-index, toolchain
  install/list) go through it; the compiler stays offline regardless.
- **Toolchain install from archive** — `typide_toolchain::install_from_archive`
  extracts a local `.tar.xz`, probes the version via `typst --version`, and
  installs it (air-gapped, §9.4). UI: "Install from file…" in the manager.
- **Institutional policy** — `typide-policy` reads `policy.toml` from the system
  location (or `TYPIDE_POLICY_PATH`); `policy_get` returns the effective policy;
  an enforced `network.mode` locks the network controls in first-run, the launcher
  and the title bar ("managed by your organization"). Sample: `docs/policy.example.toml`.

- **Mirror creation** — `typide_packages::create_mirror` resolves the project's
  packages (transitively) and writes a Universe-layout mirror
  (`out/preview/{name}-{version}.tar.gz` + filtered `index.json`) to a chosen
  folder — a cancellable job (`packages_mirror_create`). UI: "Create mirror…" in
  the Packages panel. Good for USB / intranet distribution (§10.5).

**M2 is functionally complete.** Deferred to later milestones: the real Tinymist
LSP *subprocess* bridge (§4/§17 revisit in M5 — our in-process `typst-ide` covers
completion/hover/diagnostics meanwhile); tinymist + Windows `.zip` toolchain
installs; mirror *sync*; mid-download install cancellation; and the policy
`updates`/`ai` keys (their features are M6).

### M3 (in progress): academic workflow

- **Snapshots + timeline + restore + remote sync** — `typide-vcs` wraps the
  `git` CLI (arg arrays, never a shell — §20) with a fallback identity. The Git
  panel: a message box **snapshots** (saving dirty buffers first), the timeline
  shows real history, **Restore** checks a commit's files back out *and refreshes
  the live editor* + recompiles, and **Sync** (fetch → merge → push, a job gated
  by consent) works with any git remote. `vcs_snapshot/timeline/restore/
  get_remote/set_remote/sync`.
- **Citation picker** — `CitationPicker.svelte` (Alt+C / command palette) fuzzy-
  searches the project's `.bib` citations and inserts `@key` (or
  `#cite(<key>)` with Shift+↵) at the caret via `insertAtCursor`.
- **Zotero import** — `typide-refs` reads a local Zotero library over Better
  BibTeX (`refs_zotero_status` / `refs_zotero_import` → writes `zotero.bib`,
  rescans so citations appear). Localhost HTTP via `typide-net`, consent-gated.
  **[VERIFY]**: the endpoints/port aren't verified against a running Zotero here.
- **Submission export** — `export_pdf` takes PDF standards; profiles "Submission
  PDF/A-2b" (`a-2b`) and "Submission PDF/A + UA" (`a-2b`,`ua-1`). `typide-world`
  sets a document timestamp for archival and `tagged` for UA.

**M3 is functionally complete.** The notes vault was intentionally dropped
(note-taking = writing Typst with comments). Hayagriva `.yml` references and the
submission *checker report* (§15.3) are the remaining academic-workflow polish.

### Production hardening (toward 1.0)

- **Multi-OS release** — GitHub Actions builds signed-ready installers on tag
  push: macOS universal `.dmg`, Windows `.msi`/`.exe`, Linux
  `.AppImage`/`.deb`/`.rpm` (`.github/workflows/release.yml`), plus a `ci.yml`
  (fmt + clippy + check-deps + workspace tests + frontend). The frontend is
  built in a dedicated step and the bundle runs with `--config ci.conf.json`
  (empty `beforeBuildCommand`) because `tauri-action` mis-resolved the relative
  `../ui` prefix on the runners.
- **`~` path expansion** — `expand_tilde()` resolves a leading `~`/`~/`/`~\` in
  `workspace_open`/`workspace_create`, so a project is never created in a literal
  `~` folder; the wizard also resolves its default to the real home path.
- **Unsaved-work safety** — dirty buffers are mirrored every 4 s to a crash-
  recovery store in the app-data dir (NOT the project), via `recovery_save/
  scan/read/discard`; `RecoveryBanner.svelte` offers restore-or-discard on
  reopen when a draft is newer than disk. A window **close guard**
  (`onCloseRequested` → `CloseGuard.svelte`) prompts Save/Discard/Cancel when
  buffers are dirty (needs `core:window:allow-destroy`).
- **Production CSP** — `security.csp` is tightened for release (no dev-server /
  `ws://localhost` / HMR origins); the permissive dev policy moves to
  `security.devCsp`.

Offline-first is already satisfied without `resources/`: fonts are embedded in
the compiler (`typst-kit` `embedded-fonts`), the built-in generators scaffold
offline, and compile/preview/export use the embedded compiler (no `typst`
subprocess). See `resources/README.md`.

### Dev note — window reloads on save under `cargo tauri dev`

`cargo tauri dev` runs a file watcher that rebuilds and relaunches the window
when files in the watched tree change. If a **project is created/opened inside
this repo**, saving a file there triggers that relaunch (it looks like the window
closing and reopening). Two fixes: keep projects **outside** the repo (the wizard
defaults to `~/research`), and/or run `cargo tauri dev --no-watch`. A release
build (`cargo tauri build`) has no watcher and never does this.

Remaining M1: run compile as a cancellable `typide-jobs` job (currently a
blocking command), incremental compile (keep a long-lived `World`, reset the
`FileStore` instead of rebuilding), source↔preview jump, and PDF export.
Later: move the regex analysis in `lib/analysis.ts` into a Rust index built with
`typst-syntax`.
