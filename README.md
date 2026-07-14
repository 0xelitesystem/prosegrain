# prosegrain

A linter for AI-generated prose patterns. Catches the patterns word banlists miss: structural cadence, false agency, rhythmic uniformity, and the "not just X, it's Y" reflex.

Most existing anti-slop tools are skill files or system prompts, they tell an LLM how to write at generation time. They're useless on text that's already written. `prosegrain` runs against any text file you give it. No LLM call. No account. One binary.

## What it catches

<!-- prosegrain:off -->

| Rule | Severity | What it flags |
|------|----------|---------------|
| `assistant-opener` | strong | "Certainly!", "Absolutely!", "I'm happy to help" |
| `not-but-construct` | strong | "It's not just X, it's Y", overused AI shape |
| `false-agency` | warn | "The data tells us", "the market rewards" |
| `filler-phrase` | warn | "In today's world", "in conclusion", "when it comes to" |
| `triple-list-cadence` | warn | Three or more "X, Y, and Z" lists in one paragraph |
| `negative-listing` | warn | Three+ sentences in a row starting with "Not" |
| `triple-parallel-opening` | warn | Three sentences in a row starting with the same word |
| `em-dash-overuse` | warn | More than ~2 em-dashes per 100 words |
| `uniform-sentence-length` | warn | Sentence-length variance too low |
| `uniform-paragraph-length` | warn | Paragraph-length variance too low |
| `long-average-sentence` | hint | Average sentence over 28 words |
| `banned-word` | hint | `delve`, `tapestry`, `robust`, `paradigm`, etc. |

<!-- prosegrain:on -->

Code blocks are stripped before analysis, so fenced code in markdown won't trigger false positives.

## Install

From source (requires Rust 1.75+):

```
cargo install --path .
```

Or build the binary directly:

```
cargo build --release
# binary at target/release/prosegrain
```

Pre-built binaries will be attached to GitHub releases once we cut v0.1.0.

## Usage

```
prosegrain article.md
```

Pipe from stdin:

```
cat draft.md | prosegrain -
```

JSON output for CI or scripts:

```
prosegrain article.md --json
```

Just the score:

```
prosegrain article.md --score
# prints a number 0..100
```

Fail a CI step if any `strong`-severity finding shows up:

```
prosegrain article.md --strict
```

List all rules:

```
prosegrain --list-rules
```

Disable a rule family:

```
prosegrain article.md --no-rhythm
prosegrain article.md --no-words --no-rhythm   # structure-only
```

## Example

Run against the included slop sample:

```
$ prosegrain examples/slop-sample.md
examples/slop-sample.md:3:38  warn    filler-phrase
  Throat-clearing filler. Cut it; the sentence is stronger without it.
  > it's important to note

examples/slop-sample.md:5:1  warn    false-agency
  Inanimate subject given a human verb. Name the actual person or group doing the thing.
  > The data tells

examples/slop-sample.md:7:1  strong  not-but-construct
  "Not just X, it's Y" is one of the most overused AI shapes. Pick one claim and state it directly.
  > It's not just about productivity, it's

35 finding(s)  |  grain 63/100  |  verdict: ai-leaning  |  hint: 19  warn: 14  strong: 2
```

Run against the clean sample:

```
$ prosegrain examples/clean-sample.md
examples/clean-sample.md: clean, grain 100 (human)
```

## The grain score

`grain` is a 0..100 metric. Higher means more human texture. It's normalized for length, so a 5,000-word article with 5 findings doesn't score the same as a 200-word post with 5 findings.

| Grain | Verdict |
|-------|---------|
| 85 to 100 | human |
| 65 to 84 | mostly-human |
| 40 to 64 | ai-leaning |
| 0 to 39 | ai-heavy |

Caveat: scores on text under ~50 words aren't meaningful. The findings list still is.

## Why structure, not just words

<!-- prosegrain:off -->
Word lists are easy. Anyone can check for "delve" and "tapestry". The harder, more reliable AI tells live in structure:
<!-- prosegrain:on -->

- **Sentence-length uniformity.** Human writing varies. AI prose tends to medium-length sentences, evenly packed.
- **False agency.** AI prose loves giving "the data" or "the market" human verbs. It avoids naming an actor.
- **Three-beat parallels.** Three short negations in a row. Three sentences starting with "It enables…". Three "X, Y, and Z" lists in the same paragraph.
- **The "not just X, it's Y" reflex.** Almost no human writes this twice in the same piece. AI does it three times.

These are what `prosegrain` is built around. The word list is a useful baseline, not the main signal.

## Ignoring sections

To suppress findings inside a region (useful for documentation that quotes AI patterns, like this README), wrap the section in HTML comments:

```
<!-- prosegrain:off -->

This section won't be linted. "It's not just X, it's Y" is fine here.

<!-- prosegrain:on -->
```

## Limitations (honest list)

- The sentence splitter is a heuristic. Abbreviations like "Dr." or "e.g." can over-split. This rarely affects detection but can shift line numbers.
- Code blocks are stripped, but inline code with backticks is only stripped on a single line. Multi-line tricky cases may leak.
- Custom rule packs aren't supported yet. Rules and word lists are baked in for v0.1. Configurable rules are on the v0.2 roadmap.
- The score is calibrated for English prose. Other languages will produce noisy results.

## Roadmap

- **v0.2**, YAML config for custom word lists and rule overrides; web UI for paste-and-check.
- **v0.3**, VS Code extension; pre-commit hook helper.
- **v0.4**, Pattern catalog moved to a separate versioned repo with PR workflow.

## Contributing

Patterns evolve. If you spot an AI tell that we miss, open an issue with a real example (a paragraph, not a one-liner). PRs adding rules need a new test in `tests/integration.rs` that fails before the rule and passes after.

## License

MIT. See `LICENSE`.
