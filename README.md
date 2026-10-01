# Typide

[![CI](https://github.com/mahmoudxyz/typide/actions/workflows/ci.yml/badge.svg)](https://github.com/mahmoudxyz/typide/actions/workflows/ci.yml)
[![Release](https://github.com/mahmoudxyz/typide/actions/workflows/release.yml/badge.svg)](https://github.com/mahmoudxyz/typide/actions/workflows/release.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

A free, open-source, **offline-first IDE for Typst** — in the spirit of
IntelliJ, aimed at academic writing (theses, papers, research notes).

See [`ARCHITECTURE.md`](ARCHITECTURE.md) for the full design (the source of
truth) and [`CLAUDE.md`](CLAUDE.md) for contributor conventions.

> **Status:** M0–M3 implemented — real Typst 0.15.1 compilation with live SVG
> preview, editor intelligence (completion/hover/spell-check), packages &
> templates, fonts, toolchain manager, network policy + consent, PDF/A+UA
> submission export, Git snapshots/timeline/restore/sync, and a citation picker
> with Zotero import. Built on a 16-crate Rust workspace with network I/O
> isolated to `typide-net` (enforced by `cargo xtask check-deps`).

## Download

Grab the installer for your platform from the
[**Releases**](https://github.com/mahmoudxyz/typide/releases/latest) page:

| OS | File |
|----|------|
| **Windows** | `Typide_<version>_x64_en-US.msi` or `Typide_<version>_x64-setup.exe` |
| **macOS** (Apple Silicon + Intel) | `Typide_<version>_universal.dmg` |
| **Linux** | `Typide_<version>_amd64.AppImage` (portable), `.deb` or `.rpm` |

Releases are produced automatically by the [release workflow](.github/workflows/release.yml)
when a `v*` tag is pushed. The apps are not yet code-signed, so your OS may warn
on first launch (right-click → Open on macOS; "More info → Run anyway" on
Windows).

## Run as a native desktop app

Typide is a **desktop application** (Tauri v2 — native window, system WebView,
small binary), not a web app. The browser dev server is only a convenience.

On Ubuntu/Debian, install the WebView build dependencies once:

```bash
sudo apt update && sudo apt install -y \
  libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev librsvg2-dev libssl-dev \
  libayatana-appindicator3-dev build-essential curl wget file
```

Then, from the repo root:

```bash
cargo tauri dev      # launches the native window (builds the frontend for you)
cargo tauri build    # produces .deb / .rpm / .AppImage installers
```

## Layout

```
crates/          16 library crates (typide-core, -world, -net, -lsp, …)
xtask/           repo automation (check-deps, ci)
app/src-tauri/   Tauri v2 desktop shell + IPC commands (§18.2)
app/ui/          Svelte 5 + Vite + CodeMirror 6 frontend
docs/            ADRs, open questions, verified facts
tests/fixtures/  sample projects (thesis-basic, …)
```

## Quick start

### Backend (pure-Rust crates)

```bash
cargo test --workspace      # all crate unit tests
cargo xtask check-deps      # enforce: network I/O only in typide-net
cargo xtask ci              # fmt + clippy + test + check-deps
```

### Frontend (runs in a plain browser with mock data)

```bash
cd app/ui
npm install
npm run dev                 # http://localhost:1420
```

The UI detects Tauri at runtime; without it, commands resolve against mock data
(`src/lib/mockData.ts`) so the whole interface is explorable standalone.

### Full desktop app

Requires the platform WebView libraries (Linux: `webkit2gtk-4.1`, `libsoup-3.0`).
`app/src-tauri` is intentionally excluded from the default Cargo workspace for
this reason.

```bash
cargo tauri dev             # or: cargo tauri build
```

## Principles (hard rules — see §2)

Offline-first · no telemetry · plain files, no lock-in · upstream-first ·
reproducible document builds · admin-controllable · responsive UI.

## License

Apache-2.0.
