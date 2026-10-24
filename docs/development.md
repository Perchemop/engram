# Development — Workflow and Conventions

## Constraints (facts)

- The local machine is memory-constrained; **all heavy work runs in GitHub
  Actions** (guide §4, AGENTS.md "CI-first").
- The repository is public with free CI minutes.
- Subagents run on free OpenCode models; NVIDIA free is the fallback tier
  (guide §5).

## Local vs CI

| Task | Where |
|---|---|
| Editing, reading, docs | Local |
| `cargo fmt` / `check` / `clippy` / `test` / build | **CI only** — cargo is banned locally outright (2026-10-24) |
| Read-only `git status/diff`, `gh run view` | Local |
| `cargo check`, `clippy`, `test`, release build | **CI only** — never locally |

## Slice workflow (guide §8)

1. Pick the next unfinished vertical slice (`PROGRESS.md`).
2. **Chat (plan):** the orchestrator writes the Task brief — requirement
   IDs, files likely touched, acceptance criteria, definition of done.
3. **Task (execute):** Coder implements the small slice and its tests
   (rustfmt-clean by hand; CI enforces `cargo fmt --check`), then
   `ci-runner` executes
   `~/.config/opencode/scripts/ci.sh run "<description>"`, which commits
   the tree, pushes the `opencode/ci` branch, opens/updates a **pull
   request**, and waits for the checks.
4. Watch CI: fmt → clippy `-D warnings` → `cargo test --all` (3 OSes) →
   release build. Route failures to `fixer` and repeat.
5. **Review:** the reviewer subagent checks the PR diff.
6. **Merge:** with CI green and review done, the orchestrator merges the PR
   (owner delegated merge authority — DECISIONS.md D-013), then updates the
   tracking files with **exact** CI results. Never claim a feature is done
   before CI is green (guide §9).

## Definition of Done (per slice)

Guide §8: requirements implemented; tests cover normal behaviour, validation
and important failures; committed and pushed; GitHub Actions CI green;
PR opened/updated for review (merged when green — D-013); tracking docs
updated; no fake success states; change small and reviewable; no heavy work
on the local machine.

## Testing

- Unit tests live beside the code in `src/domain/*` and `src/data/*`;
  integration tests in `tests/domain_tests.rs`.
- Headless tests must not require a display or network; filesystem writes
  only in temp dirs that are cleaned up.
- Time is injected: tests pin `now` with `Utc.with_ymd_and_hms(...)` — the
  domain never reads the clock (DECISIONS D-005).
- Roundtrip tests: serialise → deserialise → assert equality.

## Conventions

- `src/domain/` is pure: no egui, no I/O, no panics (use `thiserror` +
  `Result`).
- `src/data/` owns persistence and import/export.
- `src/ui/` renders and calls commands; egui 0.36 API (`App::ui`, `App::logic`,
  `egui::Panel::top/left/...`), disjoint-field borrows preferred over
  `RefCell`.
- snake_case, flat explicit modules, doc comments on public API.
- No trailing whitespace; code is rustfmt-clean (CI enforces it).

## Originality

Engram's code, docs and assets are original work, derived from public
standards, our own spec and capability-level inspiration
(`docs/inspiration.md`). Third-party dependencies are permissively licensed
and listed in `ATTRIBUTION.md`. Never commit reference material from other
products; `guide.md` is git-ignored.
