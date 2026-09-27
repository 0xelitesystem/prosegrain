use crate::finding::{Finding, Severity};
use crate::score::Score;
use serde::Serialize;

#[derive(Debug, Clone, Copy)]
pub enum Format {
    Plain,
    Json,
}

#[derive(Serialize)]
struct JsonReport<'a> {
    file: &'a str,
    score: Score,
    findings: &'a [Finding],
}

pub fn render(file: &str, findings: &[Finding], score: &Score, format: Format) -> String {
    match format {
        Format::Plain => render_plain(file, findings, score),
        Format::Json => {
            let report = JsonReport {
                file,
                score: score.clone(),
                findings,
            };
            serde_json::to_string_pretty(&report).unwrap_or_default()
        }
    }
}

fn render_plain(file: &str, findings: &[Finding], score: &Score) -> String {
    let file = sanitize_for_terminal(file);
    let mut out = String::new();
    if findings.is_empty() {
        out.push_str(&format!(
            "{file}: clean — grain {} ({})\n",
            score.grain,
            score.verdict()
        ));
        return out;
    }
    for f in findings {
        let tag = match f.severity {
            Severity::Hint => "hint  ",
            Severity::Warn => "warn  ",
            Severity::Strong => "strong",
        };
        out.push_str(&format!(
            "{file}:{line}:{col}  {tag}  {rule}\n  {msg}\n  > {snip}\n\n",
            file = file,
            line = f.line,
            col = f.column,
            tag = tag,
            rule = f.rule,
            msg = sanitize_for_terminal(&f.message),
            snip = sanitize_for_terminal(&truncate(&f.snippet, 100)),
        ));
    }
    out.push_str(&format!(
        "{n} finding(s)  |  grain {grain}/100  |  verdict: {verdict}  |  hint: {h}  warn: {w}  strong: {s}\n",
        n = findings.len(),
        grain = score.grain,
        verdict = score.verdict(),
        h = score.by_severity.hint,
        w = score.by_severity.warn,
        s = score.by_severity.strong,
    ));
    out
}

fn truncate(s: &str, max: usize) -> String {
    let mut out: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        out.push('…');
    }
    out
}

/// Make text from the linted file safe to print to a terminal. That text is
/// often written by someone else (a pull request, pasted model output), and a
/// raw ESC or CR in a snippet would let its author clear the screen, move the
/// cursor or hide the real verdict. C0 controls other than newline and tab,
/// DEL, C1 controls and bidi controls are shown as a visible `\u{..}` escape.
/// Everything else passes through unchanged.
fn sanitize_for_terminal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if is_unsafe_for_terminal(c) {
            out.extend(c.escape_unicode());
        } else {
            out.push(c);
        }
    }
    out
}

fn is_unsafe_for_terminal(c: char) -> bool {
    let bidi = matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}')
        || ('\u{202a}'..='\u{202e}').contains(&c)
        || ('\u{2066}'..='\u{2069}').contains(&c);
    // `is_control` covers C0, DEL and C1.
    (c.is_control() && c != '\n' && c != '\t') || bidi
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_finding(snippet: &str) -> Vec<Finding> {
        vec![Finding {
            rule: "not-but-construct",
            severity: Severity::Strong,
            line: 1,
            column: 1,
            snippet: snippet.to_string(),
            message: "msg".to_string(),
        }]
    }

    #[test]
    fn plain_output_escapes_terminal_controls_in_snippets() {
        let evil = "It's not just \x1b[2J\x1b[H\x1b[32mclean\x1b[8m\r\x07\x7f\u{9b}2J, it's";
        let bidi = "a\u{202e}b\u{2066}c\u{2069}d\u{200e}e\u{200f}f\u{061c}g\u{202a}h";
        let unsafe_controls = "\x1b\r\x07\x7f\u{9b}";
        let unsafe_bidi = "\u{202e}\u{2066}\u{2069}\u{200e}\u{200f}\u{061c}\u{202a}";
        for snippet in [evil, bidi] {
            let findings = one_finding(snippet);
            let score = Score::from_findings(&findings, 10);
            let out = render("evil\x1b[2J.md", &findings, &score, Format::Plain);
            for c in unsafe_controls.chars().chain(unsafe_bidi.chars()) {
                assert!(
                    !out.contains(c),
                    "unsafe char {c:?} reached plain output: {out:?}"
                );
            }
        }
        let findings = one_finding(evil);
        let score = Score::from_findings(&findings, 10);
        let out = render("evil.md", &findings, &score, Format::Plain);
        assert!(
            out.contains("\\u{1b}[2J"),
            "ESC should be shown escaped: {out:?}"
        );
    }

    #[test]
    fn plain_output_is_unchanged_for_ordinary_text() {
        let snippet = "It's not just about\tproductivity,\nit's “quoted” café";
        assert_eq!(sanitize_for_terminal(snippet), snippet);
        let findings = one_finding(snippet);
        let score = Score::from_findings(&findings, 10);
        let out = render("notes.md", &findings, &score, Format::Plain);
        assert!(out.contains(&format!("  > {snippet}\n")), "got {out:?}");
    }
}
