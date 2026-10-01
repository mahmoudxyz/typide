//! The embedded Typst compiler: a [`World`] over a project on disk, plus
//! [`compile`] which produces per-page SVG and diagnostics.
//!
//! Fonts come from Typst's embedded set plus the system (offline). Packages are
//! resolved from the shared Typst cache/data directories; downloading is
//! **disabled** here — the network belongs to `typide-net` (ARCHITECTURE.md P1).
#![deny(missing_docs)]

use std::any::Any;
use std::collections::HashMap;
use std::io::{self, Read};
use std::panic::AssertUnwindSafe;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

use serde::{Deserialize, Serialize};

use typst::diag::{FileResult, Severity, SourceDiagnostic, Warned};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::World;
use typst::{Library, LibraryExt, WorldExt};

use typst_kit::datetime::Time;
use typst_kit::downloader::Downloader;
use typst_kit::files::{FileLoader, FileStore, FsRoot, SystemFiles};
use typst_kit::fonts::{self, FontStore};
use typst_kit::packages::SystemPackages;

use typst::syntax::Side;
use typst_ide::{CompletionKind, Tooltip};
use typst_layout::PagedDocument;
use typst_pdf::{PdfOptions, PdfStandard, PdfStandards, Timestamp};
use typst_svg::SvgOptions;

/// A compiler diagnostic mapped to a file location.
#[derive(Debug, Clone, Serialize)]
pub struct Diagnostic {
    /// `error`, `warning`, or `hint`.
    pub severity: String,
    /// Path of the file, relative to its root (e.g. `main.typ`). Empty if detached.
    pub file: String,
    /// One-based line number (0 if unknown).
    pub line: usize,
    /// One-based column (0 if unknown).
    pub col: usize,
    /// Human-readable message.
    pub message: String,
    /// Extra hints attached to the diagnostic.
    pub hints: Vec<String>,
}

/// The result of compiling a project.
#[derive(Debug, Clone, Serialize)]
pub struct CompileResult {
    /// One SVG string per rendered page (empty if compilation failed).
    pub pages: Vec<String>,
    /// Errors and warnings produced during compilation.
    pub diagnostics: Vec<Diagnostic>,
    /// Number of pages produced.
    pub page_count: usize,
    /// Wall-clock compile time in milliseconds.
    pub compile_ms: u64,
}

/// An unsaved editor buffer to use in place of a file on disk.
#[derive(Debug, Clone, Deserialize)]
pub struct Overlay {
    /// Path relative to the project root, e.g. `chapters/01.typ`.
    pub path: String,
    /// The current (unsaved) content.
    pub content: String,
}

/// A [`Downloader`] that always refuses: keeps compilation fully offline.
struct OfflineDownloader;

impl Downloader for OfflineDownloader {
    fn stream(&self, _key: &dyn Any, _url: &str) -> io::Result<(Option<usize>, Box<dyn Read>)> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "network is disabled during compile (offline-first)",
        ))
    }
}

/// A [`FileLoader`] that serves unsaved editor buffers, falling back to disk.
///
/// This lets the preview compile the *current* editor content without writing
/// to disk first (ARCHITECTURE.md §13.1 — nothing on the UI path touches the
/// project files).
type Overrides = Arc<Mutex<HashMap<FileId, Bytes>>>;

struct OverlayLoader {
    inner: SystemFiles,
    overrides: Overrides,
}

impl FileLoader for OverlayLoader {
    fn load(&self, id: FileId) -> FileResult<Bytes> {
        if let Some(bytes) = self.overrides.lock().unwrap().get(&id) {
            return Ok(bytes.clone());
        }
        self.inner.load(id)
    }
}

fn build_overrides(overlays: &[Overlay]) -> HashMap<FileId, Bytes> {
    let mut map = HashMap::new();
    for o in overlays {
        if let Ok(vp) = VirtualPath::new(&o.path) {
            let id = FileId::new(RootedPath::new(VirtualRoot::Project, vp));
            map.insert(id, Bytes::new(o.content.clone().into_bytes()));
        }
    }
    map
}

