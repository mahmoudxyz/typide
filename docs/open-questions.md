# Open questions

Seeded from ARCHITECTURE.md §22. Add a `// DESIGN-QUESTION:` comment in code and
an entry here rather than improvising when a task requires breaking a §2 principle.

1. Link Tinymist crates directly vs. subprocess only? (Decide in M5.)
2. How much style-chain introspection is available via public Typst APIs?
3. Does Typst Universe publish tarball checksums? (Meanwhile TOFU + lockfile.)
4. How to present experimental HTML/bundle export in the UI?
5. Visual PDF diff: `typst-render` (native only) vs. a PDF rasterizer for any version?
6. Markdown notes: Typst-based renderer vs. JS renderer?
7. Naming & branding: avoid implying official affiliation with Typst GmbH.
8. **[VERIFY] Zotero local API** — `typide-refs` uses port 23119 and the Better
   BibTeX endpoints `/better-bibtex/cayw?probe=probe` (status) and
   `/better-bibtex/export/library?/1/biblatex` (export). These were not verified
   against a running Zotero in this environment; confirm the port, library id,
   and format path, and add a Zotero-7 built-in API fallback when BBT is absent.
