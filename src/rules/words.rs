//! Banned vocabulary and filler phrases.
//!
//! These are the well-known AI tells. Word lists alone don't make a strong
//! signal — `delve` shows up in human writing too — but they're cheap and
//! useful as part of a wider analysis. Severity is calibrated low (`Hint`)
//! for individual words and higher (`Warn`) for multi-word filler phrases
//! that almost never appear in unselfconscious human prose.

use crate::finding::{Finding, Severity};
use crate::text::position;
use regex::Regex;
use std::sync::OnceLock;

/// Single overused words. Hint severity — flagged but not strongly weighted.
const BANNED_WORDS: &[&str] = &[
    "delve",
    "delves",
    "delving",
    "tapestry",
    "leverage",
    "leveraging",
    "navigate",
    "navigating", // metaphor sense; we accept false positives
    "seamless",
    "seamlessly",
    "robust",
    "revolutionize",
    "revolutionized",
    "revolutionary",
    "paradigm",
    "ever-evolving",
    "ever-changing",
    "harness",
    "harnessing",
    "embark",
    "foster",
    "fostering",
    "pivotal",
    "crucial",
    "crucially",
    "multifaceted",
    "holistic",
    "elevate",
    "elevating",
    "amplify",
    "streamline",
    "streamlining",
    "empower",
    "empowering",
    "intricate",
    "ecosystem",
    "landscape",
    "realm",
    "nuanced",
    "transformative",
    "cutting-edge",
    "state-of-the-art",
    "game-changer",
    "game-changing",
    "underscore",
    "underscores",
    "myriad",
    "plethora",
    "commendable",
    "meticulous",
    "meticulously",
];

/// Multi-word filler phrases. Warn severity — these are very strong tells.
const FILLER_PHRASES: &[&str] = &[
    r"in today'?s (?:fast-paced |digital |modern )?world",
    r"in conclusion",
    r"it'?s important to note",
    r"it is important to note",
    r"let'?s (?:explore|delve|dive)",
    r"when it comes to",
    r"needless to say",
    r"at the end of the day",
    r"in essence",
    r"in the realm of",
    r"in the world of",
    r"the key takeaway",
    r"moving forward",
    r"the importance of \w+ cannot be overstated",
    r"plays? a (?:vital|crucial|key|pivotal) role",
    r"a testament to",
    r"the fast-paced world of",
    r"navigate the complexities",
    r"unlock(?:s|ing)? the (?:potential|power|secrets)",
    r"in the ever-(?:evolving|changing) (?:world|landscape) of",
];

/// Throat-clearing assistant openers. Strong tells in published text.
const ASSISTANT_OPENERS: &[&str] = &[
    r"(?m)^\s*(?:Certainly|Absolutely|Sure|Of course)[!,.]",
    r"(?m)^\s*I(?:'| a)m happy to help",
    r"(?m)^\s*I(?:'| wi)ll (?:break|walk) (?:this|that|it) down",
    r"(?m)^\s*Great (?:question|point)[!,.]",
    r"(?m)^\s*Let me (?:break this down|explain)",
];

fn word_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        let pattern = format!(r"(?i)\b({})\b", BANNED_WORDS.join("|").replace('-', r"\-"));
        Regex::new(&pattern).expect("banned-words regex")
    })
}

fn filler_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        let pattern = format!(r"(?i)\b({})\b", FILLER_PHRASES.join("|"));
        Regex::new(&pattern).expect("filler-phrases regex")
    })
}

fn opener_regexes() -> &'static [Regex] {
    static R: OnceLock<Vec<Regex>> = OnceLock::new();
    R.get_or_init(|| {
        ASSISTANT_OPENERS
            .iter()
            .map(|p| Regex::new(p).expect("opener regex"))
            .collect()
    })
}

pub fn check(input: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for m in word_regex().find_iter(input) {
        let (line, column) = position(input, m.start());
        findings.push(Finding {
            rule: "banned-word",
            severity: Severity::Hint,
            line,
            column,
            snippet: m.as_str().to_string(),
            message: format!(
                "`{}` is a common AI overuse word. If a plainer word fits, use it.",
                m.as_str()
            ),
        });
    }

    for m in filler_regex().find_iter(input) {
        let (line, column) = position(input, m.start());
        findings.push(Finding {
            rule: "filler-phrase",
            severity: Severity::Warn,
            line,
            column,
            snippet: m.as_str().to_string(),
            message: "Throat-clearing filler. Cut it; the sentence is stronger without it."
                .to_string(),
        });
    }

    for re in opener_regexes() {
        for m in re.find_iter(input) {
            let (line, column) = position(input, m.start());
            findings.push(Finding {
                rule: "assistant-opener",
                severity: Severity::Strong,
                line,
                column,
                snippet: m.as_str().trim().to_string(),
                message: "Reads like a chatbot. Open with the actual content instead.".to_string(),
            });
        }
    }

    findings
}