/// A Typst [`World`] backed by a project directory plus mutable overlays.
///
/// A world is kept alive per project (see [`with_world`]); between compiles its
/// overlays are swapped and the `FileStore` is reset, so `comemo` reuses the
/// unchanged layout — incremental compilation (ARCHITECTURE.md §13.1).
pub struct TypideWorld {
    library: LazyHash<Library>,
    fonts: FontStore,
    files: FileStore<OverlayLoader>,
    overrides: Overrides,
    main: FileId,
    time: Time,
}

impl TypideWorld {
    /// Build a world for `root` with `entrypoint` (relative to the root) as main.
    /// `overlays` replace on-disk files with unsaved editor content.
    pub fn new(root: &Path, entrypoint: &str, overlays: &[Overlay]) -> Result<Self, String> {
        Self::build(root, entrypoint, overlays, false)
    }

    /// Like [`new`](Self::new) but with embedded fonts only — faster to build,
    /// for interactive IDE queries (completion, hover) where system font
    /// discovery isn't needed.
    pub fn new_lite(root: &Path, entrypoint: &str, overlays: &[Overlay]) -> Result<Self, String> {
        Self::build(root, entrypoint, overlays, true)
    }

    fn build(
        root: &Path,
        entrypoint: &str,
        overlays: &[Overlay],
        lite: bool,
    ) -> Result<Self, String> {
        let vpath = VirtualPath::new(entrypoint).map_err(|e| format!("bad entrypoint: {e}"))?;
        let main = FileId::new(RootedPath::new(VirtualRoot::Project, vpath));

        let mut fonts = FontStore::new();
        fonts.extend(fonts::embedded());
        if !lite {
            fonts.extend(fonts::system());
            // User font folders + TYPST_FONT_PATHS + this project's `fonts/`.
            for dir in extra_font_dirs(Some(root)) {
                if dir.is_dir() {
                    fonts.extend(fonts::scan(&dir));
                }
            }
        }

        let overrides: Overrides = Arc::new(Mutex::new(build_overrides(overlays)));

        let project = FsRoot::new(root.to_path_buf());
        let packages = SystemPackages::new(OfflineDownloader);
        let loader = OverlayLoader {
            inner: SystemFiles::new(project, packages),
            overrides: overrides.clone(),
        };
        let files = FileStore::new(loader);

        Ok(Self {
            library: LazyHash::new(Library::builder().build()),
            fonts,
            files,
            overrides,
            main,
            time: Time::system(),
        })
    }

    /// Swap the overlay buffers and mark files stale so the next compile reuses
    /// unchanged layout (incremental compilation).
    pub fn set_overlays(&mut self, overlays: &[Overlay]) {
        *self.overrides.lock().unwrap() = build_overrides(overlays);
        self.files.reset();
    }
}

// ── Per-project world cache (incremental compilation) ────────────────────────

struct Cached {
    root: PathBuf,
    entrypoint: String,
    world: TypideWorld,
}

fn world_cache() -> &'static Mutex<Option<Cached>> {
    static C: OnceLock<Mutex<Option<Cached>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

/// Drop the cached world so the next compile rebuilds it (e.g. after the font
/// set changes).
fn clear_world_cache() {
    *world_cache().lock().unwrap_or_else(|p| p.into_inner()) = None;
}

/// User-configured extra font directories (scanned in addition to embedded +
/// system + each project's own `fonts/`).
fn font_dirs() -> &'static Mutex<Vec<PathBuf>> {
    static D: OnceLock<Mutex<Vec<PathBuf>>> = OnceLock::new();
    D.get_or_init(|| Mutex::new(Vec::new()))
}

/// Replace the user's extra font directories and invalidate the cached world so
/// the next compile/list picks up the new fonts.
pub fn set_font_dirs(dirs: Vec<PathBuf>) {
    *font_dirs().lock().unwrap_or_else(|p| p.into_inner()) = dirs;
    clear_world_cache();
}

