//! SQLite index of symbols, labels, refs, citations, and notes.
//!
//! Milestone-0 skeleton crate. See `ARCHITECTURE.md` for the design that the
//! full implementation must follow.
#![deny(missing_docs)]

/// Crate name, exposed for diagnostics and `typide doctor` output.
pub const CRATE_NAME: &str = "typide-index";

/// Returns a short human-readable status line for this (stub) crate.
pub fn status() -> String {
    format!("{CRATE_NAME}: skeleton (M0)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_reports_crate_name() {
        assert!(status().contains("typide-index"));
    }
}
