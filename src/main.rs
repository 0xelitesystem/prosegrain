use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;
use prosegrain::{
    analyze,
    output::{render, Format},
    rules::Ruleset,
    score::Score,
    text::word_count,
};

/// Diagnostic linter for AI-generated prose patterns.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// File to lint. Use `-` to read from stdin.
    file: Option<PathBuf>,

    /// Output as JSON.
    #[arg(long)]
    json: bool,

    /// Print only the grain score (0..100) and exit 0.
    #[arg(long)]
    score: bool,

    /// Disable word/phrase rules.
    #[arg(long)]
    no_words: bool,

    /// Disable structural rules.
    #[arg(long)]
    no_structure: bool,

    /// Disable rhythm rules.
    #[arg(long)]
    no_rhythm: bool,

    /// Exit with non-zero code if any `strong`-severity finding is present.
    /// Useful in CI.
    #[arg(long)]
    strict: bool,

    /// List all available rules and exit.
    #[arg(long)]
    list_rules: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if cli.list_rules {
        print_rules();
        return ExitCode::SUCCESS;
    }

    let (label, contents) = match cli.file.as_deref() {
        Some(p) if p.as_os_str() == "-" => match read_stdin() {
            Ok(s) => ("<stdin>".to_string(), s),
            Err(e) => {
                eprintln!("prosegrain: failed to read stdin: {e}");
                return ExitCode::from(2);
            }
        },
        Some(p) => match fs::read_to_string(p) {
            Ok(s) => (p.display().to_string(), s),
            Err(e) => {
                eprintln!("prosegrain: cannot read {}: {e}", p.display());
                return ExitCode::from(2);
            }
        },
        None => {
            eprintln!("prosegrain: no file given. Pass a path or `-` for stdin.");
            eprintln!("Try `prosegrain --help`.");
            return ExitCode::from(2);
        }
    };

    let ruleset = Ruleset {
        words: !cli.no_words,
        structure: !cli.no_structure,
        rhythm: !cli.no_rhythm,
    };

    let findings = analyze(&contents, &ruleset);
    let words = word_count(&contents);
    let score = Score::from_findings(&findings, words);

    if cli.score {
        println!("{}", score.grain);
        return ExitCode::SUCCESS;
    }

    let format = if cli.json {
        Format::Json
    } else {
        Format::Plain
    };
    print!("{}", render(&label, &findings, &score, format));

    if cli.strict && score.by_severity.strong > 0 {
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn read_stdin() -> io::Result<String> {
    let mut s = String::new();
    io::stdin().read_to_string(&mut s)?;
    Ok(s)
}

fn print_rules() {
    let rules: &[(&str, &str, &str)] = &[
        (
            "banned-word",
            "hint",
            "Single overused AI-favorite words (delve, tapestry, robust, …).",
        ),
        (
            "filler-phrase",
            "warn",
            "Multi-word filler (in conclusion, when it comes to, in today's world).",
        ),
        (
            "assistant-opener",
            "strong",
            "Chatbot-style openers (Certainly!, Absolutely!, I'm happy to help).",
        ),
        (
            "not-but-construct",
            "strong",
            "\"It's not just X, it's Y\" — overused AI rhetorical shape.",
        ),
        (
            "false-agency",
            "warn",
            "Inanimate subject + human verb (the data tells us, the market rewards).",
        ),
        (
            "triple-list-cadence",
            "warn",
            "Three or more \"X, Y, and Z\" lists in one paragraph.",
        ),
        (
            "negative-listing",
            "warn",
            "Three+ consecutive sentences starting with \"Not\".",
        ),
        (
            "em-dash-overuse",
            "warn",
            "Em-dash density above ~2 per 100 words.",
        ),
        (
            "triple-parallel-opening",
            "warn",
            "Three consecutive sentences starting with the same word.",
        ),
        (
            "uniform-sentence-length",
            "warn",
            "Sentence lengths too even (low coefficient of variation).",
        ),
        (
            "uniform-paragraph-length",
            "warn",
            "Paragraph lengths too even.",
        ),
        (
            "long-average-sentence",
            "hint",
            "Average sentence length over 28 words.",
        ),
    ];
    println!("rule                          severity  description");
    println!("----                          --------  -----------");
    for (id, sev, desc) in rules {
        println!("{id:<29} {sev:<9} {desc}");
    }
}