/// Rescan fonts on the next compile/list (e.g. after dropping a file into a
/// watched directory). Cheap: just drops the cached world.
pub fn rescan_fonts() {
    clear_world_cache();
}

/// All extra font directories to scan: user dirs + `TYPST_FONT_PATHS` + the
/// project's own `fonts/` folder (when a root is known). Non-existent paths are
/// kept here and filtered at scan time.
fn extra_font_dirs(root: Option<&Path>) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = font_dirs().lock().map(|g| g.clone()).unwrap_or_default();
    if let Ok(paths) = std::env::var("TYPST_FONT_PATHS") {
        dirs.extend(std::env::split_paths(&paths));
    }
    if let Some(r) = root {
        dirs.push(r.join("fonts"));
    }
    dirs
}

/// Run `f` against the cached world for `(root, entrypoint)`, rebuilding it only
/// when the project changes. Overlays are applied first. Panics in `f` are
/// caught (the window never crashes) and drop the cached world.
fn with_world<T>(
    root: &Path,
    entrypoint: &str,
    overlays: &[Overlay],
    f: impl FnOnce(&TypideWorld) -> T,
) -> Result<T, String> {
    let mut guard = world_cache().lock().unwrap_or_else(|p| p.into_inner());
    let stale = guard
        .as_ref()
        .is_none_or(|c| c.root != root || c.entrypoint != entrypoint);
    if stale {
        let world = TypideWorld::new(root, entrypoint, &[])?;
        *guard = Some(Cached {
            root: root.to_path_buf(),
            entrypoint: entrypoint.to_string(),
            world,
        });
    }
    let cached = guard.as_mut().unwrap();
    cached.world.set_overlays(overlays);
    match std::panic::catch_unwind(AssertUnwindSafe(|| f(&cached.world))) {
        Ok(value) => Ok(value),
        Err(_) => {
            *guard = None; // rebuild fresh next time
            Err("internal error (panic) during compilation".to_string())
        }
    }
}

impl typst_ide::IdeWorld for TypideWorld {
    fn upcast(&self) -> &dyn World {
        self
    }
}

impl World for TypideWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }
    fn main(&self) -> FileId {
        self.main
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        self.files.source(id)
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files.file(id)
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }
    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        self.time.today(offset)
    }
}

fn locate(world: &TypideWorld, d: &SourceDiagnostic) -> (String, usize, usize) {
    let Some(id) = d.span.id() else {
        return (String::new(), 0, 0);
    };
    let file = id.vpath().get_without_slash().replace('\\', "/");
    if let (Ok(source), Some(range)) = (world.source(id), world.range(d.span)) {
        if let Some((line, col)) = source.lines().byte_to_line_column(range.start) {
            return (file, line + 1, col + 1);
        }
    }
    (file, 1, 1)
}

fn to_diagnostic(world: &TypideWorld, d: &SourceDiagnostic) -> Diagnostic {
    let (file, line, col) = locate(world, d);
    Diagnostic {
        severity: match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
        .to_string(),
        file,
        line,
        col,
        message: d.message.to_string(),
        hints: d.hints.iter().map(|h| h.v.to_string()).collect(),
    }
}

/// A font family available to the compiler.
#[derive(Debug, Clone, Serialize)]
pub struct FontFamily {
    /// Family name, e.g. "New Computer Modern".
    pub name: String,
    /// Number of variants (weights/styles) present.
    pub variants: usize,
    /// Distinct style names present (Regular, Italic, …).
    pub styles: Vec<String>,
    /// Whether any variant is a variable font.
    pub variable: bool,
}

