//! Structural patterns. These are the harder, more reliable AI tells —
//! the ones a "scrub for delve and tapestry" pass leaves behind.
//!
//! Rules implemented:
//!
//!   - `not-but-construct`: "It's not just X, it's Y" / "Not X. Y."
//!   - `false-agency`: inanimate nouns given human verbs
//!     ("the data tells us", "the market rewards").
//!   - `triple-list-cadence`: high density of "X, Y, and Z" patterns.
//!   - `negative-listing`: "Not X. Not Y. Not Z." three-beat negation.
//!   - `em-dash-overuse`: em-dash density above ~2 per 100 words.
//!   - `triple-parallel-opening`: three consecutive sentences with
//!     near-identical opening structure.

use crate::finding::{Finding, Severity};
use crate::text::{paragraphs, position, sentences, word_count};
use regex::Regex;
use std::sync::OnceLock;

pub fn check(input: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(not_but_construct(input));
    findings.extend(false_agency(input));
    findings.extend(triple_list_cadence(input));
    findings.extend(negative_listing(input));
    findings.extend(em_dash_overuse(input));
    findings.extend(triple_parallel_opening(input));
    findings
}

// ─────────────────────────────────────────────────────────────────────────
// not-but-construct
// "It's not just X, it's Y." — one of the most overused AI rhetorical shapes.
// ─────────────────────────────────────────────────────────────────────────

fn not_but_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(
            r"(?i)\b(?:it'?s|this is|that'?s|we'?re|you'?re|i'?m)\s+not\s+just\s+[^.,;:]{1,60}[.,;:]?\s*(?:it'?s|that'?s|but|—)",
        )
        .expect("not-but regex")
    })
}

fn not_but_alt_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?i)\bnot\s+(?:just|merely|only|simply)\s+\w[\w\s]{0,40}?[,—-]\s*but\b")
            .expect("not-but-alt regex")
    })
}

fn not_but_construct(input: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for re in [not_but_re(), not_but_alt_re()] {
        for m in re.find_iter(input) {
            let (line, column) = position(input, m.start());
            out.push(Finding {
                rule: "not-but-construct",
                severity: Severity::Strong,
                line,
                column,
                snippet: m.as_str().to_string(),
                message:
                    "\"Not just X, it's Y\" is one of the most overused AI shapes. Pick one claim and state it directly."
                        .to_string(),
            });
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────
// false-agency
// AI prose loves giving inanimate nouns human verbs to avoid naming an
// actor. "The data tells us" → who is "us"? Who looked at the data?
// ─────────────────────────────────────────────────────────────────────────

const INANIMATE_SUBJECTS: &[&str] = &[
    "the data",
    "the numbers",
    "the chart",
    "the graph",
    "the market",
    "the economy",
    "the algorithm",
    "the model",
    "the code",
    "the system",
    "the platform",
    "the technology",
    "the research",
    "the study",
    "the report",
    "the literature",
    "the evidence",
    "the trend",
    "the trends",
    "history",
];

const HUMAN_VERBS: &[&str] = &[
    "tells",
    "shows",
    "reveals",
    "demonstrates",
    "suggests",
    "wants",
    "demands",
    "rewards",
    "punishes",
    "decides",
    "thinks",
    "believes",
    "knows",
    "understands",
    "agrees",
    "argues",
    "claims",
    "insists",
    "warns",
    "promises",
];

fn false_agency_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        let subj = INANIMATE_SUBJECTS.join("|");
        let verb = HUMAN_VERBS.join("|");
        let pat = format!(r"(?i)\b({subj})\s+({verb})\b");
        Regex::new(&pat).expect("false-agency regex")
    })
}

fn false_agency(input: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for m in false_agency_re().find_iter(input) {
        let (line, column) = position(input, m.start());
        out.push(Finding {
            rule: "false-agency",
            severity: Severity::Warn,
            line,
            column,
            snippet: m.as_str().to_string(),
            message:
                "Inanimate subject given a human verb. Name the actual person or group doing the thing."
                    .to_string(),
        });
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────
// triple-list-cadence
// Heavy use of "X, Y, and Z" lists in close succession is an AI rhythm.
// Flag paragraphs with ≥3 such lists.
// ─────────────────────────────────────────────────────────────────────────

fn triple_list_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        // word, word, (and|or) word — basic three-item list
        Regex::new(r"\b\w+,\s+\w+,\s+(?:and|or)\s+\w+\b").expect("triple-list regex")
    })
}

