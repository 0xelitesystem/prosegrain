//! Text-handling utilities. Kept deliberately simple and dependency-light.

use std::ops::Range;

/// Replace fenced code blocks, inline code, and explicit ignore regions
/// with whitespace, preserving byte offsets so line/column numbers stay
/// accurate. We don't lint code, and we honor `<!-- prosegrain:off -->`
/// .. `<!-- prosegrain:on -->` so docs that quote AI patterns can opt out.
pub fn strip_code_blocks(input: &str) -> String {
    let with_ignores = strip_ignore_regions(input);
    let mut out = String::with_capacity(with_ignores.len());
    let mut chars = with_ignores.char_indices().peekable();
    let mut in_fence = false;
    let mut fence_marker: Option<char> = None;

    while let Some((i, c)) = chars.next() {
        // Detect ``` or ~~~ fence
        if (c == '`' || c == '~') && is_fence_at(&with_ignores, i, c) {
            in_fence = !in_fence;
            if in_fence {
                fence_marker = Some(c);
            } else {
                fence_marker = None;
            }
            // consume the three fence chars
            for _ in 0..2 {
                chars.next();
            }
            out.push_str("   "); // preserve byte count
            continue;
        }

        if in_fence {
            out.push(if c == '\n' { '\n' } else { ' ' });
            continue;
        }

        // Inline `code` — strip when on a single line
        if c == '`' {
            // find matching backtick on same line
            let rest = &with_ignores[i + 1..];
            if let Some(end) = rest.find(['`', '\n']) {
                if rest.as_bytes()[end] == b'`' {
                    out.push(' ');
                    for _ in 0..end {
                        chars.next();
                    }
                    out.push_str(&" ".repeat(end));
                    chars.next(); // closing tick
                    out.push(' ');
                    continue;
                }
            }
        }

        out.push(c);
        let _ = fence_marker; // silence unused
    }

    out
}

/// Replace any text between `<!-- prosegrain:off -->` and
/// `<!-- prosegrain:on -->` with whitespace, byte-for-byte, preserving
/// line numbers.
fn strip_ignore_regions(input: &str) -> String {
    const OFF: &str = "<!-- prosegrain:off -->";
    const ON: &str = "<!-- prosegrain:on -->";
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let bytes = input.as_bytes();
    while i < bytes.len() {
        if input[i..].starts_with(OFF) {
            // copy the marker as-is so byte positions are unchanged
            out.push_str(OFF);
            i += OFF.len();
            // find ON
            if let Some(end_rel) = input[i..].find(ON) {
                let end = i + end_rel;
                for &b in &bytes[i..end] {
                    out.push(if b == b'\n' { '\n' } else { ' ' });
                }
                out.push_str(ON);
                i = end + ON.len();
            } else {
                // no closing marker — strip to end of input
                for &b in &bytes[i..] {
                    out.push(if b == b'\n' { '\n' } else { ' ' });
                }
                break;
            }
        } else {
            // copy one char
            let ch = input[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

fn is_fence_at(input: &str, i: usize, c: char) -> bool {
    let bytes = input.as_bytes();
    if i + 2 >= bytes.len() {
        return false;
    }
    bytes[i] == c as u8 && bytes[i + 1] == c as u8 && bytes[i + 2] == c as u8
}

/// Split text into sentences. Cheap heuristic: split on `[.!?]+` followed by
/// whitespace and an uppercase letter or end of input. Known limitation:
/// abbreviations like "Dr." or "e.g." may produce false splits. Acceptable
/// trade-off — over-splitting is harmless for our rules.
pub fn sentences(text: &str) -> Vec<Range<usize>> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i];
        if c == b'.' || c == b'!' || c == b'?' {
            // consume run of sentence-ending punctuation
            let mut j = i + 1;
            while j < bytes.len() && (bytes[j] == b'.' || bytes[j] == b'!' || bytes[j] == b'?') {
                j += 1;
            }
            // optional closing quote/bracket
            while j < bytes.len() && matches!(bytes[j], b'"' | b'\'' | b')' | b']') {
                j += 1;
            }
            // require whitespace after, then uppercase or end-of-input
            let ends_here = j >= bytes.len()
                || (bytes[j].is_ascii_whitespace() && following_is_upper_or_end(bytes, j));
            if ends_here {
                let end = j;
                if end > start {
                    out.push(start..end);
                }
                // advance past whitespace
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                start = j;
                i = j;
                continue;
            }
            // Every later byte of this run would reach the same `j` and the
            // same answer, so skip the run instead of rescanning it from each
            // byte (which was quadratic on a long run of dots).
            i = j;
            continue;
        }
        i += 1;
    }

    if start < bytes.len() {
        let trimmed_end = trim_trailing_ws(text, start, bytes.len());
        if trimmed_end > start {
            out.push(start..trimmed_end);
        }
    }
    out
}

fn following_is_upper_or_end(bytes: &[u8], from: usize) -> bool {
    let mut j = from;
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    if j >= bytes.len() {
        return true;
    }
    bytes[j].is_ascii_uppercase() || matches!(bytes[j], b'"' | b'\'' | b'_' | b'*')
}

fn trim_trailing_ws(text: &str, start: usize, end: usize) -> usize {
    let bytes = text.as_bytes();
    let mut e = end;
    while e > start && bytes[e - 1].is_ascii_whitespace() {
        e -= 1;
    }
    e
}

/// Split text into paragraphs by blank lines.
pub fn paragraphs(text: &str) -> Vec<Range<usize>> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\n' {
            // detect blank-line separator
            let mut j = i + 1;
            let mut newlines = 1;
            while j < bytes.len() && (bytes[j] == b'\n' || bytes[j] == b' ' || bytes[j] == b'\t') {
                if bytes[j] == b'\n' {
                    newlines += 1;
                }
                j += 1;
            }
            if newlines >= 2 {
                let end = trim_trailing_ws(text, start, i);
                if end > start {
                    out.push(start..end);
                }
                start = j;
                i = j;
                continue;
            }
        }
        i += 1;
    }
    if start < bytes.len() {
        let end = trim_trailing_ws(text, start, bytes.len());
        if end > start {
            out.push(start..end);
        }
    }
    out
}