/// List every font family available: embedded + system + the user's extra font
/// dirs + the given project's `fonts/` folder (when `root` is provided).
pub fn list_fonts(root: Option<&Path>) -> Vec<FontFamily> {
    use std::collections::BTreeSet;
    let mut fonts = FontStore::new();
    fonts.extend(fonts::embedded());
    fonts.extend(fonts::system());
    for dir in extra_font_dirs(root) {
        if dir.is_dir() {
            fonts.extend(fonts::scan(&dir));
        }
    }
    let book = fonts.book();

    let mut families: Vec<FontFamily> = Vec::new();
    for (name, indices) in book.families() {
        let mut count = 0usize;
        let mut styles: BTreeSet<String> = BTreeSet::new();
        let mut variable = false;
        for idx in indices {
            if let Some(info) = book.info(idx) {
                count += 1;
                styles.insert(
                    match info.variant.style {
                        typst::text::FontStyle::Normal => "Regular",
                        typst::text::FontStyle::Italic => "Italic",
                        typst::text::FontStyle::Oblique => "Oblique",
                    }
                    .to_string(),
                );
                if !info.axes.is_empty() {
                    variable = true;
                }
            }
        }
        families.push(FontFamily {
            name: name.to_string(),
            variants: count,
            styles: styles.into_iter().collect(),
            variable,
        });
    }
    families.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    families
}

/// Compile `entrypoint` under `root`, returning per-page SVG and diagnostics.
///
/// `overlays` supply unsaved editor content so the preview reflects the current
/// buffer without touching disk. Always returns a value: on failure `pages` is
/// empty and `diagnostics` carries the errors.
pub fn compile(root: &Path, entrypoint: &str, overlays: &[Overlay]) -> CompileResult {
    let start = Instant::now();

    let err_result = |message: String| CompileResult {
        pages: vec![],
        diagnostics: vec![Diagnostic {
            severity: "error".into(),
            file: entrypoint.to_string(),
            line: 0,
            col: 0,
            message,
            hints: vec![],
        }],
        page_count: 0,
        compile_ms: start.elapsed().as_millis() as u64,
    };

    let outcome = with_world(root, entrypoint, overlays, |world| {
        let Warned { output, warnings } = typst::compile::<PagedDocument>(world);
        let mut diagnostics: Vec<Diagnostic> = Vec::new();
        let mut pages: Vec<String> = Vec::new();
        let mut page_count = 0;

        match output {
            Ok(doc) => {
                let opts = SvgOptions::default();
                page_count = doc.pages().len();
                for page in doc.pages() {
                    pages.push(typst_svg::svg(page, &opts));
                }
            }
            Err(errors) => {
                diagnostics.extend(errors.iter().map(|e| to_diagnostic(world, e)));
            }
        }
        diagnostics.extend(warnings.iter().map(|w| to_diagnostic(world, w)));
        (pages, diagnostics, page_count)
    });

    // Bound comemo's cache but keep recent entries so recompiles are incremental.
    comemo::evict(30);

    let (pages, diagnostics, page_count) = match outcome {
        Ok(t) => t,
        Err(message) => return err_result(message),
    };

    CompileResult {
        pages,
        diagnostics,
        page_count,
        compile_ms: start.elapsed().as_millis() as u64,
    }
}

// ── PDF export ───────────────────────────────────────────────────────────────

fn parse_standard(s: &str) -> Option<PdfStandard> {
    Some(match s.to_lowercase().as_str() {
        "a-1b" => PdfStandard::A_1b,
        "a-1a" => PdfStandard::A_1a,
        "a-2b" => PdfStandard::A_2b,
        "a-2u" => PdfStandard::A_2u,
        "a-2a" => PdfStandard::A_2a,
        "a-3b" => PdfStandard::A_3b,
        "a-3u" => PdfStandard::A_3u,
        "a-3a" => PdfStandard::A_3a,
        "a-4" => PdfStandard::A_4,
        "a-4f" => PdfStandard::A_4f,
        "a-4e" => PdfStandard::A_4e,
        "ua-1" => PdfStandard::Ua_1,
        _ => return None,
    })
}

fn one_err(entrypoint: &str, message: String) -> Vec<Diagnostic> {
    vec![Diagnostic {
        severity: "error".into(),
        file: entrypoint.to_string(),
        line: 0,
        col: 0,
        message,
        hints: vec![],
    }]
}

