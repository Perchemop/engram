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
| `cargo fmt --all` (+ `--check`) | Local (trivial) |
| Read-only `git status/diff`, `gh run view` | Local |
| `cargo check`, `clippy`, `test`, release build | **CI only** — never locally |

## Slice workflow (guide §8)

1. Pick the next unfinished vertical slice (`PROGRESS.md`).
2. Implement via the subagent pipeline in `.opencode/agents/`:
   `domain-dev` (types + unit tests) → `test-writer` → `ui-dev` →
   `reviewer` (read-only) → `docs-writer` → `fixer` (fmt/clippy/test
   failures). Subagents never commit or push; the parent agent does.
3. `cargo fmt --all`, commit a small focused slice, push to `main`.
4. Watch CI (`gh run view <id>`): fmt → clippy `-D warnings` →
   `cargo test --all` (3 OSes) → release build. Route failures to `fixer`.
5. Record exact results in `PROGRESS.md` / `CHANGELOG.md`; update
   `REQUIREMENTS_TRACEABILITY.md`. Never claim a feature is done before CI
   is green (guide §9).

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
- No trailing whitespace; format with `cargo fmt --all`.

## Originality

Engram's code, docs and assets are original work, derived from public
standards, our own spec and capability-level inspiration
(`docs/inspiration.md`). Third-party dependencies are permissively licensed
and listed in `ATTRIBUTION.md`. Never commit reference material from other
products; `guide.md` is git-ignored.
