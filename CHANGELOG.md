# Changelog

All notable changes to Engram are documented here. Format:
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[SemVer](https://semver.org/).

## [Unreleased]

### Fixed

- CSV import/export is now quote-aware across the whole file: multi-line
  and quoted-comma/quote card content round-trips correctly (previously a
  multi-line field split into bogus records).
- JSON backup export is atomic (temp + rename), matching library saves.
- Deck/path/ancestor walks are cycle-guarded, so a hand-edited or corrupt
  `.engram.json` can no longer freeze the app; the domain reports sane
  results instead.
- Daily new/review limits are now counted per calendar day instead of per
  session build: the review log records the queue kind of every review
  (`New` = card introduction), and learning/relearning steps are always
  served. CSV tags are comma-separated so multi-word tags round-trip.
- Card/deck/settings dialogs are closed when a library is opened or
  created, so editors can never commit into a stale deck.
- A failing auto-save backs off (retry every 5s) instead of hammering the
  disk every frame; the explicit Save remains immediate.
- Study page counts are labelled accurately ("new queued (daily cap …)").

### Changed

- Process: adopted the guide's CTO.new-style flow — Chat (plan) → Task
  (execute) → PR (review) → merge, with agent roles aligned to the guide
  (`orchestrator`/`coder`/`explorer`/`reviewer`) and specialists retained as
  Task-mode helpers; PRs are self-merged when CI is green (D-013).

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
  `-D warnings`, full test suite, release build — with the per-OS release
  binaries and `Cargo.lock` attached as run artefacts.
- Original procedurally drawn application icon and violet/dark+light theme.
