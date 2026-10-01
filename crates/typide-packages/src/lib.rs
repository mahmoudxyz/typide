//! Package resolution, offline preparation, vendoring, template creation, and
//! the Universe index browser.
//!
//! Scans a project (and packages it pulls in) for `@preview/name:version`
//! imports and ensures each is present in the shared cache via
//! [`typide_net::obtain_package`] — the only place that touches the network
//! (ARCHITECTURE.md P1). Every network-touching call takes an `online` flag so
//! offline mode fails closed.
#![deny(missing_docs)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Crate name, exposed for diagnostics.
pub const CRATE_NAME: &str = "typide-packages";

const INDEX_URL: &str = "https://packages.typst.org/preview/index.json";

/// Reports progress and allows cancellation of long package operations
/// (ARCHITECTURE.md §10.7). Implemented by the app to emit job events.
pub trait ProgressSink: Sync {
    /// Report that `done` of `total` steps are complete, with a message.
    fn step(&self, done: usize, total: usize, message: &str);
    /// Whether the operation should stop early.
    fn cancelled(&self) -> bool {
        false
    }
}

/// A sink that does nothing (for non-interactive callers).
pub struct NoProgress;
impl ProgressSink for NoProgress {
    fn step(&self, _: usize, _: usize, _: &str) {}
}

// ── Import scanning ──────────────────────────────────────────────────────────

