# Contributing to Engram

Thank you for considering a contribution! Engram is a free, open-source,
offline-first spaced-repetition app.

## Ways to contribute

- **Report a bug** — use the [bug report
  template](.github/ISSUE_TEMPLATE/bug_report.yml).
- **Propose a feature** — use the [feature request
  template](.github/ISSUE_TEMPLATE/feature_request.yml).
- **Report an accessibility problem** — use the [accessibility
  template](.github/ISSUE_TEMPLATE/accessibility.yml); these take a
  dedicated path (see `ACCESSIBILITY.md`).
- **Improve code, tests or documentation** — open a pull request.
- **Help with triage or research** — reproduction, labelling and original,
  sourced summaries are all valuable (see `CONTRIBUTORS.md`).

## Licence terms for contributions

By submitting a pull request or patch you agree that your contribution is
licensed under the project's licence, **MIT**, without additional terms or
conditions. This is inbound equals outbound: what you contribute comes in
under MIT, and it goes out to users under MIT. A DCO sign-off
(`git commit -s`) is appreciated but not required.

## Reporting a bug

Open the repository's issue page and choose **Bug report**. The template asks
for what happened, what you expected, steps to reproduce, the Engram version,
your OS and version, and whether the problem reproduces on a new library.
Please do not include secrets, tokens or personal data, and never report a
security issue in a public issue — use `SECURITY.md` instead.

## Proposing a feature

Choose **Feature request** on the issue page and describe the problem before
the solution. For anything larger than a small fix, **open an issue first**
so the approach can be agreed before code is written; that saves both sides
rework. Check `VISION.md` (explicit non-goals) and `ROADMAP.md` (already
planned) first.

## Setting up the environment

1. Install a stable Rust toolchain with [rustup](https://rustup.rs). The
   pinned version in `rust-toolchain.toml` is installed automatically when
   you build.
2. Clone the repository and enter it:
   ```sh
   git clone https://github.com/Perchemop/engram.git
   cd engram
   ```
3. Build the app:
   ```sh
   cargo build
   ```

## Running the tests locally

Contributors are expected and encouraged to run the full quality gates
locally before opening a pull request:

```sh
cargo test --all
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

What the maintainer does differently, and why: the maintainer's machine is
resource-constrained, so **for the maintainer only**, CI is authoritative and
`cargo` is not run locally — all formatting, linting, tests and builds happen
in GitHub Actions (`docs/development.md`). That is a constraint of one
machine, not a policy for the project. If your machine can run the commands
above, run them; they catch most problems before CI does.

## How to contribute

1. **Open an issue first** for anything larger than a small fix — describe
   the behaviour you want to see, not the implementation.
2. **Fork + branch**, keep the change small and reviewable (one vertical
   slice per PR).
3. **Use the PR template** (`.github/PULL_REQUEST_TEMPLATE.md`) — reviewers
   reject PRs that fail its checklist. UI changes must also complete the
   accessibility subsection.
4. CI must be green: `cargo fmt --all -- --check`, `cargo clippy
   --all-targets -- -D warnings`, `cargo test --all`, release build on
   Linux (x86_64 + aarch64), Windows and macOS.

## What to expect

- **Tone.** Be kind and specific in issues and reviews; assume good faith.
  The project follows `CODE_OF_CONDUCT.md`.
- **Response time.** A reply within seven days is the project's stated
  expectation; it is maintained in spare time (a few hours a week), so please
  be patient after that.
- **Small pull requests.** One focused change with a clear description is
  reviewed far faster than a large mixed one.
- **Tests for behaviour changes.** Any change in behaviour needs a test that
  would fail without it; see `docs/development.md` for the testing strategy.

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
