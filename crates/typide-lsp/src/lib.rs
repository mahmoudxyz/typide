//! Tinymist process management and the LSP message bridge.
//!
//! Milestone-0 skeleton crate. See `ARCHITECTURE.md` for the design that the
//! full implementation must follow.
#![deny(missing_docs)]

/// Crate name, exposed for diagnostics and `typide doctor` output.
pub const CRATE_NAME: &str = "typide-lsp";

/// Returns a short human-readable status line for this (stub) crate.
pub fn status() -> String {
    format!("{CRATE_NAME}: skeleton (M0)")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_reports_crate_name() {
        assert!(status().contains("typide-lsp"));
    }
}
