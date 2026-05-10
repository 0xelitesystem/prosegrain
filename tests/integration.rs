use prosegrain::{analyze, rules::Ruleset};

fn rules() -> Ruleset {
    Ruleset::default()
}

fn rule_ids(input: &str) -> Vec<&'static str> {
    analyze(input, &rules())
        .into_iter()
        .map(|f| f.rule)
        .collect()
}

#[test]
fn detects_not_but_construct() {
    let s = "It's not just a tool, it's a movement.";
    let ids = rule_ids(s);
    assert!(ids.contains(&"not-but-construct"), "got {ids:?}");
}

#[test]
fn detects_false_agency() {
    let s = "The data tells us that growth is slowing.";
    let ids = rule_ids(s);
    assert!(ids.contains(&"false-agency"), "got {ids:?}");
}

#[test]
fn detects_assistant_opener() {
    let s = "Certainly! Here is the answer you asked for.";
    let ids = rule_ids(s);
    assert!(ids.contains(&"assistant-opener"), "got {ids:?}");
}

#[test]
fn detects_filler_phrase() {
    let s = "In today's world, productivity matters.";
    let ids = rule_ids(s);
    assert!(ids.contains(&"filler-phrase"), "got {ids:?}");
}

#[test]
fn detects_banned_word() {
    let s = "We should delve into the details.";
    let ids = rule_ids(s);
    assert!(ids.contains(&"banned-word"), "got {ids:?}");
}

#[test]
fn detects_negative_listing() {
    let s =
        "Not every team needs this. Not every workflow benefits. Not every founder will see it.";
    let ids = rule_ids(s);
    assert!(ids.contains(&"negative-listing"), "got {ids:?}");
}

#[test]
fn detects_triple_parallel_opening() {
    let s = "They unlock potential. They harness data. They navigate change.";
    let ids = rule_ids(s);
    assert!(ids.contains(&"triple-parallel-opening"), "got {ids:?}");
}

#[test]
fn clean_human_prose_has_zero_findings() {
    let s = "I was 14. My older cousin had a copy of a Python book on his desk, \
             and I borrowed it for a weekend that turned into a year. The book \
             was awful. By the time I finished it, I had built two small games \
             and a script that scraped my school's lunch menu. That last one is \
             what hooked me. I had never made a computer do something useful for \
             me before.";
    let findings = analyze(s, &rules());
    assert_eq!(findings.len(), 0, "unexpected findings: {findings:?}");
}

#[test]
fn code_blocks_are_not_linted() {
    let s = "Real prose here.\n\n```\nlet delve = robust();  // would otherwise be flagged\n```\n\nMore prose.";
    let findings = analyze(s, &rules());
    for f in &findings {
        assert!(
            !f.snippet.contains("delve") && !f.snippet.contains("robust"),
            "code block leaked through: {:?}",
            f
        );
    }
}

#[test]
fn rulesets_can_be_disabled() {
    let s = "It's not just X, it's Y. Delve into this.";
    let no_words = Ruleset {
        words: false,
        structure: true,
        rhythm: true,
    };
    let ids: Vec<_> = analyze(s, &no_words).iter().map(|f| f.rule).collect();
    assert!(!ids.contains(&"banned-word"));
    assert!(ids.contains(&"not-but-construct"));
}

#[test]
fn ignore_regions_suppress_findings() {
    let s = "Real prose here.\n\n\
             <!-- prosegrain:off -->\n\
             It's not just X, it's Y. Delve into this. The data tells us.\n\
             <!-- prosegrain:on -->\n\n\
             More real prose.";
    let findings = analyze(s, &rules());
    for f in &findings {
        assert!(
            !f.snippet.to_lowercase().contains("delve")
                && !f.snippet.contains("not just")
                && !f.snippet.contains("data tells"),
            "ignored region leaked: {f:?}"
        );
    }
}
