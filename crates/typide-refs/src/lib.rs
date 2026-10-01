//! Bibliography / references, including the **Zotero local API** (ARCHITECTURE.md
//! §15.2). All HTTP goes through `typide-net`; Zotero runs on `localhost`, so no
//! data ever leaves the machine.
//!
//! [VERIFY] The Zotero 7 / Better BibTeX endpoints and port (commonly 23119) are
//! not verified against a running instance in this environment — see
//! `docs/verified-facts.md` / `docs/open-questions.md`.
#![deny(missing_docs)]

/// Crate name, exposed for diagnostics.
pub const CRATE_NAME: &str = "typide-refs";

/// Default Zotero local-API port. [VERIFY]
pub const ZOTERO_PORT: u16 = 23119;

/// Whether a Zotero instance with Better BibTeX is reachable locally.
pub fn zotero_status() -> bool {
    // The BBT "cite as you write" probe returns readiness when BBT is loaded.
    let url = format!("http://127.0.0.1:{ZOTERO_PORT}/better-bibtex/cayw?probe=probe");
    typide_net::fetch_bytes(&url).is_ok()
}

/// Export the whole Zotero library as BibLaTeX via Better BibTeX. [VERIFY]
pub fn zotero_export_biblatex() -> Result<String, String> {
    let url = format!("http://127.0.0.1:{ZOTERO_PORT}/better-bibtex/export/library?/1/biblatex");
    let bytes = typide_net::fetch_bytes(&url)
        .map_err(|e| format!("Zotero not reachable (is it running with Better BibTeX?): {e}"))?;
    String::from_utf8(bytes).map_err(|e| format!("invalid UTF-8 from Zotero: {e}"))
}

/// Count the `@type{...}` entries in BibTeX text.
pub fn count_entries(bib: &str) -> usize {
    bib.lines()
        .filter(|l| l.trim_start().starts_with('@'))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_bib_entries() {
        let bib = "@article{a, title={X}}\n@book{b, title={Y}}\nnot an entry\n";
        assert_eq!(count_entries(bib), 2);
    }
}
