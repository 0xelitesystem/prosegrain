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
            msg = f.message,
            snip = truncate(&f.snippet, 100),
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