/// Compile `entrypoint` (from disk) and export to a PDF byte vector, applying
/// any PDF standards (e.g. `["a-2b", "ua-1"]`). On failure returns diagnostics.
pub fn export_pdf(
    root: &Path,
    entrypoint: &str,
    standards: &[String],
) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let world = TypideWorld::new(root, entrypoint, &[]).map_err(|m| one_err(entrypoint, m))?;

    let has_ua = standards.iter().any(|s| s.eq_ignore_ascii_case("ua-1"));
    let parsed: Vec<PdfStandard> = standards.iter().filter_map(|s| parse_standard(s)).collect();
    let pdf_standards = PdfStandards::new(&parsed)
        .map_err(|e| one_err(entrypoint, format!("invalid PDF standards: {e:?}")))?;

    let Warned { output, .. } = typst::compile::<PagedDocument>(&world);
    let doc = output.map_err(|errs| {
        errs.iter()
            .map(|e| to_diagnostic(&world, e))
            .collect::<Vec<_>>()
    })?;

    let archival = standards.iter().any(|s| s.to_lowercase().starts_with("a-"));
    let mut opts = PdfOptions {
        standards: pdf_standards,
        ..Default::default()
    };
    if has_ua {
        opts.tagged = true; // PDF/UA requires a tagged PDF (accessibility)
    }
    if archival {
        // PDF/A requires a document date; provide the current date if the
        // document itself doesn't set one.
        if let Some(dt) = world.today(None) {
            opts.timestamp = Some(Timestamp::new_utc(dt));
        }
    }
    typst_pdf::pdf(&doc, &opts)
        .map_err(|errs| errs.iter().map(|e| to_diagnostic(&world, e)).collect())
}

// ── Editor intelligence (completion + hover) via typst-ide ───────────────────

/// UTF-16 offset (CodeMirror position) → UTF-8 byte offset (Typst).
fn utf16_to_byte(text: &str, utf16: usize) -> usize {
    let mut u = 0;
    for (b, c) in text.char_indices() {
        if u >= utf16 {
            return b;
        }
        u += c.len_utf16();
    }
    text.len()
}

/// UTF-8 byte offset (Typst) → UTF-16 offset (CodeMirror position).
fn byte_to_utf16(text: &str, byte: usize) -> usize {
    text.get(..byte.min(text.len()))
        .map(|s| s.chars().map(|c| c.len_utf16()).sum())
        .unwrap_or(0)
}

/// A single completion candidate.
#[derive(Debug, Clone, Serialize)]
pub struct Completion {
    /// Category, e.g. "func", "param", "label" (drives the editor icon).
    pub kind: String,
    /// The text shown in the list.
    pub label: String,
    /// Text to insert (may contain `${…}` snippet placeholders).
    pub apply: Option<String>,
    /// Extra detail (type/signature).
    pub detail: Option<String>,
}

/// Result of a completion request.
#[derive(Debug, Clone, Serialize)]
pub struct CompleteResponse {
    /// UTF-16 offset where the replacement text should start.
    pub from: usize,
    /// The candidates.
    pub items: Vec<Completion>,
}

fn kind_str(k: &CompletionKind) -> &'static str {
    match k {
        CompletionKind::Syntax => "syntax",
        CompletionKind::Func => "func",
        CompletionKind::Type => "type",
        CompletionKind::Param => "param",
        CompletionKind::Constant => "constant",
        CompletionKind::Path => "path",
        CompletionKind::Package => "package",
        CompletionKind::Label => "label",
        CompletionKind::Font => "font",
        CompletionKind::Symbol(_) => "symbol",
    }
}

