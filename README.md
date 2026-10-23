# Engram

**Spaced repetition that leaves a trace.**

Engram is a free, open-source, cross-platform flashcard app for active recall
and spaced repetition, built in pure Rust with egui/eframe. It runs fully
offline, keeps your data in human-readable JSON on your own disk, and has no
accounts, subscriptions or telemetry.

## Features

- **Deck tree** — nested decks with colour tokens, per-deck due counts and a
  virtual "All cards" root.
- **Two card kinds** — Basic (front/back) and Cloze (`{{c1::…}}` deletions).
- **SM-2 scheduling** — the publicly documented SuperMemo-2 algorithm with
  learning/relearning steps, ease clamps, easy bonus and configurable
  parameters (daily new/review limits, steps, intervals).
- **Keyboard-first study mode** — Space/Enter reveals, `1`–`4` grades
  (Again/Hard/Good/Easy) with live interval previews, Esc ends the session.
- **Statistics** — 5-week activity heatmap, due forecast, maturity split,
  30-day retention and study streak.
- **Search & tags** — case-insensitive full-text search, tag conjunction
  filters, deck-scoped browsing.
- **Import/export** — CSV (deck exchange), JSON (portable backups), plus the
  native `.engram.json` library format with atomic writes.
- **Offline card generator** — paste notes and Engram drafts cards locally
  from `Term: Definition`, `Q:/A:` and "X is Y" shapes. No network, no AI
  service; drafts never invent facts.

## Platforms

Windows, macOS and Linux desktop builds are produced and tested in CI —
every CI run attaches ready-to-run binaries as artefacts
(`engram-linux`, `engram-linux-arm64`, `engram-windows`, `engram-macos`).
Tagged releases (`v*.*.*`) additionally publish signed-checksum binaries on
the [Releases page](https://github.com/Perchemop/engram/releases/latest).
The scheduler, domain and data layers are pure Rust and platform-independent.

## Privacy

Engram is offline-first and private by design:

- **Your data never leaves your machine.** Cards, decks and review history
  live in a local file; there is no server, account or telemetry.
- **Crash reports are created locally and sent only with your explicit
  click.** The report contains the app version, OS/arch, the panic message
  and a home-path-scrubbed backtrace. You can review, copy, or discard it;
  submission opens a pre-filled GitHub issue in your browser (you sign in
  yourself — the app embeds no tokens).
- **Update check** queries the public GitHub Releases API
  (`api.github.com/repos/Perchemop/engram/releases/latest`) at most once
  per 24 hours (or on demand from Help → Check for updates), sends no data
  beyond a standard User-Agent, fails silently offline, and links to the
  release page. It can simply not be used.

## Known limitations

See [`docs/known-limitations.md`](docs/known-limitations.md) for the honest
list (CSV comma-tags, single cloze face, UTC day boundaries, and more).

## Build

```sh
cargo build --release
```

Requires a stable Rust toolchain (`rustup` will install it via
`rust-toolchain.toml`). Heavy verification (tests, clippy, release builds)
runs in GitHub Actions on every push to `main`.

## Data location

The default library lives at:

- Linux: `~/.local/share/engram/Engram/library.engram.json`
- macOS: `~/Library/Application Support/engram/Engram/library.engram.json`
- Windows: `%APPDATA%\engram\Engram\data\library.engram.json`

Use *File → Save As…* to keep a library anywhere else.

## Originality

All code, docs, icons and design are original. Scheduling follows the public
SuperMemo-2 algorithm; the `{{cn::…}}` cloze convention is a long-standing
public interchange format. Third-party dependencies are permissive
(MIT/Apache-2.0) and listed in `ATTRIBUTION.md`. See `docs/inspiration.md`
for the capability-level inspiration statement.

## Licence

MIT — see [LICENSE](LICENSE).
