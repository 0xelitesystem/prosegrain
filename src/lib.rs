//! prosegrain — diagnostic linter for AI-generated prose patterns.
//!
//! Most "anti-slop" tools are word banlists. The valuable signal isn't in
//! vocabulary — it's in *structure* and *rhythm*. This crate detects:
//!
//!   - Banned phrases and words (the easy part)
//!   - Formulaic constructs ("not X, but Y", false agency, three-list cadence)
//!   - Rhythmic uniformity (sentence-length variance, paragraph evenness)
//!   - Throat-clearing openers and corporate hedges
//!
//! Public entry point is [`analyze`]. See [`Finding`] for output shape.

pub mod finding;
pub mod output;
pub mod rules;
pub mod score;
pub mod text;

pub use finding::{Finding, Severity};
pub use score::Score;

use rules::Ruleset;

/// Run all enabled rules over `input` and return findings in source order.
pub fn analyze(input: &str, ruleset: &Ruleset) -> Vec<Finding> {
    let stripped = text::strip_code_blocks(input);
    let mut findings = Vec::new();

    if ruleset.words {
        findings.extend(rules::words::check(&stripped));
    }
    if ruleset.structure {
        findings.extend(rules::structure::check(&stripped));
    }
    if ruleset.rhythm {
        findings.extend(rules::rhythm::check(&stripped));
    }

    findings.sort_by_key(|f| (f.line, f.column));
    findings
}