/// Complete at `cursor` (a UTF-16 offset) in `path` (relative to the root).
/// `overlays` supply the current editor buffer. Always returns a response.
pub fn complete(
    root: &Path,
    entrypoint: &str,
    path: &str,
    overlays: &[Overlay],
    cursor: usize,
    explicit: bool,
) -> CompleteResponse {
    let empty = || CompleteResponse {
        from: cursor,
        items: vec![],
    };
    with_world(root, entrypoint, overlays, |world| {
        let Ok(vp) = VirtualPath::new(path) else {
            return empty();
        };
        let id = FileId::new(RootedPath::new(VirtualRoot::Project, vp));
        let Ok(source) = world.source(id) else {
            return empty();
        };
        let text = source.text();
        let byte = utf16_to_byte(text, cursor);
        match typst_ide::autocomplete(world, None::<&PagedDocument>, &source, byte, explicit) {
            Some((from_byte, items)) => CompleteResponse {
                from: byte_to_utf16(text, from_byte),
                items: items
                    .iter()
                    .take(200)
                    .map(|c| Completion {
                        kind: kind_str(&c.kind).to_string(),
                        label: c.label.to_string(),
                        apply: c.apply.as_ref().map(|a| a.to_string()),
                        detail: c.detail.as_ref().map(|d| d.to_string()),
                    })
                    .collect(),
            },
            None => empty(),
        }
    })
    .unwrap_or_else(|_| empty())
}

/// Where a preview click maps to in the source.
#[derive(Debug, Clone, Serialize)]
pub struct JumpTarget {
    /// File path relative to the project root.
    pub file: String,
    /// UTF-16 offset in that file.
    pub offset: usize,
    /// One-based line number.
    pub line: usize,
}

/// Map a click on `page` at `(x_pt, y_pt)` (points from the top-left) back to the
/// source location that produced it (source↔preview sync, ARCHITECTURE.md §13.1).
pub fn jump_from_click(
    root: &Path,
    entrypoint: &str,
    overlays: &[Overlay],
    page: usize,
    x_pt: f64,
    y_pt: f64,
) -> Option<JumpTarget> {
    use std::num::NonZeroUsize;
    use typst::introspection::PagedPosition;
    use typst::layout::{Abs, Point};

    with_world(root, entrypoint, overlays, |world| {
        let Warned { output, .. } = typst::compile::<PagedDocument>(world);
        let doc = output.ok()?;
        let pos = PagedPosition {
            page: NonZeroUsize::new(page.max(1))?,
            point: Point {
                x: Abs::pt(x_pt),
                y: Abs::pt(y_pt),
            },
        };
        match typst_ide::jump_from_click(world, &doc, &pos)? {
            typst_ide::Jump::File(id, byte) => {
                let source = world.source(id).ok()?;
                let text = source.text();
                Some(JumpTarget {
                    file: id.vpath().get_without_slash().replace('\\', "/"),
                    offset: byte_to_utf16(text, byte),
                    line: source
                        .lines()
                        .byte_to_line(byte)
                        .map(|l| l + 1)
                        .unwrap_or(1),
                })
            }
            _ => None,
        }
    })
    .ok()
    .flatten()
}

/// A location in the rendered document, for editor→preview sync.
#[derive(Debug, Clone, Serialize)]
pub struct PreviewPosition {
    /// One-based page number.
    pub page: usize,
    /// X offset in points from the page's top-left corner.
    pub x: f64,
    /// Y offset in points from the page's top-left corner.
    pub y: f64,
}

/// Map a cursor (UTF-16 offset in `path`) to its location in the rendered
/// document, so the preview can scroll to — and highlight — where you're
/// editing (forward source→preview sync, ARCHITECTURE.md §13.1). Returns `None`
/// when the cursor isn't over text that maps to the output.
pub fn jump_from_cursor(
    root: &Path,
    entrypoint: &str,
    path: &str,
    overlays: &[Overlay],
    cursor: usize,
) -> Option<PreviewPosition> {
    with_world(root, entrypoint, overlays, |world| {
        let vp = VirtualPath::new(path).ok()?;
        let id = FileId::new(RootedPath::new(VirtualRoot::Project, vp));
        let source = world.source(id).ok()?;
        let byte = utf16_to_byte(source.text(), cursor);
        let Warned { output, .. } = typst::compile::<PagedDocument>(world);
        let doc = output.ok()?;
        let pos = typst_ide::jump_from_cursor(&doc, &source, byte)
            .into_iter()
            .next()?;
        Some(PreviewPosition {
            page: pos.page.get(),
            x: pos.point.x.to_pt(),
            y: pos.point.y.to_pt(),
        })
    })
    .ok()
    .flatten()
}