/// Count words in a slice (whitespace-separated, alpha-containing tokens).
pub fn word_count(s: &str) -> usize {
    s.split_whitespace()
        .filter(|t| t.chars().any(|c| c.is_alphabetic()))
        .count()
}

/// Convert a byte offset to (line, column), both 1-indexed.
pub fn position(input: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, c) in input.char_indices() {
        if i >= offset {
            return (line, col);
        }
        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

/// Bytes per checkpoint in [`LineIndex`].
const CHAR_BLOCK: usize = 256;

/// Answers [`position`] queries without rescanning the input from byte 0.
///
/// Rules report one position per match, and calling [`position`] for each
/// one is quadratic on input with many matches. Build this once per input
/// and call [`LineIndex::position`] instead; it gives the same answers.
pub struct LineIndex<'a> {
    bytes: &'a [u8],
    /// Byte offset of every `\n`.
    newlines: Vec<usize>,
    /// `chars_at_block[k]` is the number of chars that start before byte
    /// `k * CHAR_BLOCK`.
    chars_at_block: Vec<usize>,
}

impl<'a> LineIndex<'a> {
    pub fn new(input: &'a str) -> Self {
        let bytes = input.as_bytes();
        let mut newlines = Vec::new();
        let mut chars_at_block = Vec::with_capacity(bytes.len() / CHAR_BLOCK + 2);
        let mut chars = 0usize;
        for (k, block) in bytes.chunks(CHAR_BLOCK).enumerate() {
            chars_at_block.push(chars);
            for (j, &b) in block.iter().enumerate() {
                if b == b'\n' {
                    newlines.push(k * CHAR_BLOCK + j);
                }
                if !is_utf8_continuation(b) {
                    chars += 1;
                }
            }
        }
        // Entry for an offset at the very end of the input.
        chars_at_block.push(chars);
        LineIndex {
            bytes,
            newlines,
            chars_at_block,
        }
    }

    /// Convert a byte offset to (line, column), both 1-indexed. Same result
    /// as [`position`] on the input this index was built from.
    pub fn position(&self, offset: usize) -> (usize, usize) {
        let offset = offset.min(self.bytes.len());
        let line = self.newlines.partition_point(|&nl| nl < offset);
        let line_start = if line == 0 {
            0
        } else {
            self.newlines[line - 1] + 1
        };
        let col = 1 + self.chars_before(offset) - self.chars_before(line_start);
        (line + 1, col)
    }

    /// Number of chars that start before byte `offset`.
    fn chars_before(&self, offset: usize) -> usize {
        let block = offset / CHAR_BLOCK;
        let from = block * CHAR_BLOCK;
        let partial = self.bytes[from..offset]
            .iter()
            .filter(|&&b| !is_utf8_continuation(b))
            .count();
        self.chars_at_block[block] + partial
    }
}

fn is_utf8_continuation(b: u8) -> bool {
    (b & 0xC0) == 0x80
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_index_matches_position() {
        let mut s = String::new();
        for i in 0..60 {
            s.push_str(&"é中x🙂".repeat(i % 7));
            s.push_str(["\n\n", "\n", "\n"][i % 3]);
            s.push_str(&"a".repeat(i));
        }
        for input in ["", "\n", "x", "ab\ncd\nef", s.as_str()] {
            let index = LineIndex::new(input);
            for offset in 0..input.len() + 3 {
                assert_eq!(
                    index.position(offset),
                    position(input, offset),
                    "offset {offset} of {} bytes",
                    input.len()
                );
            }
        }
    }

    #[test]
    fn sentences_unchanged_around_punctuation_runs() {
        let s = "Wait... what?! He said \"no.\" Then left.) ok. Fine... sure. End?!";
        let got: Vec<&str> = sentences(s).into_iter().map(|r| &s[r]).collect();
        let want = [
            "Wait... what?!",
            "He said \"no.\"",
            "Then left.) ok.",
            "Fine... sure.",
            "End?!",
        ];
        assert_eq!(got, want);
    }

    #[test]
    fn splits_simple_sentences() {
        let s = "First sentence. Second sentence. Third one!";
        let r = sentences(s);
        assert_eq!(r.len(), 3);
    }

    #[test]
    fn splits_paragraphs_on_blank_line() {
        let s = "One.\n\nTwo.\n\nThree.";
        assert_eq!(paragraphs(s).len(), 3);
    }

    #[test]
    fn position_tracks_line_and_column() {
        let s = "ab\ncd\nef";
        assert_eq!(position(s, 0), (1, 1));
        assert_eq!(position(s, 3), (2, 1));
        assert_eq!(position(s, 6), (3, 1));
    }

    #[test]
    fn strip_fenced_code() {
        let s = "before\n```\nlet x = 1;\n```\nafter";
        let out = strip_code_blocks(s);
        assert!(out.contains("before"));
        assert!(out.contains("after"));
        assert!(!out.contains("let x"));
    }
}
