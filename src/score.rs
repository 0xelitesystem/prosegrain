use crate::finding::Finding;
use serde::Serialize;

/// Aggregate score for a piece of text.
///
/// `grain` is a 0..100 quality metric. Higher = more human texture.
/// 100 means no findings; 0 means dense AI patterns.
#[derive(Debug, Clone, Serialize)]
pub struct Score {
    pub grain: u32,
    pub findings: usize,
    pub by_severity: ByCounts,
}

#[derive(Debug, Clone, Serialize)]
pub struct ByCounts {
    pub hint: usize,
    pub warn: usize,
    pub strong: usize,
}

impl Score {
    pub fn from_findings(findings: &[Finding], word_count: usize) -> Self {
        let mut counts = ByCounts {
            hint: 0,
            warn: 0,
            strong: 0,
        };
        let mut weight: u32 = 0;
        for f in findings {
            weight += f.severity.weight();
            match f.severity {
                crate::finding::Severity::Hint => counts.hint += 1,
                crate::finding::Severity::Warn => counts.warn += 1,
                crate::finding::Severity::Strong => counts.strong += 1,
            }
        }
        // Normalize by length so a 200-word piece with 5 findings doesn't
        // tie a 5000-word piece with 5 findings.
        let words_per_100 = (word_count.max(1) as f64) / 100.0;
        let density = weight as f64 / words_per_100.max(1.0);
        let grain = (100.0 - density.min(100.0)).max(0.0) as u32;
        Score {
            grain,
            findings: findings.len(),
            by_severity: counts,
        }
    }

    pub fn verdict(&self) -> &'static str {
        match self.grain {
            85..=100 => "human",
            65..=84 => "mostly-human",
            40..=64 => "ai-leaning",
            _ => "ai-heavy",
        }
    }
}
