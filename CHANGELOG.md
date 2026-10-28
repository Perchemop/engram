# Changelog

All notable changes to Engram are documented here. Format:
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/); versioning follows
[SemVer](https://semver.org/).

## [0.1.0] - 2026-10-23

First functional test build. Scope is deliberately limited (see
`docs/scope.md`); the study core is complete and verified.

### Added
- Study core: deck tree with colours, tags, Basic + Cloze cards, SM-2
  scheduling with learning steps and configurable parameters, keyboard-first
  study sessions (Space reveal, 1–4 grading with interval previews),
  statistics (activity heatmap, due forecast, retention, streak).
- Offline card generator from plain notes.
- Atomic JSON persistence (`.engram.json`), CSV import/export, JSON backups.
- Crash reporting: panic hook writes a home-path-scrubbed report locally;
  on next launch the user can review it and submit via a pre-filled GitHub
  issue (no tokens embedded) or discard it.
- Update check: GitHub Releases API, at most one automatic check per 24h,
  fails soft offline; manual check in Help → Check for updates.
- Platforms: Windows, macOS, Linux x86_64 and Linux aarch64, with release
  binaries + SHA-256 checksums attached to GitHub Releases.

### Fixed
- (initial release)

### Known limitations
- Single cloze face per card (all deletions studied together).
- CSV tags cannot contain commas; day boundaries are UTC.
- No sync, accounts, mobile or web target (out of scope for this release).
- Full list: `docs/known-limitations.md`.

## [0.3.0] - 2026-10-27

Data & interoperability milestone.

### Added
- Anki-style text import (File → Import text): TAB/CSV auto-detection,
  RFC-4180 quoting, column mapping with a ten-row preview, cloze detection,
  case-insensitive duplicate skipping, HTML sanitisation, and `Parent::Child`
  deck columns created on demand.
- Pluggable scheduler: Settings → Scheduler offers **SM-2** (default,
  unchanged) and **FSRS** (three-variable model with the public default
  weights), with a target-retention slider (0.70–0.95). Learning steps and
  lapse handling are shared; only graduated intervals differ.
- Example library ships with every release
  (`sample.engram.json`) — decks, Basic + Cloze cards, suspended and buried
  cards, all three phases, ready to open via File → Open Library.

### Changed
- Release pipeline verifies SHA-256 digests and picks per-platform assets
  before offering an update.

### Fixed
- The in-app version string now matches the release tag (0.2.0 and 0.3.0
  binaries reported 0.1.0).

## [0.2.0] - 2026-10-24

Richer study workflow (research notes approved per guide §2.3/§8.4).

### Added
- Queue ordering: Due first / Interleaved / daily-seeded Shuffle
  (Settings → Queue order).
- Deck-local daily-limit overrides, resolved up the deck tree.
- Card history browser and per-card suspend / bury controls.
- Cram sessions that ignore due dates and daily limits.
- Open-source readiness: Code of Conduct, Security policy, Accessibility
  statement, Governance, issue templates, supply-chain CI (gitleaks, CodeQL,
  cargo-deny, SBOM) and Dependabot.

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

- Study-queue ordering (`docs/research/queue-ordering.md`): Due-first
  (unchanged default), Interleaved (new cards spread through the due reviews
  by the documented `floor(i·(R+N)/N)` formula) and a daily-seeded Shuffle;
  learning steps always keep their own due order.
- Per-deck daily limits (`docs/research/deck-local-limits.md`): a deck or any
  ancestor can override the library-wide new/review limits; the most specific
  override wins, and daily accounting is per deck subtree.
- Suspend and bury (`docs/research/card-history-suspend-bury.md`): cards can
  be paused indefinitely or postponed to a chosen instant; blocked cards leave
  every queue, due count and forecast, and a review-history browser shows a
  card's full log.
- Cram sessions (`docs/research/cram-sessions.md`): study a whole scope while
  ignoring the scheduler and daily limits; cram answers are logged but never
  mutate the schedule.
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
