//! Inspections — currently a prose **spell checker** for Typst markup.
//!
//! Text is extracted from the *markup Text nodes only* using `typst-syntax` (so
//! code, math, labels and imports are never flagged — ARCHITECTURE.md §7.1), and
//! suggestions are ranked with Damerau-Levenshtein distance against a bundled
//! English word list (ordered by frequency, so common words rank first).
#![deny(missing_docs)]

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde::Serialize;
use typst_syntax::{LinkedNode, Source, SyntaxKind};

/// Crate name, exposed for diagnostics.
pub const CRATE_NAME: &str = "typide-check";

const WORDS_RAW: &str = include_str!("../data/words.txt");
const EXTRA: &[&str] = &[
    "typst",
    "typeset",
    "typesetting",
    "offline",
    "workflow",
    "workflows",
    "pipeline",
    "dataset",
    "datasets",
    "runtime",
    "async",
    "boolean",
    "config",
    "metadata",
    "namespace",
    "svg",
    "pdf",
    "http",
    "https",
    "url",
    "uri",
    "json",
    "toml",
    "cli",
    "ide",
    "api",
    "app",
    "apps",
    "md",
    "utf",
    "png",
    "jpeg",
    "regex",
    "lockfile",
    "vendored",
    "backlinks",
    "wiki",
    "zotero",
    "bibliography",
    "citations",
    "preprint",
    "arxiv",
    "et",
    "al",
    "fig",
    "eq",
    "ref",
    "todo",
    "params",
    "param",
    "struct",
    "enum",
    "https",
    "todo",
    "todos",
];

struct Dict {
    ordered: Vec<String>,
    set: HashSet<String>,
}

fn dict() -> &'static Dict {
    static D: OnceLock<Dict> = OnceLock::new();
    D.get_or_init(|| {
        let ordered: Vec<String> = WORDS_RAW
            .split_whitespace()
            .map(|w| w.to_string())
            .collect();
        let mut set: HashSet<String> = ordered.iter().cloned().collect();
        for w in EXTRA {
            set.insert((*w).to_string());
        }
        Dict { ordered, set }
    })
}

/// A misspelled word with UTF-16 offsets (CodeMirror positions) and suggestions.
#[derive(Debug, Clone, Serialize)]
pub struct Misspelling {
    /// UTF-16 start offset in the document.
    pub start: usize,
    /// UTF-16 end offset.
    pub end: usize,
    /// The word as written.
    pub word: String,
    /// Ranked replacement suggestions (best first).
    pub suggestions: Vec<String>,
}

fn byte_to_utf16(text: &str, byte: usize) -> usize {
    text.get(..byte.min(text.len()))
        .map(|s| s.chars().map(|c| c.len_utf16()).sum())
        .unwrap_or(0)
}

/// Damerau-Levenshtein (optimal string alignment) distance, capped at `max`.
fn distance(a: &[char], b: &[char], max: usize) -> usize {
    let (n, m) = (a.len(), b.len());
    if n.abs_diff(m) > max {
        return max + 1;
    }
    let mut prev2 = vec![0usize; m + 1];
    let mut prev = (0..=m).collect::<Vec<_>>();
    let mut curr = vec![0usize; m + 1];
    for i in 1..=n {
        curr[0] = i;
        let mut row_min = curr[0];
        for j in 1..=m {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            let mut v = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                v = v.min(prev2[j - 2] + 1);
            }
            curr[j] = v;
            row_min = row_min.min(v);
        }
        if row_min > max {
            return max + 1;
        }
        std::mem::swap(&mut prev2, &mut prev);
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

fn known(lower: &str) -> bool {
    dict().set.contains(lower)
}

fn suggest(word: &str) -> Vec<String> {
    let lower = word.to_lowercase();
    let wl: Vec<char> = lower.chars().collect();
    let max = if wl.len() <= 4 { 1 } else { 2 };

    let mut scored: Vec<(usize, usize, &str)> = Vec::new();
    for (rank, cand) in dict().ordered.iter().enumerate() {
        if cand.len().abs_diff(wl.len()) > max {
            continue;
        }
        let cc: Vec<char> = cand.chars().collect();
        let d = distance(&wl, &cc, max);
        if d >= 1 && d <= max {
            scored.push((d, rank, cand.as_str()));
        }
    }
    scored.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    // Preserve the original capitalization pattern.
    let capitalize = word.chars().next().is_some_and(|c| c.is_uppercase());
    scored
        .into_iter()
        .take(7)
        .map(|(_, _, s)| {
            if capitalize {
                let mut c = s.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => s.to_string(),
                }
            } else {
                s.to_string()
            }
        })
        .collect()
}

