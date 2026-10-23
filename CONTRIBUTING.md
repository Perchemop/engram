# Contributing to Engram

Thank you for considering a contribution! Engram is a free, open-source,
offline-first spaced-repetition app.

## Licence terms for contributions

By submitting a pull request or patch you agree that your contribution is
licensed under the project's licence, **MIT OR Apache-2.0** (at the
recipient's option), without additional terms or conditions. A DCO sign-off
(`git commit -s`) is appreciated but not required.

## How to contribute

1. **Open an issue first** for anything larger than a small fix — describe
   the behaviour you want to see, not the implementation.
2. **Fork + branch**, keep the change small and reviewable (one vertical
   slice per PR).
3. **Use the PR template** (`.github/PULL_REQUEST_TEMPLATE.md`) — reviewers
   reject PRs that fail its checklist.
4. CI must be green: `cargo fmt --all -- --check`, `cargo clippy
   --all-targets -- -D warnings`, `cargo test --all`, release build on
   Linux (x86_64 + aarch64), Windows and macOS.

## Ground rules

- **Original work only.** Do not submit proprietary source, decompiled
  material, copied UI text, or third-party assets without a permissive
  licence (anything third-party goes in `ATTRIBUTION.md`).
- **No secrets** (tokens, keys) in commits — the crash-report and
  update-check features deliberately use no embedded credentials.
- **Domain-first architecture:** business logic lives in `src/domain`
  (pure Rust, no UI, no I/O), persistence in `src/data`, UI in `src/ui`.
  Tests ship with every behaviour change.
- Be kind in issues and reviews. See `docs/known-limitations.md` before
  filing "missing feature" reports — some gaps are documented decisions.
