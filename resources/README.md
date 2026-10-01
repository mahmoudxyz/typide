# `resources/` — optional drop-in assets

These folders are **optional**. Typide works fully offline without them:

- **Fonts** — the embedded Typst compiler (`typide-world`) ships Typst's default
  font set compiled into the binary (`typst-kit`'s `embedded-fonts`), plus any
  fonts installed on the system. Documents using the default fonts compile and
  export to PDF with **no network and no files here**. Drop extra `.ttf`/`.otf`
  files in `fonts/` only if you want to vendor institution-specific typefaces.
- **Templates** — the built-in project generators (thesis, paper, notes,
  generic) are scaffolded in Rust and need no network. `templates/` is for
  bundling additional institutional template directories. "New from a Universe
  template" downloads on demand and is the only template path that uses the
  network.
- **Toolchains** — the core compile/preview/export use the **embedded** Typst
  compiler, not an external `typst` binary, so no toolchain file is required.
  `toolchains/` is where the toolchain manager can place additional pinned Typst
  versions for projects that request a specific one.

Nothing in this directory is read automatically at runtime yet, and it is not
part of the Tauri bundle. It is a staging area for air-gapped / institutional
distributions (ARCHITECTURE.md §10.5).