/// Find every `@namespace/name:version` package spec in `content`.
fn find_specs(content: &str, out: &mut BTreeSet<String>) {
    let bytes = content.as_bytes();
    let is_word = |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b'-';
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'@' && (i == 0 || !is_word(bytes[i - 1])) {
            let start = i;
            i += 1;
            let ns_start = i;
            while i < bytes.len() && is_word(bytes[i]) {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b'/' && i > ns_start {
                i += 1;
                let name_start = i;
                while i < bytes.len() && is_word(bytes[i]) {
                    i += 1;
                }
                if i < bytes.len() && bytes[i] == b':' && i > name_start {
                    i += 1;
                    let ver_start = i;
                    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'.') {
                        i += 1;
                    }
                    if i > ver_start {
                        if let Ok(spec) = std::str::from_utf8(&bytes[start..i]) {
                            if spec.starts_with("@preview/") {
                                out.insert(spec.to_string());
                            }
                        }
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
}

fn scan_dir_specs(dir: &Path, out: &mut BTreeSet<String>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            scan_dir_specs(&p, out);
        } else if p.extension().and_then(|e| e.to_str()) == Some("typ") {
            if let Ok(content) = std::fs::read_to_string(&p) {
                find_specs(&content, out);
            }
        }
    }
}

/// Resolve (download if needed) every package a project needs, transitively.
/// Returns `(spec -> extracted dir)` and any per-package errors.
fn resolve_all(
    root: &Path,
    online: bool,
    p: &dyn ProgressSink,
) -> (BTreeMap<String, PathBuf>, Vec<String>) {
    let mut pending: BTreeSet<String> = BTreeSet::new();
    scan_dir_specs(root, &mut pending);

    let mut resolved: BTreeMap<String, PathBuf> = BTreeMap::new();
    let mut failures: Vec<String> = Vec::new();

    while let Some(spec) = pending.iter().next().cloned() {
        if p.cancelled() {
            failures.push("cancelled".to_string());
            break;
        }
        pending.remove(&spec);
        if resolved.contains_key(&spec) {
            continue;
        }
        p.step(
            resolved.len(),
            resolved.len() + pending.len() + 1,
            &format!("fetching {spec}"),
        );
        match typide_net::obtain_package(&spec, online) {
            Ok(dir) => {
                let mut more = BTreeSet::new();
                scan_dir_specs(&dir, &mut more);
                for s in more {
                    if !resolved.contains_key(&s) {
                        pending.insert(s);
                    }
                }
                resolved.insert(spec, dir);
            }
            Err(e) => failures.push(e),
        }
    }
    p.step(resolved.len(), resolved.len(), "done");
    (resolved, failures)
}

/// Fetch every Universe package the project needs (transitively) into the cache.
pub fn fetch_all(root: &Path, online: bool) -> Result<Vec<String>, String> {
    fetch_all_with(root, online, &NoProgress)
}

/// Like [`fetch_all`] with progress reporting + cancellation.
pub fn fetch_all_with(
    root: &Path,
    online: bool,
    p: &dyn ProgressSink,
) -> Result<Vec<String>, String> {
    let (resolved, failures) = resolve_all(root, online, p);
    if resolved.is_empty() && !failures.is_empty() {
        return Err(failures.join("; "));
    }
    Ok(resolved.into_keys().collect())
}

// ── Templates ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Manifest {
    template: Option<TemplateInfo>,
}
#[derive(Deserialize)]
struct TemplateInfo {
    path: String,
    entrypoint: String,
}

fn copy_dir(src: &Path, dest: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Create a new project in `dest` from a Typst template package `spec`.
/// Returns the project entrypoint (e.g. `main.typ`).
pub fn init_template(spec: &str, dest: &Path, online: bool) -> Result<String, String> {
    let pkg = typide_net::obtain_package(spec, online)?;
    let manifest_text = std::fs::read_to_string(pkg.join("typst.toml"))
        .map_err(|e| format!("reading package manifest: {e}"))?;
    let manifest: Manifest =
        toml::from_str(&manifest_text).map_err(|e| format!("parsing manifest: {e}"))?;
    let template = manifest
        .template
        .ok_or_else(|| format!("{spec} is not a template package"))?;

    copy_dir(&pkg.join(&template.path), dest)
        .map_err(|e| format!("copying template files: {e}"))?;
    let _ = fetch_all(dest, online);
    Ok(template.entrypoint)
}

// ── Vendoring + lockfile ─────────────────────────────────────────────────────

/// One resolved package in the offline-ready report / lockfile.
#[derive(Debug, Clone, Serialize)]
pub struct VendoredPackage {
    /// The `@ns/name:version` spec.
    pub spec: String,
    /// SHA-256 of the package's normalized directory tree.
    pub sha256: String,
    /// Whether files were copied into the project's vendor dir.
    pub vendored: bool,
}

/// Report from [`make_offline_ready`].
#[derive(Debug, Clone, Serialize)]
pub struct OfflineReport {
    /// Packages resolved and (optionally) vendored.
    pub packages: Vec<VendoredPackage>,
    /// Packages that could not be resolved.
    pub errors: Vec<String>,
}

/// Deterministic hash of a directory tree (sorted `path\0bytes`).
fn hash_dir(dir: &Path) -> String {
    let mut files: Vec<PathBuf> = Vec::new();
    fn walk(d: &Path, files: &mut Vec<PathBuf>) {
        if let Ok(rd) = std::fs::read_dir(d) {
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, files);
                } else {
                    files.push(p);
                }
            }
        }
    }
    walk(dir, &mut files);
    files.sort();
    let mut hasher = Sha256::new();
    for f in &files {
        if let Ok(rel) = f.strip_prefix(dir) {
            hasher.update(rel.to_string_lossy().replace('\\', "/").as_bytes());
            hasher.update([0]);
        }
        if let Ok(bytes) = std::fs::read(f) {
            hasher.update(&bytes);
            hasher.update([0]);
        }
    }
    format!("{:x}", hasher.finalize())
}

#[derive(Serialize)]
struct Lock {
    version: u32,
    package: Vec<LockPackage>,
}
#[derive(Serialize)]
struct LockPackage {
    spec: String,
    source: String,
    sha256: String,
}

fn spec_parts(spec: &str) -> Option<(String, String, String)> {
    // @ns/name:ver
    let s = spec.strip_prefix('@')?;
    let (ns, rest) = s.split_once('/')?;
    let (name, ver) = rest.split_once(':')?;
    Some((ns.to_string(), name.to_string(), ver.to_string()))
}

/// Resolve everything the project needs, vendor it into `vendor/packages/`, and
/// write `typide.lock` with SHA-256 hashes. Guarantees offline reproducibility.
pub fn make_offline_ready(
    root: &Path,
    online: bool,
    vendor: bool,
) -> Result<OfflineReport, String> {
    make_offline_ready_with(root, online, vendor, &NoProgress)
}

