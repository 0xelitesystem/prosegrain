//! Rule families.
//!
//! - [`words`]: banned vocabulary and filler phrases (the easy tier).
//! - [`structure`]: formulaic constructs ("not X, but Y", false agency,
//!   throat-clearing openers, three-list cadence). The differentiator.
//! - [`rhythm`]: sentence-length variance and paragraph uniformity.

pub mod rhythm;
pub mod structure;
pub mod words;

/// Toggle which rule families run.
#[derive(Debug, Clone, Copy)]
pub struct Ruleset {
    pub words: bool,
    pub structure: bool,
    pub rhythm: bool,
}

impl Default for Ruleset {
    fn default() -> Self {
        Self {
            words: true,
            structure: true,
            rhythm: true,
        }
    }
}