/// Render one page (1-based) to a PNG at `pixel_per_pt` pixels per typographic
/// point, for a high-fidelity raster preview. Rendering at the display's exact
/// pixel density (CSS px × devicePixelRatio) gives crisp output that matches a
/// browser canvas and avoids the WebView's softer scaled-SVG text. Returns
/// `None` if the page doesn't exist or compilation fails.
pub fn render_page_png(
    root: &Path,
    entrypoint: &str,
    overlays: &[Overlay],
    page: usize,
    pixel_per_pt: f64,
) -> Option<Vec<u8>> {
    with_world(root, entrypoint, overlays, |world| {
        let Warned { output, .. } = typst::compile::<PagedDocument>(world);
        let doc = output.ok()?;
        let p = doc.pages().get(page.saturating_sub(1))?;
        let opts = typst_render::RenderOptions {
            pixel_per_pt: typst::utils::Scalar::new(pixel_per_pt.clamp(1.0, 6.0)),
            render_bleed: false,
        };
        typst_render::render(p, &opts).encode_png().ok()
    })
    .ok()
    .flatten()
}

/// A hover tooltip.
#[derive(Debug, Clone, Serialize)]
pub struct Hover {
    /// The tooltip text.
    pub text: String,
    /// Whether it should be rendered as Typst code.
    pub code: bool,
}