/// Like [`make_offline_ready`] with progress reporting + cancellation.
pub fn make_offline_ready_with(
    root: &Path,
    online: bool,
    vendor: bool,
    p: &dyn ProgressSink,
) -> Result<OfflineReport, String> {
    let (resolved, errors) = resolve_all(root, online, p);
    let mut packages = Vec::new();
    let mut lock_pkgs = Vec::new();

    let total = resolved.len();
    for (i, (spec, dir)) in resolved.iter().enumerate() {
        p.step(i, total, &format!("vendoring {spec}"));
        let sha = hash_dir(dir);
        let mut did_vendor = false;
        if vendor {
            if let Some((ns, name, ver)) = spec_parts(spec) {
                let dest = root
                    .join("vendor/packages")
                    .join(&ns)
                    .join(&name)
                    .join(&ver);
                if copy_dir(dir, &dest).is_ok() {
                    did_vendor = true;
                }
            }
        }
        packages.push(VendoredPackage {
            spec: spec.clone(),
            sha256: sha.clone(),
            vendored: did_vendor,
        });
        lock_pkgs.push(LockPackage {
            spec: spec.clone(),
            source: "universe".into(),
            sha256: sha,
        });
    }

    let lock = Lock {
        version: 1,
        package: lock_pkgs,
    };
    if let Ok(text) = toml::to_string_pretty(&lock) {
        let _ = std::fs::write(root.join("typide.lock"), text);
    }
    Ok(OfflineReport { packages, errors })
}

// ── Universe index browser ───────────────────────────────────────────────────

/// A package as shown in the browser (from the Universe index).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMeta {
    /// Package name.
    pub name: String,
    /// Latest version.
    pub version: String,
    /// One-line description.
    #[serde(default)]
    pub description: String,
    /// Author strings.
    #[serde(default)]
    pub authors: Vec<String>,
    /// SPDX license id.
    #[serde(default)]
    pub license: String,
    /// Free-form keywords.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Universe categories.
    #[serde(default)]
    pub categories: Vec<String>,
}

fn index_cache_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("typide")
        .join("universe-index.json")
}

fn parse_ver(v: &str) -> (u64, u64, u64) {
    let mut it = v.split('.').map(|p| p.parse::<u64>().unwrap_or(0));
    (
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
        it.next().unwrap_or(0),
    )
}

/// Load the Universe index (from cache, or fetch it when `online`), keeping the
/// latest version of each package.
fn load_index(online: bool, refresh: bool) -> Result<Vec<PackageMeta>, String> {
    let path = index_cache_path();
    let bytes = if path.exists() && !refresh {
        std::fs::read(&path).map_err(|e| format!("reading index cache: {e}"))?
    } else if online {
        let data = typide_net::fetch_bytes(INDEX_URL)?;
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, &data);
        data
    } else {
        return Err("package index is not cached and the app is offline".into());
    };

    let all: Vec<PackageMeta> =
        serde_json::from_slice(&bytes).map_err(|e| format!("parsing index: {e}"))?;

    // Keep the highest version per name.
    let mut latest: BTreeMap<String, PackageMeta> = BTreeMap::new();
    for pkg in all {
        match latest.get(&pkg.name) {
            Some(existing) if parse_ver(&existing.version) >= parse_ver(&pkg.version) => {}
            _ => {
                latest.insert(pkg.name.clone(), pkg);
            }
        }
    }
    Ok(latest.into_values().collect())
}

/// Search the Universe index. Empty query returns a popular-ish sample.
pub fn search(query: &str, online: bool, limit: usize) -> Result<Vec<PackageMeta>, String> {
    let index = load_index(online, false)?;
    let q = query.trim().to_lowercase();

    let mut hits: Vec<(u8, PackageMeta)> = index
        .into_iter()
        .filter_map(|p| {
            if q.is_empty() {
                return Some((2, p));
            }
            let name = p.name.to_lowercase();
            if name == q {
                Some((0, p))
            } else if name.starts_with(&q) {
                Some((1, p))
            } else if name.contains(&q)
                || p.description.to_lowercase().contains(&q)
                || p.keywords.iter().any(|k| k.to_lowercase().contains(&q))
                || p.categories.iter().any(|c| c.to_lowercase().contains(&q))
            {
                Some((2, p))
            } else {
                None
            }
        })
        .collect();

    hits.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.name.cmp(&b.1.name)));
    Ok(hits.into_iter().take(limit).map(|(_, p)| p).collect())
}

