//! Toolchain management: list, detect and install Typst compiler versions.
//!
//! The app bundles one **native** Typst version (linked into `typide-world`);
//! other versions are downloaded from GitHub releases into the app data dir and
//! run as CLI subprocesses (ARCHITECTURE.md §9). Downloads go through
//! `typide-net` (the only crate doing network I/O — P1).
#![deny(missing_docs)]

use std::io::{Cursor, Read};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

/// Crate name, exposed for diagnostics.
pub const CRATE_NAME: &str = "typide-toolchain";

/// The Typst version linked into the app (ARCHITECTURE.md Appendix B).
pub const NATIVE_VERSION: &str = "0.15.1";

/// An installed or detected toolchain.
#[derive(Debug, Clone, Serialize)]
pub struct Toolchain {
    /// Typst version, e.g. "0.15.1".
    pub version: String,
    /// "bundled" | "system" | "installed".
    pub source: String,
    /// Where it lives (path or "embedded"/"PATH").
    pub path: String,
}

fn toolchains_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("typide")
        .join("toolchains")
        .join("typst")
}

/// List every Typst toolchain available: the bundled one, any on `PATH`, and
/// versions installed into the app data dir.
pub fn list_installed() -> Vec<Toolchain> {
    let mut out = vec![Toolchain {
        version: NATIVE_VERSION.to_string(),
        source: "bundled".into(),
        path: "embedded".into(),
    }];

    if let Ok(o) = Command::new("typst").arg("--version").output() {
        if o.status.success() {
            let text = String::from_utf8_lossy(&o.stdout);
            if let Some(ver) = text.split_whitespace().nth(1) {
                if ver != NATIVE_VERSION {
                    out.push(Toolchain {
                        version: ver.to_string(),
                        source: "system".into(),
                        path: "PATH".into(),
                    });
                }
            }
        }
    }

    if let Ok(rd) = std::fs::read_dir(toolchains_dir()) {
        for e in rd.flatten() {
            let bin = e
                .path()
                .join(if cfg!(windows) { "typst.exe" } else { "typst" });
            if bin.exists() {
                out.push(Toolchain {
                    version: e.file_name().to_string_lossy().to_string(),
                    source: "installed".into(),
                    path: bin.to_string_lossy().to_string(),
                });
            }
        }
    }
    out
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    prerelease: bool,
}

/// List Typst versions available to install from GitHub releases.
pub fn list_available(online: bool) -> Result<Vec<String>, String> {
    if !online {
        return Err("offline: cannot list available toolchains".into());
    }
    let bytes =
        typide_net::fetch_bytes("https://api.github.com/repos/typst/typst/releases?per_page=40")?;
    let rels: Vec<Release> =
        serde_json::from_slice(&bytes).map_err(|e| format!("parse releases: {e}"))?;
    Ok(rels
        .into_iter()
        .filter(|r| !r.prerelease)
        .map(|r| r.tag_name.trim_start_matches('v').to_string())
        .collect())
}

/// The release asset name for the current platform.
fn asset_name() -> Option<&'static str> {
    Some(match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => "typst-x86_64-unknown-linux-musl.tar.xz",
        ("linux", "aarch64") => "typst-aarch64-unknown-linux-musl.tar.xz",
        ("macos", "x86_64") => "typst-x86_64-apple-darwin.tar.xz",
        ("macos", "aarch64") => "typst-aarch64-apple-darwin.tar.xz",
        ("windows", "x86_64") => "typst-x86_64-pc-windows-msvc.zip",
        _ => return None,
    })
}

fn bin_name() -> &'static str {
    if cfg!(windows) {
        "typst.exe"
    } else {
        "typst"
    }
}

