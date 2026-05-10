//! Rhythmic uniformity. Human writing varies. AI prose tends toward
//! medium-length sentences in medium-length paragraphs, packed evenly.
//! We compute coefficient of variation (CV) and flag low values.

use crate::finding::{Finding, Severity};
use crate::text::{paragraphs, position, sentences, word_count};

pub fn check(input: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    out.extend(sentence_length_uniformity(input));
    out.extend(paragraph_length_uniformity(input));
    out.extend(long_sentence_average(input));
    out
}

fn coefficient_of_variation(samples: &[usize]) -> Option<f64> {
    if samples.len() < 4 {
        return None;
    }
    let n = samples.len() as f64;
    let mean = samples.iter().copied().map(|x| x as f64).sum::<f64>() / n;
    if mean <= 0.0 {
        return None;
    }
    let var = samples
        .iter()
        .copied()
        .map(|x| {
            let d = x as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / n;
    let sd = var.sqrt();
    Some(sd / mean)
}

fn sentence_length_uniformity(input: &str) -> Vec<Finding> {
    let sents = sentences(input);
    let lengths: Vec<usize> = sents
        .iter()
        .map(|r| word_count(&input[r.clone()]))
        .collect();
    if let Some(cv) = coefficient_of_variation(&lengths) {
        // CV < 0.4 means sentence lengths are unusually uniform.
        if cv < 0.40 && lengths.len() >= 6 {
            let (line, column) = sents
                .first()
                .map(|r| position(input, r.start))
                .unwrap_or((1, 1));
            return vec![Finding {
                rule: "uniform-sentence-length",
                severity: Severity::Warn,
                line,
                column,
                snippet: format!(
                    "{} sentences, length CV {:.2} (low variance)",
                    lengths.len(),
                    cv
                ),
                message:
                    "Sentence lengths are too even. Human prose mixes short punches with longer flows; revise for variance."
                        .to_string(),
            }];
        }
    }
    Vec::new()
}

fn paragraph_length_uniformity(input: &str) -> Vec<Finding> {
    let paras = paragraphs(input);
    let lengths: Vec<usize> = paras
        .iter()
        .map(|r| word_count(&input[r.clone()]))
        .collect();
    if let Some(cv) = coefficient_of_variation(&lengths) {
        if cv < 0.30 && lengths.len() >= 4 {
            let (line, column) = paras
                .first()
                .map(|r| position(input, r.start))
                .unwrap_or((1, 1));
            return vec![Finding {
                rule: "uniform-paragraph-length",
                severity: Severity::Warn,
                line,
                column,
                snippet: format!(
                    "{} paragraphs, length CV {:.2} (low variance)",
                    lengths.len(),
                    cv
                ),
                message:
                    "Paragraph lengths are uniform. Vary them — short paragraphs land hard, long ones build."
                        .to_string(),
            }];
        }
    }
    Vec::new()
}

fn long_sentence_average(input: &str) -> Vec<Finding> {
    let sents = sentences(input);
    if sents.len() < 5 {
        return Vec::new();
    }
    let total: usize = sents.iter().map(|r| word_count(&input[r.clone()])).sum();
    let avg = total as f64 / sents.len() as f64;
    if avg > 28.0 {
        let (line, column) = sents
            .first()
            .map(|r| position(input, r.start))
            .unwrap_or((1, 1));
        return vec![Finding {
            rule: "long-average-sentence",
            severity: Severity::Hint,
            line,
            column,
            snippet: format!("avg sentence length {:.1} words", avg),
            message:
                "Average sentence is long. Drop in some short ones — they reset the reader's attention."
                    .to_string(),
        }];
    }
    Vec::new()
}