/// Hover at `cursor` (UTF-16 offset) in `path`. `None` when there's nothing.
pub fn hover(
    root: &Path,
    entrypoint: &str,
    path: &str,
    overlays: &[Overlay],
    cursor: usize,
) -> Option<Hover> {
    with_world(root, entrypoint, overlays, |world| {
        let vp = VirtualPath::new(path).ok()?;
        let id = FileId::new(RootedPath::new(VirtualRoot::Project, vp));
        let source = world.source(id).ok()?;
        let byte = utf16_to_byte(source.text(), cursor);
        match typst_ide::tooltip(world, None::<&PagedDocument>, &source, byte, Side::Before)? {
            Tooltip::Text(t) => Some(Hover {
                text: t.to_string(),
                code: false,
            }),
            Tooltip::Code(t) => Some(Hover {
                text: t.to_string(),
                code: true,
            }),
        }
    })
    .ok()
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(dir: &Path, name: &str, body: &str) {
        let p = dir.join(name);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, body).unwrap();
    }

    #[test]
    fn compiles_a_simple_document_to_svg() {
        let dir = std::env::temp_dir().join(format!("typide-world-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        fixture(
            &dir,
            "main.typ",
            "= Hello\n\nA paragraph with math $1 + 1 = 2$.\n",
        );
        let result = compile(&dir, "main.typ", &[]);
        assert!(result.page_count >= 1, "expected at least one page");
        assert!(!result.pages.is_empty());
        assert!(result.pages[0].contains("<svg"), "output should be SVG");
        assert!(
            !result.diagnostics.iter().any(|d| d.severity == "error"),
            "unexpected errors: {:?}",
            result.diagnostics
        );
    }

    #[test]
    fn reports_errors_with_location() {
        let dir = std::env::temp_dir().join(format!("typide-world-err-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // `#foo()` is an unknown function → a compile error.
        fixture(&dir, "main.typ", "Intro line\n#undefined_function()\n");
        let result = compile(&dir, "main.typ", &[]);
        assert!(result.diagnostics.iter().any(|d| d.severity == "error"));
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.line >= 1 && !d.file.is_empty()));
    }

    #[test]
    fn exports_pdf_bytes() {
        let dir = std::env::temp_dir().join(format!("typide-pdf-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        fixture(&dir, "main.typ", "= Title\n\nSome text.\n");
        let bytes = export_pdf(&dir, "main.typ", &[]).expect("pdf export");
        assert!(bytes.starts_with(b"%PDF"), "output is a PDF");
    }

    #[test]
    fn exports_pdf_a_archival() {
        let dir = std::env::temp_dir().join(format!("typide-pdfa-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        fixture(
            &dir,
            "main.typ",
            "#set document(title: \"Archival\", author: \"T\")\n#set text(lang: \"en\")\n= Archival\n\nText.\n",
        );
        match export_pdf(&dir, "main.typ", &["a-2b".into()]) {
            Ok(bytes) => assert!(bytes.starts_with(b"%PDF")),
            Err(diags) => panic!(
                "pdf/a failed: {:?}",
                diags.iter().map(|d| &d.message).collect::<Vec<_>>()
            ),
        }
    }

    #[test]
    fn jump_maps_a_click_to_source() {
        let dir = std::env::temp_dir().join(format!("typide-jmp-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        fixture(
            &dir,
            "main.typ",
            "#set page(margin: 1cm)\n#set text(size: 20pt)\n= Heading Here\n\nSome body text here.\n",
        );
        // Click near the top-left where the heading sits.
        let t = jump_from_click(&dir, "main.typ", &[], 1, 45.0, 40.0);
        if let Some(t) = t {
            assert_eq!(t.file, "main.typ");
            assert!(t.line >= 1);
        }
    }

    #[test]
    fn lists_embedded_fonts() {
        let families = list_fonts(None);
        assert!(!families.is_empty(), "embedded fonts should be listed");
        assert!(
            families.iter().any(|f| f.name.to_lowercase().contains("computer modern")
                || f.name.to_lowercase().contains("libertinus")),
            "expected a bundled academic font family"
        );
    }

    #[test]
    fn renders_a_page_to_png() {
        let dir = std::env::temp_dir().join(format!("typide-png-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        fixture(&dir, "main.typ", "= Title\n\nBody text.\n");
        let png = render_page_png(&dir, "main.typ", &[], 1, 2.0).expect("render page");
        assert!(png.len() > 8 && &png[1..4] == b"PNG", "should be a PNG, got {} bytes", png.len());
    }

    #[test]
    fn jump_maps_a_cursor_to_a_page() {
        let dir = std::env::temp_dir().join(format!("typide-jmpc-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        fixture(&dir, "main.typ", "= Heading Here\n\nSome body text here.\n");
        // A cursor inside the body text maps to a point on page 1.
        if let Some(p) = jump_from_cursor(&dir, "main.typ", "main.typ", &[], 20) {
            assert_eq!(p.page, 1);
            assert!(p.y >= 0.0 && p.x >= 0.0);
        }
    }

    #[test]
    fn completes_functions() {
        let dir = std::env::temp_dir().join(format!("typide-cmp-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        fixture(&dir, "main.typ", "#figur");
        let ov = [Overlay {
            path: "main.typ".into(),
            content: "#figur".into(),
        }];
        let r = complete(&dir, "main.typ", "main.typ", &ov, 6, true);
        assert!(
            r.items.iter().any(|c| c.label.starts_with("figure")),
            "expected a 'figure' completion, got {:?}",
            r.items
                .iter()
                .map(|c| &c.label)
                .take(10)
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn overlay_replaces_disk_content() {
        let dir = std::env::temp_dir().join(format!("typide-world-ov-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        fixture(&dir, "main.typ", "= On Disk\n");
        // Compile with an overlay that supersedes the file — no disk write.
        let overlays = [Overlay {
            path: "main.typ".into(),
            content: "= From Overlay\n\nLive.\n".into(),
        }];
        let result = compile(&dir, "main.typ", &overlays);
        assert!(result.page_count >= 1);
        assert!(result.pages[0].contains("<svg"));
        // Disk is untouched.
        assert_eq!(
            std::fs::read_to_string(dir.join("main.typ")).unwrap(),
            "= On Disk\n"
        );
    }
}
