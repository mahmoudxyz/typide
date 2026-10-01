# Verified facts

Findings for `[VERIFY]` items, checked against upstream source. Format: fact ·
source · date checked.

## Typst 0.15.1 crate & compiler API (checked 2026-09-30)

- Crate names/versions all publish at `0.15.1`: `typst`, `typst-kit`,
  `typst-svg`, `typst-render`, `typst-syntax`, `typst-pdf`, `typst-layout`,
  `typst-library`. · crates.io
- **`World` trait** (`typst_library::World`, re-exported as `typst::World`):
  `library() -> &LazyHash<Library>`, `book() -> &LazyHash<FontBook>`,
  `main() -> FileId`, `source(FileId) -> FileResult<Source>`,
  `file(FileId) -> FileResult<Bytes>`, `font(usize) -> Option<Font>`,
  `today(Option<Duration>) -> Option<Datetime>`. · typst-library 0.15.1 src/lib.rs
- **Compile:** `typst::compile::<PagedDocument>(&world) -> Warned<SourceResult<T>>`
  where `T: Output`. `PagedDocument` lives in `typst_layout`; `.pages() -> &[Page]`.
  · typst 0.15.1 src/lib.rs, typst-layout src/document.rs
- **SVG export:** `typst_svg::svg(&Page, &SvgOptions) -> String`;
  `SvgOptions: Default`. Also `svg_merged(&PagedDocument, &SvgOptions, gap)`.
  · typst-svg 0.15.1 src/lib.rs
- **FileId construction (reworked in 0.15):**
  `FileId::new(RootedPath::new(VirtualRoot::Project, VirtualPath::new(rel)?))`.
  `VirtualRoot` is an enum `{ Project, Package(PackageSpec) }`.
  · typst-syntax 0.15.1 src/path.rs
- **Library:** `Library::builder().build()` — `builder()` requires
  `use typst::LibraryExt`. · typst-library 0.15.1
- **Diagnostics:** `SourceDiagnostic { severity, span: DiagSpan, message, hints, .. }`.
  `DiagSpan::id() -> Option<FileId>`; `WorldExt::range(span) -> Option<Range<usize>>`;
  `Source::lines().byte_to_line_column(byte) -> Option<(usize, usize)>` (0-based).
  · typst-syntax 0.15.1 span.rs/lines.rs
- **typst-kit 0.15** was reworked into composable providers (confirms the
  architecture note): `files::{FileStore<L>, FileLoader, SystemFiles, FsRoot}`,
  `fonts::{FontStore, embedded(), system(), scan()}`,
  `packages::{SystemPackages, FsPackages}`, `datetime::Time`,
  `downloader::Downloader`. Feature flags: `embedded-fonts`, `scan-fonts`,
  `system-files`, `system-packages`, `universe-packages`, `datetime`,
  `system-downloader` (network — intentionally NOT enabled in `typide-world`).
  · typst-kit 0.15.1 src/*
- **Offline compile:** `SystemPackages::new(downloader)` requires a `Downloader`;
  a no-op impl whose `stream()` errors keeps compilation fully offline while
  still resolving already-cached packages from disk. · typide-world/src/lib.rs

## Package download & templates (checked 2026-09-30)

- Universe index reachable at `https://packages.typst.org/preview/index.json`
  (HTTP 200). · live
- `PackageSpec: FromStr` parses `@namespace/name:version`; `Display` writes the
  same. Fields: `namespace`, `name`, `version`. · typst-syntax 0.15.1 package.rs
- Downloading: `typst_kit::packages::SystemPackages::new(SystemDownloader::new(ua))`
  then `.obtain(&spec) -> PackageResult<FsRoot>` — checks data/cache first, else
  downloads from Universe and extracts into the shared cache; returns the dir.
  `SystemDownloader` needs the `system-downloader` feature (uses `ureq` +
  `native-tls`; `libssl-dev` on Linux). Verified with a real fetch of
  `@preview/oxifmt:0.2.1`. · typide-net/src/lib.rs
- Templates: a package's `typst.toml` has `[template] { path, entrypoint }`;
  copying `<pkg>/<path>/*` into a new dir yields a working project whose
  entrypoint is `<entrypoint>`. Verified end-to-end (download → instantiate →
  compile) with `@preview/charged-ieee:0.1.3`. · typide-packages/src/lib.rs

## PDF export & editor intelligence (checked 2026-09-30)

- PDF: `typst_pdf::pdf(&PagedDocument, &PdfOptions) -> SourceResult<Vec<u8>>`;
  `PdfOptions: Default` (fields: ident, creator, timestamp, page_ranges,
  standards, tagged, pretty). Verified output begins `%PDF`. · typst-pdf 0.15.1
- Completion: `typst_ide::autocomplete(world: &dyn IdeWorld, output:
  Option<impl AsOutput>, source: &Source, cursor: usize (byte), explicit: bool)
  -> Option<(usize from_byte, Vec<Completion>)>`. `Completion { kind:
  CompletionKind, label, apply: Option<..>, detail: Option<..> }`. Verified it
  suggests `figure` for `#figur`. · typst-ide 0.15.1 complete.rs
- Hover: `typst_ide::tooltip(world, output, source, cursor, Side) ->
  Option<Tooltip>` where `Tooltip = Text(..) | Code(..)`. · tooltip.rs
- `IdeWorld: World` needs only `fn upcast(&self) -> &dyn World` (packages/files
  have defaults). `impl<T: Output> AsOutput for &T`, so pass
  `None::<&PagedDocument>` when no compiled doc is available. · typst-ide lib.rs
- Cursor offsets: typst-ide uses **UTF-8 byte** offsets; CodeMirror uses UTF-16.
  `typide-world` converts both ways against the `Source` text. · typide-world

## PDF standards (checked 2026-10-01)

- `PdfStandard` (typst-pdf 0.15.1) serde ids: `a-1b` `a-1a` `a-2b` `a-2u` `a-2a`
  `a-3b` `a-3u` `a-3a` `a-4` `a-4f` `a-4e` `ua-1`. `PdfStandards::new(&[..])`
  validates combinations (at most one PDF/UA). PDF/A requires a document date —
  set `PdfOptions.timestamp` (`Timestamp::new_utc(world.today(None))`) when none
  is in the source; PDF/UA needs `PdfOptions.tagged = true`. Verified a `a-2b`
  export produces a `%PDF`. · typst-pdf 0.15.1, typide-world

Still open: Tinymist config keys (§12.1),
Zotero local API port/paths (§15.2), package tarball checksums (§10.3).
