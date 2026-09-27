//! Regression tests for hostile input. The file being linted is often written
//! by someone else, so it must not be able to drive the terminal or pin the
//! CPU of whoever runs prosegrain on it.

use prosegrain::{
    analyze,
    output::{render, Format},
    rules::Ruleset,
    score::Score,
    text::word_count,
};
use std::time::{Duration, Instant};

fn plain_report(input: &str) -> String {
    let findings = analyze(input, &Ruleset::default());
    let score = Score::from_findings(&findings, word_count(input));
    render("evil.md", &findings, &score, Format::Plain)
}

#[test]
fn plain_output_does_not_pass_terminal_controls_through() {
    // Clear screen, home, green fake verdict, then conceal everything after.
    let payload = "\x1b[2J\x1b[H\x1b[32mprosegrain clean grain 100 human\x1b[8m\r\x07";
    let s = format!("It's not just {payload}, it's fine.\n");
    let out = plain_report(&s);
    assert!(out.contains("not-but-construct"), "got {out:?}");
    for c in ['\x1b', '\r', '\x07'] {
        assert!(!out.contains(c), "raw {c:?} reached the terminal: {out:?}");
    }
}

fn assert_fast(label: &str, input: &str) {
    let start = Instant::now();
    let findings = analyze(input, &Ruleset::default());
    let took = start.elapsed();
    assert!(
        took < Duration::from_secs(20),
        "{label}: analyze() took {took:?} on {} bytes ({} findings)",
        input.len(),
        findings.len()
    );
}

#[test]
fn long_punctuation_run_is_not_quadratic() {
    assert_fast("dots", &format!("{} x", ".".repeat(400_000)));
}

#[test]
fn many_findings_on_one_line_are_not_quadratic() {
    assert_fast("delve", &"delve ".repeat(200_000));
}

#[test]
fn many_findings_over_many_lines_are_not_quadratic() {
    assert_fast("go", &"Go.\n".repeat(300_000));
}