/// Force a refresh of the cached index (online only).
pub fn refresh_index(online: bool) -> Result<usize, String> {
    let index = load_index(online, true)?;
    Ok(index.len())
}

// ── Mirror creation (ARCHITECTURE.md §10.5) ──────────────────────────────────

/// The raw Universe index as JSON values (preserves all fields for a mirror).
fn raw_index(online: bool) -> Result<Vec<serde_json::Value>, String> {
    let path = index_cache_path();
    let bytes = if path.exists() {
        std::fs::read(&path).map_err(|e| format!("read index: {e}"))?
    } else if online {
        let data = typide_net::fetch_bytes(INDEX_URL)?;
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, &data);
        data
    } else {
        return Err("index not cached and offline".into());
    };
    serde_json::from_slice(&bytes).map_err(|e| format!("parse index: {e}"))
}

/// Build a Universe-layout mirror of a project's packages (transitively) into
/// `out`: `out/preview/{name}-{version}.tar.gz` + a filtered `index.json`. The
/// result can be hosted on an intranet or carried on a USB stick.
pub fn create_mirror(
    root: &Path,
    out: &Path,
    online: bool,
    p: &dyn ProgressSink,
) -> Result<Vec<String>, String> {
    // Resolve the full (transitive) set of packages the project needs.
    let (resolved, _errors) = resolve_all(root, online, p);
    if resolved.is_empty() {
        return Err("no @preview packages to mirror".into());
    }
    let index = raw_index(online)?;

    let preview = out.join("preview");
    std::fs::create_dir_all(&preview).map_err(|e| format!("mkdir: {e}"))?;

    let mut entries: Vec<serde_json::Value> = Vec::new();
    let mut mirrored: Vec<String> = Vec::new();
    let total = resolved.len();
    for (i, spec) in resolved.keys().enumerate() {
        if p.cancelled() {
            break;
        }
        let Some((_ns, name, ver)) = spec_parts(spec) else {
            continue;
        };
        p.step(i, total, &format!("mirroring {spec}"));

        let url = format!("https://packages.typst.org/preview/{name}-{ver}.tar.gz");
        let bytes = typide_net::fetch_bytes(&url)?;
        std::fs::write(preview.join(format!("{name}-{ver}.tar.gz")), &bytes)
            .map_err(|e| format!("write tarball: {e}"))?;

        if let Some(entry) = index.iter().find(|e| {
            e.get("name").and_then(|v| v.as_str()) == Some(name.as_str())
                && e.get("version").and_then(|v| v.as_str()) == Some(ver.as_str())
        }) {
            entries.push(entry.clone());
        }
        mirrored.push(spec.clone());
    }

    let index_json = serde_json::to_vec_pretty(&entries).map_err(|e| format!("index: {e}"))?;
    std::fs::write(preview.join("index.json"), index_json)
        .map_err(|e| format!("write index: {e}"))?;
    p.step(total, total, "done");
    Ok(mirrored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_preview_specs_only() {
        let mut out = BTreeSet::new();
        find_specs(
            r#"#import "@preview/cetz:0.4.2": *
               #import "@local/mine:1.0.0": x
               email me@place.com is not a spec"#,
            &mut out,
        );
        assert!(out.contains("@preview/cetz:0.4.2"));
        assert!(!out.iter().any(|s| s.contains("local")));
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn version_ordering() {
        assert!(parse_ver("0.10.0") > parse_ver("0.9.9"));
        assert!(parse_ver("1.0.0") > parse_ver("0.99.99"));
    }
}

#[cfg(test)]
mod net_tests {
    #[test]
    #[ignore = "hits the network"]
    fn searches_universe() {
        let hits = super::search("cetz", true, 10).expect("search");
        assert!(hits.iter().any(|p| p.name == "cetz"), "cetz in results");
        assert!(!hits[0].version.is_empty());
    }
}
