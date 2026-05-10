use serde::Serialize;

/// Severity of a finding. Used for filtering and scoring weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Stylistic preference; not a strong AI tell on its own.
    Hint,
    /// Common AI pattern; usually worth revising.
    Warn,
    /// Strong AI tell; almost always reads as machine-written.
    Strong,
}

impl Severity {
    pub fn weight(self) -> u32 {
        match self {
            Severity::Hint => 1,
            Severity::Warn => 3,
            Severity::Strong => 6,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Severity::Hint => "hint",
            Severity::Warn => "warn",
            Severity::Strong => "strong",
        }
    }
}

/// One issue detected by a rule.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Stable rule identifier, e.g. `not-but-construct`.
    pub rule: &'static str,
    /// Severity tier.
    pub severity: Severity,
    /// 1-indexed line number.
    pub line: usize,
    /// 1-indexed column.
    pub column: usize,
    /// The text fragment that matched (for context in output).
    pub snippet: String,
    /// Why this is flagged. Plain language, no jargon.
    pub message: String,
}
