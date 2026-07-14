# Changelog

All notable changes to prosegrain will be recorded here.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0], 2026-05-09

First release. CLI works, tests pass, README dogfoods clean.

### Added

- 12 detection rules across three families:
  - **Words.** `banned-word`, `filler-phrase`, `assistant-opener`.
  - **Structure.** `not-but-construct`, `false-agency`, `triple-list-cadence`, `negative-listing`, `triple-parallel-opening`, `em-dash-overuse`.
  - **Rhythm.** `uniform-sentence-length`, `uniform-paragraph-length`, `long-average-sentence`.
- Plain and JSON output formats.
- `grain` score (0..100) with verdict labels: `human`, `mostly-human`, `ai-leaning`, `ai-heavy`.
- File or stdin input.
- `--strict` flag for CI use (non-zero exit when any `strong`-severity finding is present).
- `--no-words`, `--no-structure`, `--no-rhythm` rule-family toggles.
- `--list-rules` to print all rules.
- HTML-comment ignore regions (`<!-- prosegrain:off -->` / `<!-- prosegrain:on -->`).
- Markdown code-block stripping so fenced code doesn't trigger false positives.
- 15 tests (4 unit, 11 integration).

### Known limitations

- Sentence splitter is heuristic; abbreviations like "Dr." can over-split.
- Score is unreliable for text under ~50 words.
- No custom rule packs yet, rules are baked in.
- Calibrated for English; other languages will be noisy.

[Unreleased]: https://github.com/0xelitesystem/prosegrain/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/0xelitesystem/prosegrain/releases/tag/v0.1.0
