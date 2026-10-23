# AGENTS.md

This document provides guidance for AI coding agents working on **Engram**.

## Purpose
Build a free, cross-platform Rust spaced-repetition / active-recall
flashcards app (egui/eframe). Adhere to the build guide (kept outside the
repo, never committed/pushed); other products may be named only as
capability-level inspiration in `docs/inspiration.md`. Maintain quality
gates.

## Codebase rules
- Rust only, 100% free OSS.
- Domain-first: `src/domain/` is pure (no UI, no I/O), fully serialisable
  with serde, testable headlessly.
- Formatting: `cargo fmt --all -- --check` and formatted code. No trailing
  whitespace.
- Linting: `cargo clippy --all-targets -- -D warnings` (warnings are
  errors).
- Testing: add/extend tests when changing behaviour or fixing bugs.
- Error handling: use `thiserror` for domain/data errors; avoid panics in
  logic; prefer `Result`.
- Conventions: snake_case, modules flat and explicit; doc comments for
  public API.
- Dependencies: prefer well-maintained, MIT/Apache-2.0 crates.
- UI: egui/eframe; responsive panels; keyboard-first study mode (Space
  reveal, 1–4 grade); Esc cancels; Ctrl+S saves.
- Persistence: JSON (serde_json), `.engram.json` extension, human-readable;
  include UUIDs and timestamps; atomic writes (write temp → rename).
- Git: commit small, focused slices; messages imperative and descriptive.
  Prefer the PR loop: push a branch, open/update a PR, verify CI, merge
  (D-013).

## Testing strategy
- Unit tests in `src/domain/*` modules and `tests/domain_tests.rs`.
- Headless tests must not require a display, network, or filesystem writes
  outside `target/`. UI is an integration surface; logic lives in domain.
- Roundtrip tests: serialise → deserialise and assert equality.
- Scheduling tests pin `now` explicitly — never `Utc::now()` inside domain
  logic (the caller injects time).

## Implementation guidance
- Build in vertical slices; update PROGRESS.md, CHANGELOG.md,
  DECISIONS.md, REQUIREMENTS_TRACEABILITY.md as you go.
- When adding features: define domain types first, add tests, then UI.
- Scheduler: SM-2 with configurable ease/interval params; grade mapping
  Again/Hard/Good/Easy; lapses re-enter learning steps; ease clamped
  (1.3–2.5).
- Study queue: due first, then learning, then new (respecting the daily new
  limit); order is deterministic for tests.

## CTO.new-style operating model (Chat → Task → PR → merge)
Every unit of work is a **Task**:

1. **Chat (Orchestrator)** — plan mode: clarify, write the Task brief
   (requirement IDs, files, acceptance criteria). Never edits code.
2. **Task (Coder)** — execute exactly one bounded slice: implement, tests,
   `cargo fmt`, then delegate GitHub interaction to `ci-runner`
   (commit → push branch → open/update PR).
3. **GitHub Actions** — the free Task Runner verifies the PR.
4. **Reviewer** — read-only review of the diff/PR.
5. **Merge** — with CI green and review done, the orchestrator merges the
   PR (owner delegated merge authority: DECISIONS.md D-013). `main` is
   updated; `opencode/ci` is updated in place on the next run.
6. Update `PROGRESS.md` / `CHANGELOG.md` / `REQUIREMENTS_TRACEABILITY.md`
   with exact CI results.

Canonical roles (`.opencode/agents/`): `orchestrator` (Chat, plans only),
`coder` (Task, executes), `explorer` (read-only search), `reviewer`
(read-only review). Supplementary Task-mode helpers: `domain-dev`,
`ui-dev`, `test-writer`, `fixer`, `docs-writer`, `ci-runner` (owns every
GitHub interaction). All run on free OpenCode models in `subagent` mode;
conventions: subagents never commit or push except `ci-runner`.

## CI-first (no heavy local runs)
All CPU/RAM-intensive work runs in **GitHub Actions**, never on the desktop
host (it is memory-constrained and restarts under load): `cargo test`,
`cargo clippy`, `cargo check`, and release builds are CI-only. Locally, run
just `cargo fmt --all` (trivial) and read-only git. Workflow: edit →
`cargo fmt --all` → commit → push → read the CI run (`gh run view <id>`) →
route any failures back through the `fixer` subagent. Every push to `main`
runs fmt check, clippy (`-D warnings`), full tests (Linux/Windows/macOS),
and a release build.

## Do not
- Commit external reference material or notes about other products;
  capability-level inspiration belongs only in `docs/inspiration.md`.
- Add anything that is not original work. All code, docs and assets must be
  original (public standards + our own spec + capability-level inspiration
  only).
- Introduce AGPL/GPL dependencies without approval.
- Mix UI and business logic.
- Push without running quality gates.
- Claim a slice is done before CI is green.
