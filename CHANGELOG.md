# Changelog

All notable changes to Engram are documented here. Format:
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[SemVer](https://semver.org/).

## [Unreleased]

### Added

- Core domain: card model (Basic + Cloze), deck tree with colours and
  re-parenting semantics, SM-2 scheduler (pure, time-injected), study
  sessions, review log, statistics (heatmap/forecast/retention/streak) and
  the offline note-to-cards generator.
- Data layer: human-readable `.engram.json` library with atomic writes,
  app preferences, CSV import/export and JSON backups.
- UI: deck tree + card table with search and tag filters, card/deck editors,
  settings dialog, keyboard-first study mode (Space reveal, 1–4 grades with
  interval previews), stats page (activity heatmap, due forecast, tiles) and
  the generate page.
- File management: new/open/save/save-as library, CSV import/export, JSON
  backup export, automatic prompt-free persistence of every mutation.
- Cross-platform CI (Linux/Windows/macOS): `cargo fmt --check`, clippy
  `-D warnings`, full test suite, release build.
- Original procedurally drawn application icon and violet/dark+light theme.
