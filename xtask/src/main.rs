//! Repository automation tasks.
//!
//! Usage: `cargo xtask <command>`
//!   check-deps   Enforce the network-isolation rule (ARCHITECTURE.md §5).
//!   ci           Run fmt + clippy + tests + check-deps (thin wrapper).
#![deny(missing_docs)]

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

/// Crates permitted to depend on `typide-net` (ARCHITECTURE.md §5 dependency rule).
const NET_ALLOWED: &[&str] = &[
    "typide-toolchain",
    "typide-packages",
    "typide-refs",
    "typide-vcs",
    "typide-ai",
];

fn main() -> ExitCode {
    let cmd = std::env::args().nth(1).unwrap_or_default();
    match cmd.as_str() {
        "check-deps" => check_deps(),
        "ci" => ci(),
        other => {
            eprintln!("unknown xtask command: {other:?}");
            eprintln!("available: check-deps, ci");
            ExitCode::FAILURE
        }
    }
}

/// Workspace root = parent of the `xtask` crate directory.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask has a parent dir")
        .to_path_buf()
}

/// Enforce that only whitelisted crates depend on `typide-net`.
fn check_deps() -> ExitCode {
    let crates_dir = workspace_root().join("crates");
    let mut violations = Vec::new();

    let entries = match std::fs::read_dir(&crates_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("cannot read {}: {e}", crates_dir.display());
            return ExitCode::FAILURE;
        }
    };

    for entry in entries.flatten() {
        let manifest = entry.path().join("Cargo.toml");
        if !manifest.exists() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let text = std::fs::read_to_string(&manifest).unwrap_or_default();
        let depends_on_net = text
            .lines()
            .any(|l| l.contains("typide-net") && !l.trim_start().starts_with('#'));

        if depends_on_net && name != "typide-net" && !NET_ALLOWED.contains(&name.as_str()) {
            violations.push(name);
        }
    }

    if violations.is_empty() {
        println!("check-deps: OK — network I/O is isolated to typide-net.");
        ExitCode::SUCCESS
    } else {
        eprintln!("check-deps: FAILED — these crates must not depend on typide-net:");
        for v in &violations {
            eprintln!("  - {v}");
        }
        eprintln!("Only {NET_ALLOWED:?} may (ARCHITECTURE.md §5).");
        ExitCode::FAILURE
    }
}

/// Run the local CI pipeline.
fn ci() -> ExitCode {
    let steps: &[(&str, &[&str])] = &[
        ("fmt", &["fmt", "--all", "--check"]),
        ("clippy", &["clippy", "--workspace", "--", "-D", "warnings"]),
        ("test", &["test", "--workspace"]),
    ];
    for (label, args) in steps {
        eprintln!("== cargo {label} ==");
        let status = Command::new("cargo").args(*args).status();
        match status {
            Ok(s) if s.success() => {}
            _ => {
                eprintln!("step {label} failed");
                return ExitCode::FAILURE;
            }
        }
    }
    check_deps()
}