/// Extract the `typst` binary bytes from a `.tar.xz` archive.
fn typst_from_tar_xz(archive_bytes: &[u8]) -> Result<Vec<u8>, String> {
    let mut tar_bytes = Vec::new();
    lzma_rs::xz_decompress(&mut Cursor::new(archive_bytes), &mut tar_bytes)
        .map_err(|e| format!("xz decompress: {e}"))?;
    let mut archive = tar::Archive::new(Cursor::new(tar_bytes));
    for entry in archive.entries().map_err(|e| format!("tar: {e}"))? {
        let mut entry = entry.map_err(|e| format!("tar entry: {e}"))?;
        let is_bin = entry
            .path()
            .map(|p| p.file_name().map(|n| n == bin_name()).unwrap_or(false))
            .unwrap_or(false);
        if is_bin {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| format!("read: {e}"))?;
            return Ok(buf);
        }
    }
    Err("typst binary not found in archive".into())
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755));
}
#[cfg(not(unix))]
fn make_executable(_path: &Path) {}

/// Place extracted binary bytes under `toolchains/<version>/` and return it.
fn place_binary(version: &str, bytes: &[u8]) -> Result<Toolchain, String> {
    let dest = toolchains_dir().join(version);
    std::fs::create_dir_all(&dest).map_err(|e| format!("mkdir: {e}"))?;
    let bin = dest.join(bin_name());
    std::fs::write(&bin, bytes).map_err(|e| format!("write: {e}"))?;
    make_executable(&bin);
    Ok(Toolchain {
        version: version.to_string(),
        source: "installed".into(),
        path: bin.to_string_lossy().to_string(),
    })
}

/// Run `typst --version` on a binary file and parse the version.
fn probe_version(bin: &Path) -> Result<String, String> {
    let out = Command::new(bin)
        .arg("--version")
        .output()
        .map_err(|e| format!("running typst --version: {e}"))?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.split_whitespace()
        .nth(1)
        .map(|s| s.to_string())
        .ok_or_else(|| "could not parse typst version".into())
}

/// Download and install a Typst version into the app data dir (returns it).
pub fn install(version: &str, online: bool) -> Result<Toolchain, String> {
    if !online {
        return Err("offline: cannot download toolchain".into());
    }
    let asset = asset_name().ok_or("unsupported platform")?;
    if !asset.ends_with(".tar.xz") {
        return Err("archive format not supported on this platform yet".into());
    }
    let url = format!("https://github.com/typst/typst/releases/download/v{version}/{asset}");
    let bytes = typide_net::fetch_bytes(&url)?;
    let bin_bytes = typst_from_tar_xz(&bytes)?;
    place_binary(version, &bin_bytes)
}

/// Install a Typst toolchain from a local `.tar.xz` archive (fully offline).
///
/// The version is read from the extracted binary (`typst --version`), so the
/// archive can be carried on a USB stick to air-gapped machines (§9.4).
pub fn install_from_archive(path: &Path) -> Result<Toolchain, String> {
    let name = path.to_string_lossy();
    if !name.ends_with(".tar.xz") {
        return Err("only .tar.xz archives are supported".into());
    }
    let bytes = std::fs::read(path).map_err(|e| format!("read archive: {e}"))?;
    let bin_bytes = typst_from_tar_xz(&bytes)?;

    // Stage in a temp file to probe the version.
    let tmp = std::env::temp_dir().join(format!("typide-tc-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).map_err(|e| format!("tmp: {e}"))?;
    let tmp_bin = tmp.join(bin_name());
    std::fs::write(&tmp_bin, &bin_bytes).map_err(|e| format!("write tmp: {e}"))?;
    make_executable(&tmp_bin);
    let version = probe_version(&tmp_bin)?;
    let _ = std::fs::remove_dir_all(&tmp);

    place_binary(&version, &bin_bytes)
}

/// Remove an installed toolchain (bundled/system cannot be removed).
pub fn remove(version: &str) -> Result<(), String> {
    let dir = toolchains_dir().join(version);
    if dir.is_dir() {
        std::fs::remove_dir_all(&dir).map_err(|e| format!("remove: {e}"))
    } else {
        Err(format!("{version} is not an installed toolchain"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_at_least_the_bundled_toolchain() {
        let list = list_installed();
        assert!(list
            .iter()
            .any(|t| t.source == "bundled" && t.version == NATIVE_VERSION));
    }

    #[test]
    fn asset_name_known_for_common_platforms() {
        // On the dev/CI platform this must resolve.
        assert!(
            asset_name().is_some()
                || cfg!(not(any(
                    target_os = "linux",
                    target_os = "macos",
                    target_os = "windows"
                )))
        );
    }
}