fn checkable(word: &str) -> bool {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < 3 {
        return false;
    }
    // Skip ALL-CAPS acronyms and anything with a digit.
    if chars.iter().all(|c| c.is_uppercase()) {
        return false;
    }
    if chars.iter().any(|c| c.is_ascii_digit()) {
        return false;
    }
    true
}

/// Spell-check Typst `text`, returning misspellings in markup prose only.
pub fn spellcheck(text: &str) -> Vec<Misspelling> {
    let source = Source::detached(text.to_string());
    let root = LinkedNode::new(source.root());
    let mut spans: Vec<(usize, &str)> = Vec::new(); // (abs byte start, word)
    collect_text(&root, text, &mut spans);

    let mut cache: HashMap<String, Vec<String>> = HashMap::new();
    let mut out: Vec<Misspelling> = Vec::new();
    for (byte_start, word) in spans {
        if !checkable(word) {
            continue;
        }
        let lower = word.to_lowercase();
        let base = lower.trim_end_matches("'s");
        if known(&lower) || known(base) {
            continue;
        }
        let suggestions = cache
            .entry(lower.clone())
            .or_insert_with(|| suggest(word))
            .clone();
        out.push(Misspelling {
            start: byte_to_utf16(text, byte_start),
            end: byte_to_utf16(text, byte_start + word.len()),
            word: word.to_string(),
            suggestions,
        });
        if out.len() >= 1000 {
            break;
        }
    }
    out
}

/// Walk the syntax tree collecting words inside markup `Text` nodes.
fn collect_text<'a>(node: &LinkedNode<'a>, doc: &'a str, out: &mut Vec<(usize, &'a str)>) {
    if node.get().kind() == SyntaxKind::Text {
        let start = node.range().start;
        let slice = &doc[start..node.range().end.min(doc.len())];
        for (off, word) in words(slice) {
            out.push((start + off, word));
        }
    }
    for child in node.children() {
        collect_text(&child, doc, out);
    }
}

/// Yield `(byte offset, word)` for alphabetic words (allowing internal `'`).
fn words(text: &str) -> Vec<(usize, &str)> {
    let bytes = text.as_bytes();
    let is_letter = |c: u8| c.is_ascii_alphabetic();
    let is_inner = |c: u8| c.is_ascii_alphabetic() || c == b'\'';
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if is_letter(bytes[i]) {
            let start = i;
            i += 1;
            while i < bytes.len() && is_inner(bytes[i]) {
                i += 1;
            }
            // Trim a trailing apostrophe.
            let mut end = i;
            if end > start && bytes[end - 1] == b'\'' {
                end -= 1;
            }
            if let Some(w) = text.get(start..end) {
                out.push((start, w));
            }
        } else {
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_prose_typos_not_code() {
        let text = "This sentance has a typพo.\n\n#set text(size: 10pt) // colr is code\n";
        let ms = spellcheck(text);
        assert!(
            ms.iter().any(|m| m.word == "sentance"),
            "found: {:?}",
            ms.iter().map(|m| &m.word).collect::<Vec<_>>()
        );
        // 'colr' is inside a comment/code path, or at least 'size'/'text' (code) not flagged.
        assert!(!ms.iter().any(|m| m.word == "size" || m.word == "text"));
    }

    #[test]
    fn suggests_correct_spelling() {
        let s = suggest("sentance");
        assert!(s.iter().any(|w| w == "sentence"), "suggestions: {s:?}");
    }

    #[test]
    fn skips_known_and_short_words() {
        let ms = spellcheck("The cat sat on the mat and ran.");
        assert!(
            ms.is_empty(),
            "unexpected: {:?}",
            ms.iter().map(|m| &m.word).collect::<Vec<_>>()
        );
    }
}
