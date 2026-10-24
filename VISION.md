# Vision

Engram exists because remembering what you study should not require handing
your notes, your review history and your attention to a subscription service:
it is a free, open-source, offline-first desktop app for spaced repetition
and active recall, in which you build decks of Basic and Cloze cards, study
them a few minutes a day under the public SM-2 algorithm, and watch an
honest picture of your retention emerge from statistics kept entirely on your
own disk — solving the user's real problem, which is not "where do I store
cards" but "how do I actually remember this next month without a company in
the middle and without losing my data".

## Who it is for

- Learners who study on a computer and want their material to keep working
  when the network, the vendor or the subscription does not.
- People who prefer plain, inspectable files they own and can back up, diff
  or move between machines themselves.
- Users who want active recall and spaced repetition without accounts,
  telemetry or lock-in.
- Contributors who want a small, readable, pure-Rust codebase with a
  domain-first architecture.

## Goals

### Near term

- Keep the v0.1.x build solid: finish the keyboard and visual quality-control
  walk, fix what it surfaces, and hold the contrast and accessibility bar in
  both themes.
- Make the everyday study loop — create, study, grade, review statistics —
  fast and unsurprising on all four supported platforms.

### Medium term

- Richer study workflow: per-deletion cloze faces, card history with
  suspend/bury, filtered cram sessions, deck-local limits and optional queue
  interleaving.
- Better interoperability: public-format Anki text import and an FSRS
  scheduler option alongside SM-2.
- Safer data stewardship: backup reminders and rotation of `.engram.json`
  snapshots.

### Long term

- A finished, dependable release that passes the project's own release gates
  on every platform, with the accessibility and security posture documented
  in `ACCESSIBILITY.md` and `SECURITY.md`.
- The same product, still free and offline-first and still owned by its
  users, with no account or server ever in the path.

## Non-goals

Engram will not:

- Require accounts, sign-in or a hosted service.
- Operate a sync server, or store your data anywhere but your own disk; you
  move data with files, not with a backend.
- Target mobile or web/WASM yet — desktop first, and those targets are
  deferred rather than promised.
- Adopt a proprietary or undocumented file format; the native format stays
  human-readable `.engram.json`.
- Add telemetry, advertising or mandatory network calls.

## See also

The staged, evidence-based plan for reaching these goals — and the current
readiness of each milestone — lives in [`ROADMAP.md`](ROADMAP.md).
