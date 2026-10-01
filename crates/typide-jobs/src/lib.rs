//! Background job runner: progress reporting and cooperative cancellation.
//!
//! Long operations (compile, resolve, fetch, export, migrate) run as jobs so the
//! UI thread never blocks (ARCHITECTURE.md P7). Milestone-0 skeleton defines the
//! identifiers and progress/result shapes the runner and events use.
#![deny(missing_docs)]

use std::sync::atomic::{AtomicU64, Ordering};

/// Crate name, exposed for diagnostics.
pub const CRATE_NAME: &str = "typide-jobs";

/// Opaque identifier for a background job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobId(pub u64);

/// Progress update emitted while a job runs. `fraction` is in `0.0..=1.0`.
#[derive(Debug, Clone)]
pub struct Progress {
    /// The job this update belongs to.
    pub job: JobId,
    /// Completion fraction, clamped to `0.0..=1.0`.
    pub fraction: f32,
    /// Human-readable status message.
    pub message: String,
}

/// Terminal outcome of a job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobResult {
    /// Completed successfully.
    Ok,
    /// Cancelled by the user.
    Cancelled,
    /// Failed with a message.
    Failed(String),
}

/// Hands out monotonically increasing [`JobId`]s.
#[derive(Debug, Default)]
pub struct JobIdAllocator {
    next: AtomicU64,
}

impl JobIdAllocator {
    /// Create a fresh allocator starting at id 1.
    pub fn new() -> Self {
        Self {
            next: AtomicU64::new(1),
        }
    }

    /// Allocate the next unique job id.
    pub fn allocate(&self) -> JobId {
        JobId(self.next.fetch_add(1, Ordering::Relaxed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_increasing() {
        let a = JobIdAllocator::new();
        let first = a.allocate();
        let second = a.allocate();
        assert_ne!(first, second);
        assert!(second.0 > first.0);
    }
}
