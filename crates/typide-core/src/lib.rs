//! The facade crate: workspace/project model, config, and the typed event bus.
//!
//! `typide-core` is the seam the Tauri layer talks to. Milestone-0 skeleton
//! defines the core domain types (ARCHITECTURE.md §6) and the [`CoreEvent`] enum
//! the event bus carries.
#![deny(missing_docs)]

use std::path::PathBuf;

pub use typide_jobs::{JobId, JobResult};

/// Crate name, exposed for diagnostics.
pub const CRATE_NAME: &str = "typide-core";

/// Opaque project identifier within a [`Workspace`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectId(pub u64);

/// What kind of project this is; drives templates and default inspection sets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectKind {
    /// A thesis (defaults to vendored packages, strict submission checks).
    Thesis,
    /// A paper / article.
    Paper,
    /// A notes vault.
    Notes,
    /// A reusable Typst package.
    Package,
    /// Anything else.
    Generic,
}

/// A single Typst project rooted at a folder containing `typide.toml`.
#[derive(Debug, Clone)]
pub struct Project {
    /// The Typst `--root`.
    pub root: PathBuf,
    /// Entry point, e.g. `main.typ`.
    pub entrypoint: PathBuf,
    /// Project classification.
    pub kind: ProjectKind,
}

/// A compiler diagnostic surfaced to the UI.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// File the diagnostic applies to.
    pub file: PathBuf,
    /// One-based line number.
    pub line: u32,
    /// Severity string: `error` | `warning` | `hint`.
    pub severity: String,
    /// Human-readable message.
    pub message: String,
}

/// Events published on the core event bus and forwarded to the frontend
/// (ARCHITECTURE.md §6.4).
#[derive(Debug, Clone)]
pub enum CoreEvent {
    /// A watched file changed on disk.
    FileChanged {
        /// The file that changed.
        path: PathBuf,
    },
    /// A compile finished.
    CompileFinished {
        /// The project that was compiled.
        project: ProjectId,
        /// Wall-clock compile time in milliseconds.
        duration_ms: u64,
        /// Diagnostics produced.
        diagnostics: Vec<Diagnostic>,
    },
    /// Background job progress.
    JobProgress {
        /// The job reporting progress.
        job: JobId,
        /// Completion fraction in `0.0..=1.0`.
        fraction: f32,
        /// Status message.
        message: String,
    },
    /// A background job finished.
    JobFinished {
        /// The job that finished.
        job: JobId,
        /// Its terminal result.
        result: JobResult,
    },
    /// The network policy's offline flag changed.
    NetworkPolicyChanged {
        /// Whether the app is now offline.
        offline: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_kind_is_copy() {
        let k = ProjectKind::Thesis;
        let k2 = k;
        assert_eq!(k, k2);
    }

    #[test]
    fn event_carries_diagnostics() {
        let ev = CoreEvent::CompileFinished {
            project: ProjectId(1),
            duration_ms: 42,
            diagnostics: vec![],
        };
        match ev {
            CoreEvent::CompileFinished { duration_ms, .. } => {
                assert_eq!(duration_ms, 42)
            }
            _ => panic!("wrong variant"),
        }
    }
}
