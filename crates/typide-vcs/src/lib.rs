//! Snapshots and version history — "Git for non-Git users" (ARCHITECTURE.md §16).
//!
//! Implemented over the `git` CLI (invoked with argument arrays, never a shell —
//! §20). Snapshots are ordinary commits, presented to the user as versions. A
//! fallback identity is supplied so commits work even without a configured
//! `user.name`/`user.email`.
#![deny(missing_docs)]

use std::path::Path;
use std::process::Command;

/// Crate name, exposed for diagnostics.
pub const CRATE_NAME: &str = "typide-vcs";

/// A snapshot (git commit) in the project timeline.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Snapshot {
    /// Short commit hash.
    pub id: String,
    /// Commit message.
    pub message: String,
    /// Relative time ("2 hours ago").
    pub when: String,
    /// Whether Typide created it automatically.
    pub auto: bool,
}

fn git(root: &Path, args: &[&str]) -> Result<std::process::Output, String> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|e| format!("git not available: {e}"))
}

fn git_ok(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = git(root, args)?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// True if `root` is inside a git work tree.
pub fn is_repo(root: &Path) -> bool {
    git(root, &["rev-parse", "--is-inside-work-tree"])
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Initialize a git repository at `root` if there isn't one already.
pub fn ensure_repo(root: &Path) -> Result<(), String> {
    if is_repo(root) {
        return Ok(());
    }
    git_ok(root, &["init", "-q"]).map(|_| ())
}

/// Create a snapshot (stage everything, commit). Returns the short hash.
pub fn snapshot(root: &Path, message: &str) -> Result<String, String> {
    ensure_repo(root)?;
    git_ok(root, &["add", "-A"])?;
    // Supply a fallback identity so a commit succeeds on unconfigured machines.
    let out = git(
        root,
        &[
            "-c",
            "user.name=Typide",
            "-c",
            "user.email=typide@localhost",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            message,
        ],
    )?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    git_ok(root, &["rev-parse", "--short", "HEAD"]).map(|s| s.trim().to_string())
}

/// The snapshot timeline (most recent first).
pub fn timeline(root: &Path) -> Result<Vec<Snapshot>, String> {
    if !is_repo(root) {
        return Ok(vec![]);
    }
    // Separate fields with US (0x1f) and records with RS (0x1e).
    let fmt = "--pretty=format:%h\x1f%s\x1f%cr\x1e";
    let text = match git_ok(root, &["log", fmt, "-n", "100"]) {
        Ok(t) => t,
        Err(_) => return Ok(vec![]), // e.g. no commits yet
    };
    let mut out = Vec::new();
    for record in text.split('\x1e') {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }
        let mut parts = record.split('\x1f');
        let id = parts.next().unwrap_or("").to_string();
        let message = parts.next().unwrap_or("").to_string();
        let when = parts.next().unwrap_or("").to_string();
        let auto = message.starts_with("Auto") || message.starts_with("Snapshot before");
        out.push(Snapshot {
            id,
            message,
            when,
            auto,
        });
    }
    Ok(out)
}

/// Restore the working tree to a given commit (files only; HEAD stays put).
pub fn restore(root: &Path, commit: &str) -> Result<(), String> {
    if !is_repo(root) {
        return Err("not a git repository".into());
    }
    git_ok(root, &["checkout", commit, "--", "."]).map(|_| ())
}

/// The current branch name (defaults to "main" if unknown).
pub fn current_branch(root: &Path) -> String {
    git_ok(root, &["branch", "--show-current"])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "main".into())
}

/// The configured `origin` remote URL, if any.
pub fn get_remote(root: &Path) -> Option<String> {
    git_ok(root, &["remote", "get-url", "origin"])
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Set (or update) the `origin` remote URL.
pub fn set_remote(root: &Path, url: &str) -> Result<(), String> {
    ensure_repo(root)?;
    if git(root, &["remote", "set-url", "origin", url])
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return Ok(());
    }
    git_ok(root, &["remote", "add", "origin", url]).map(|_| ())
}

/// Sync with the remote: fetch → merge → push (uses the user's git credentials —
/// SSH agent or an HTTPS credential helper). Network happens inside the `git`
/// subprocess (ARCHITECTURE.md §16.2), gated by the app's consent layer.
pub fn sync(root: &Path) -> Result<String, String> {
    if get_remote(root).is_none() {
        return Err("no 'origin' remote set".into());
    }
    let branch = current_branch(root);
    // Fetch (hard error on failure — that's where auth/network problems surface).
    git_ok(root, &["fetch", "origin"])?;
    // Merge the remote branch if it exists (ignore when it doesn't yet).
    let _ = git(root, &["merge", &format!("origin/{branch}"), "--no-edit"]);
    // Push and set upstream.
    let out = git(root, &["push", "-u", "origin", &branch])?;
    if out.status.success() {
        Ok(format!("synced '{branch}' with origin"))
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!(
            "typide-vcs-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
        ));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn snapshot_and_timeline_roundtrip() {
        // Skip gracefully if git isn't installed in the environment.
        if Command::new("git").arg("--version").output().is_err() {
            return;
        }
        let dir = tmp();
        std::fs::write(dir.join("main.typ"), "= Hello\n").unwrap();
        let id = snapshot(&dir, "Initial snapshot").expect("snapshot");
        assert!(!id.is_empty());

        std::fs::write(dir.join("main.typ"), "= Hello\n\nMore.\n").unwrap();
        snapshot(&dir, "Add a paragraph").expect("second");

        let tl = timeline(&dir).expect("timeline");
        assert_eq!(tl.len(), 2);
        assert_eq!(tl[0].message, "Add a paragraph");
        assert_eq!(tl[1].message, "Initial snapshot");
    }
}