fn triple_list_cadence(input: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    for para in paragraphs(input) {
        let slice = &input[para.clone()];
        let count = triple_list_re().find_iter(slice).count();
        if count >= 3 {
            let (line, column) = position(input, para.start);
            out.push(Finding {
                rule: "triple-list-cadence",
                severity: Severity::Warn,
                line,
                column,
                snippet: format!("{count} three-item lists in one paragraph"),
                message:
                    "Three-item lists piled up in one paragraph create AI cadence. Vary the rhythm."
                        .to_string(),
            });
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────
// negative-listing
// "Not X. Not Y. Not Z." — short negations in three-beat parallel.
// ─────────────────────────────────────────────────────────────────────────

fn negative_listing(input: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    let sents = sentences(input);
    let starts_with_not = |r: &std::ops::Range<usize>| {
        let s = input[r.clone()].trim_start();
        s.starts_with("Not ") || s.starts_with("not ")
    };
    let mut run_start: Option<std::ops::Range<usize>> = None;
    let mut run_len = 0usize;
    for r in &sents {
        if starts_with_not(r) {
            if run_len == 0 {
                run_start = Some(r.clone());
            }
            run_len += 1;
        } else {
            if run_len >= 3 {
                if let Some(start) = &run_start {
                    let (line, column) = position(input, start.start);
                    out.push(Finding {
                        rule: "negative-listing",
                        severity: Severity::Warn,
                        line,
                        column,
                        snippet: format!("{run_len} consecutive sentences begin with \"Not\""),
                        message:
                            "Three-beat negation is an AI rhetorical reflex. Keep one negation; cut the rest."
                                .to_string(),
                    });
                }
            }
            run_len = 0;
            run_start = None;
        }
    }
    if run_len >= 3 {
        if let Some(start) = &run_start {
            let (line, column) = position(input, start.start);
            out.push(Finding {
                rule: "negative-listing",
                severity: Severity::Warn,
                line,
                column,
                snippet: format!("{run_len} consecutive sentences begin with \"Not\""),
                message:
                    "Three-beat negation is an AI rhetorical reflex. Keep one negation; cut the rest."
                        .to_string(),
            });
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────
// em-dash-overuse
// AI-generated prose uses em-dashes at much higher density than typical
// human writing. Threshold: more than 2 em-dashes per 100 words.
// ─────────────────────────────────────────────────────────────────────────

fn em_dash_overuse(input: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    let dashes = input.matches('—').count();
    let words = word_count(input);
    if words >= 100 {
        let per_100 = (dashes as f64) / (words as f64 / 100.0);
        if per_100 > 2.0 {
            // find first em-dash for position
            if let Some(idx) = input.find('—') {
                let (line, column) = position(input, idx);
                out.push(Finding {
                    rule: "em-dash-overuse",
                    severity: Severity::Warn,
                    line,
                    column,
                    snippet: format!("{dashes} em-dashes in {words} words ({per_100:.1} per 100)"),
                    message:
                        "Em-dash density above ~2 per 100 words is a strong AI tell. Replace some with periods, commas, or parens."
                            .to_string(),
                });
            }
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────
// triple-parallel-opening
// Three consecutive sentences starting with the same word/structure.
// e.g. "It enables X. It enables Y. It enables Z."
// ─────────────────────────────────────────────────────────────────────────

fn first_word_lower(s: &str) -> Option<String> {
    s.split_whitespace()
        .next()
        .map(|w| w.trim_matches(|c: char| !c.is_alphabetic()).to_lowercase())
        .filter(|w| !w.is_empty())
}

fn triple_parallel_opening(input: &str) -> Vec<Finding> {
    let sents = sentences(input);
    let mut out = Vec::new();
    if sents.len() < 3 {
        return out;
    }
    let mut i = 0;
    while i + 2 < sents.len() {
        let a = first_word_lower(&input[sents[i].clone()]);
        let b = first_word_lower(&input[sents[i + 1].clone()]);
        let c = first_word_lower(&input[sents[i + 2].clone()]);
        if let (Some(a), Some(b), Some(c)) = (a, b, c) {
            // skip articles/conjunctions that are common honest openers
            let skip = matches!(a.as_str(), "the" | "a" | "an" | "and" | "but" | "or" | "so");
            if !skip && a == b && b == c && a.len() > 1 {
                let (line, column) = position(input, sents[i].start);
                out.push(Finding {
                    rule: "triple-parallel-opening",
                    severity: Severity::Warn,
                    line,
                    column,
                    snippet: format!(
                        "three sentences in a row start with \"{}\"",
                        a
                    ),
                    message:
                        "Anaphora across three sentences is an AI cadence. Vary at least one opener."
                            .to_string(),
                });
                i += 3;
                continue;
            }
        }
        i += 1;
    }
    out
}
