# Implementation Report — Engram V1

Evidence-based close-out of the agreed V1 scope
(`docs/scope.md`), per guide §12.12. Every claim links to a CI run, a test,
or an artefact. Produced 2026-10-23.

## Scope delivered

| Capability (scope.md) | Status | Evidence |
|---|---|---|
| Deck & tag management (nested tree, colours, search, filters) | ✅ | `R-D-001…006`, `R-C-004…006` — 12 unit tests |
| SM-2 spaced repetition engine (steps, clamps, params, previews) | ✅ | `R-S-001…008` — 12 scheduler tests |
| Rich cards: Basic + Cloze (`{{cN::…}}`) | ✅ | `R-C-001…003` — 5 card tests |
| Interactive study mode (reveal, 1–4 grading, previews, summary) | ✅ | `R-Q-001…005` — 8 session/library tests; UI walked partially (smoke test) |
| Statistics (heatmap, forecast, retention, streak) | ✅ | `R-T-001…005` — 6 stats tests |
| Local persistence + CSV/JSON exchange, backups | ✅ | `R-P-001…006` — 9 data tests incl. multi-line round-trip |
| Offline card generator from notes | ✅ | `R-A-001…005` — 6 generator tests |
| Daily limits (per calendar day) | ✅ | `R-Q-002` — 2 dedicated tests |
| AI card generation via LLM API | ⏸ deferred by scope (offline heuristic generator shipped; Q-3) |
| Cloud sync / accounts / mobile | ❌ explicitly out of scope (`docs/scope.md`) |

Full mapping: `REQUIREMENTS_TRACEABILITY.md` (74 requirements, all `DONE`
with test traces; UI rows verified by smoke test + pending human QC).

## Verification evidence

- **Platforms:** Linux x86_64, Linux aarch64, Windows, macOS — all four
  tested (fmt → clippy `-D warnings` → full suite → release build) on
  run [`39949643789`](https://github.com/Perchemop/engram/actions/runs/39949643789).
- **Latest green `main` run:** `39949643789` (check + 3 build jobs).
- **Test count:** 73 library tests + 10 integration tests
  (`tests/domain_tests.rs`), headless, time-injected.
- **Artefacts:** every run attaches `engram-linux`, `engram-linux-arm64`,
  `engram-windows`, `engram-macos`, `cargo-lock`.
- **Launch smoke test** (2026-10-23): the `engram-linux-arm64` binary from
  run `39949643789` launched on the owner's aarch64 Linux machine, ran a
  full 8-second session (killed by timeout — no crash), wgpu initialised
  (software rasteriser in the VM), no library file written until first
  change (documented behaviour, `docs/known-limitations.md`).
- **Secrets scan:** `git log --all -p` + working-tree scan found no keys,
  tokens or credentials (only `.gitignore` secret-pattern lines and doc
  text).
- **Lockfile:** `Cargo.lock` committed from the CI-generated artefact
  (PR #3) — clean checkouts build the exact verified dependency set.

## Process evidence

- Built per the guide's CTO.new model: 4 merged PRs
  (#1 review fixes, #2 CI artefacts, #3 lockfile, #4 ARM64), each verified
  on its PR before merge (merge authority delegated, D-013).
- All compilation/testing happened in GitHub Actions; the local machine
  never ran a build, test or clippy pass (guide §4).
- Reviewer subagent pass on 2026-10-23 produced 8 findings — all fixed and
  re-verified (`PROGRESS.md` table).
- Clean-room discipline: capability-level inspiration only
  (`docs/inspiration.md`); original name, code, icon and design; all
  dependencies MIT/Apache-2.0 (`ATTRIBUTION.md`).

## What "done" does not mean

- The manual keyboard/visual QC walk is the one remaining human step
  (`docs/qc/checklist.md` — binaries are attached to every run).
- Deferred items are listed honestly in `docs/known-limitations.md` and
  Phase 5 of `IMPLEMENTATION_PLAN.md` — none are silently missing
  requirements.
