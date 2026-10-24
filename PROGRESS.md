# Progress

Running status of the project. Newest entries at the top. Append per slice;
never rewrite history.

**Last updated:** 2026-10-24

---
## 2026-10-24 — v0.2.0 slice merged; PR #8 closed; CI required checks green

### Merge
- PR #8 `[ci] v0.2.0: queue ordering, deck-local limits, suspend/bury + history, cram sessions` merged to `main` (squash, branch deleted).
- Required CI checks green on the merged commit:
  - `check (ubuntu-latest)` ✅
  - `build (windows-latest)` ✅
  - `build (macos-latest)` ✅
  - `supply-chain` ✅ (secret scanning, cargo-deny licences, CycloneDX SBOM upload now non-fatal)
- CodeQL remains red (shallow-clone/fetch issue, not in `.ci-required.txt`).
- Formatting, clippy `-D warnings`, tests, and release builds all green; the queue-ordering regression `queue_order_due_first_matches_v010` fixed by sorting reviews by `Reverse(due)`.

### Guide Part II
- Part II repository-health docs and CI supply-chain changes shipped with the merge.
- Branch protection still pending; labels created via `gh` earlier.
- Next: enable branch protection on `main` with required status checks, update `.ci-required.txt` to include supply-chain once stable, and tag `v0.2.0` after manual QC passes.

---

## 2026-10-24 — Guide Part II adopted; v0.2.0 study-workflow slice implemented

### v0.2.0 slice (queue ordering, deck limits, suspend/bury + history, cram)
- Four of the five gated v0.2.0 research notes are implemented: queue
  ordering (Due-first / Interleaved / daily-seeded Shuffle), per-deck
  daily-limit overrides resolved up the deck tree, suspend + bury with a
  review-history browser, and cram sessions that log answers without
  mutating the schedule.
- Domain changes stay pure and time-injected; every new persisted field has
  a serde default, so a 0.1.0 `.engram.json` still loads
  (`v010_library_json_without_new_fields_still_loads`).
- UI: the study start screen gains the cram toggle, an effective-limit
  caption naming the deck that supplied the cap, and the queue-order label;
  Settings gains the queue-order selector; the deck dialog gains limit
  overrides; the card editor and deck table gain suspend/bury controls and
  a per-card "Log" history window.
- **Deferred:** multi-cloze faces (`docs/research/multi-cloze-faces.md`) —
  it reworks the queue entry to carry a face and migrates every cloze card,
  so it lands as its own slice.
- CI: verification run in progress on `opencode/ci`.

### Guide Part II (§16–§25) — open-source readiness
- Repository-health files added: `CODE_OF_CONDUCT.md`, `SECURITY.md`
  (private reporting + threat model + incident response), `ACCESSIBILITY.md`
  (WCAG 2.2 AA target), `VISION.md`, `GOVERNANCE.md`, `CONTRIBUTORS.md`.
- `.github/ISSUE_TEMPLATE/` added: bug, feature and accessibility forms plus
  `config.yml`; the PR template gains the accessibility checklist.
- CI supply chain added: secret scanning (gitleaks), SAST (CodeQL),
  dependency licence classes (cargo-deny via `deny.toml`) and a CycloneDX
  SBOM; Dependabot configured for Cargo and Actions; every Action pinned to
  a commit SHA.
- Contribution labels created (`good first issue`, `help wanted`,
  `documentation`, `first-timers-only`, and area labels including
  `accessibility` and `security`).
- Repository-local git identity set to `Engram Dev <dev@example.com>`
  (guide §5.5); `ci.sh` already asserts it before committing.
- Guide §26 open questions recorded as decisions in `DECISIONS.md`.

### Next
1. Confirm CI green for the v0.2.0 slice, then merge.
2. Enable branch protection on `main` with required status checks.
3. Implement multi-cloze faces, then the v0.3.0 slice (Anki import, FSRS).

---

## 2026-10-24 — Step 7 of the guide sequence: ROADMAP expanded + research notes drafted

- **`ROADMAP.md` expanded** (guide §6.2): staged milestones v0.1.x →
  v1.0.0 with evidence-based readiness (breadth ~55% implemented,
  next-milestone ready-for-work 100%).
- **`docs/research/` created** with 7 DRAFT notes (original behaviour
  summaries, acceptance criteria, edge cases, public URLs only):
  `multi-cloze-faces.md`, `card-history-suspend-bury.md`, `cram-sessions.md`,
  `deck-local-limits.md`, `queue-ordering.md`, `anki-import.md`,
  `fsrs-scheduler.md`.
- **HUMAN GATE (guide §2.3/§8.4):** these notes require the owner's
  approval before any dependent implementation Task starts. Approval
  criteria: original wording, URLs present, no copyrighted paste, no
  instruction to clone vendor UI.

### Next per the guide (§6.2/§14)
1. Owner approves / edits / rejects the research notes (the gate).
2. Human QC walk of the installed app (`docs/qc/checklist.md`).
3. On approval: approved notes convert to staged Tasks → v0.2.0
   (Chat → Task → PR → merge → Release).
4. Small fixes from the QC walk can ship as v0.1.1 without research.

---


## 2026-10-24 — v0.1.0 RELEASED; guide v3 (suite-ready) implemented; cargo banned locally

| Signal | State |
| --- | --- |
| **GitHub Release** | ✅ **v0.1.0 published** — <https://github.com/Perchemop/engram/releases/tag/v0.1.0> — binaries for x86_64-linux, aarch64-linux, Windows, macOS + `checksums-sha256.txt` |
| Release run | ✅ `release.yml` green: `40045064229` (4 build jobs + release job) |
| PRs | ✅ #5 (suite-readiness: crash report, update check, release workflow, PR template, ROADMAP, CONTRIBUTING, brand) · #6 (packaging path fix) · #7 (release-notes extraction fix) — all merged |
| `main` CI | ✅ green after every merge (latest: run for `2ba0e44`) |
| Local install | `~/.local/bin/engram` updated to the v0.1.0 aarch64 release binary |

### Guide v3 ("Suite-Ready") implementation — complete
- **§6.1 v0.1.0 hard gate**: all criteria met (installable binary, real
  workflow, identity, crash stub → full crash reporting, update check,
  CI green, docs, legal baseline, tagged Release with honest notes).
- **§7 crash reporting**: panic hook → scrubbed local report → next-launch
  consent dialog (review / copy / pre-filled issue URL / discard). No
  embedded tokens. Traceability `R-CR-001…005`.
- **§7.2 update check**: GitHub Releases API, semver compare, fail-soft
  offline, 24h auto-check cache, manual Help-menu check. Traceability
  `R-UC-001…005`.
- **§8.3 PR template** in force; **ROADMAP.md** + **CONTRIBUTING.md** +
  **docs/design/brand.md** + `assets/engram.svg` + README privacy section.
- **§2.5 legal checklist** added to the QC gate and walked for v0.1.0.
- Policy: **cargo is banned on the local machine outright** (D-017) —
  agent definitions and process docs updated; fmt is hand-maintained and
  CI-enforced.

### Open items (next session)
1. Human keyboard/visual QC walk (app installed and running).
2. Deferred: per-frame memoisation; post-v0.1.0 research notes are
   human-gated (guide §2.3) before large new feature Tasks.

---


## 2026-10-23 — ARM64 build, launch smoke test, QC close-out (PR #4 merged)

- **PR #4 merged** (PR CI `39949148154`, main CI `39949643789`): the build
  matrix now includes a free `ubuntu-24.04-arm` runner → `engram-linux-arm64`
  artefact. The owner's machine is aarch64, so the binary is now runnable
  locally (the x86_64 artefact never was).
- **Launch smoke test:** the arm64 binary from run `39949643789` launched
  with an isolated `XDG_DATA_HOME`, ran 8s without crash, wgpu initialised
  (llvmpipe software rasteriser in the VM). Behaviour note: no library file
  is written until the first change (now documented).
- **Guide §9/§12 close-out:** `docs/known-limitations.md` written; evidence
  based `docs/implementation-report.md` produced; secrets scan of the full
  history + tree came back clean.
- Platform set now: Linux x86_64, Linux aarch64, Windows, macOS.

### Local install (2026-10-23)
`engram-linux-arm64` from run `39950660293` installed to
`~/.local/bin/engram` (on PATH) with a desktop entry
(`~/.local/share/applications/engram.desktop`, original SVG icon). The app
is running on the owner's desktop for the manual QC walk. Update path:
re-download the `engram-linux-arm64` artefact from the latest green CI run
and repeat the `install -m 755` step.

### Open items (next session)
1. Human keyboard/visual QC walk (`docs/qc/checklist.md`) — the app is
   installed and running on the owner's desktop for exactly this.
2. Deferred: per-frame memoisation of deck summaries / filtered ids.

---


## 2026-10-23 — Review-fix slice merged via PR #1 (first CTO-style cycle complete)

| Signal | State |
| --- | --- |
| PR | ✅ **#1 merged** (`opencode/ci` → `main`, merge commit `190a633`) — first full Chat → Task → PR → merge cycle (guide §5, D-013) |
| PR CI | ✅ green — run `39938361670` (`check`, `build (windows-latest)`, `build (macos-latest)` all success) |
| `main` CI | ✅ green — run `39939371572` (same three jobs, success) after the merge |
| Test suite | 73 lib tests + 10 integration tests passing on all three runners |

Slice contents (all reviewer findings fixed, verified by CI):
- CSV: quote-aware records (multi-line/quoted fields round-trip), single-pass
  export (no panic, no O(n²)), atomic writes, comma-separated tags.
- Domain: `ReviewLog` records the queue kind (`#[serde(default)]`, schema
  compatible); per-calendar-day new/review limits derived from it; learning
  steps always served (D-014).
- Robustness: deck-path/ancestor walks cycle-guarded; UI closes dialogs on
  library switch; autosave backs off 5s on failure; explicit Save immediate.
- Process: guide updated to the CTO.new model — agent roles aligned
  (`orchestrator`/`coder`/`explorer`/`reviewer`), AGENTS.md /
  docs/development.md rewritten, merge authority delegated (D-013).

CI bring-up history for this slice: 5 red runs fixed from logs only
(`39931250734` unused import + `usize::from(u32)` + test unwrap on Result →
`39934841977` 2 test-expectation bugs → `39937012627` missing test import →
`39937293771` CSV tag expectation → `39938361670` green).

### PRs #2/#3 — CI artefacts and lockfile (2026-10-23)
- **PR #2 merged** (CI `39942963880`): every CI run now uploads the release
  binary per OS (`engram-linux` / `engram-windows` / `engram-macos`) and the
  generated `Cargo.lock`.
- **PR #3 merged** (CI `39945498142`): `Cargo.lock` committed to the repo —
  a clean checkout builds the exact verified dependency set (guide §1).
- Post-merge `main` CI: `39946756653` green.

### Open items (next session)
1. Manual QC walk of `docs/qc/checklist.md` UI sections — download the
   `engram-linux` artefact from the latest CI run (no local builds; the
   binary is a single file).
2. Deferred from review: per-frame recomputation of deck summaries /
   filtered ids (fine at current scale; memoise if libraries grow).

---


## 2026-10-23 — Phases 0–3 complete; first green CI run (all three OSes)

| Signal | State |
| --- | --- |
| CI | ✅ **GREEN** — run `39889290098`: `check (ubuntu-latest)` ✅ (fmt → clippy `-D warnings` → full test suite → release build), `build (windows-latest)` ✅, `build (macos-latest)` ✅ |
| Test suite | 65 lib tests + 10 integration tests, all passing on all three runners |
| Working tree | clean, all commits on `main` (`0e48c1d…3350c59`) |

### How CI was brought to green (the guide §4 loop, fix-by-CI only)
Six pushes, zero local builds. Failure classes found and fixed purely from
CI logs:
1. **Pass 1 (42 errors):** missing domain/data re-exports (`Scope`,
   cloze renderers, `default_library_path`), `#[serde(default)]` on types
   without `Default`, egui 0.36 API (`allocate_exact_size` returns
   `(Rect, Response)`), editor constructors returning `Option`,
   icon glyph vars lost in an edit, format-string brace escaping.
2. **Pass 2:** `Scope::from_option` missing, `hover_pos()` returns
   `Option<Pos2>`, session-test `Uuid` import.
3. **Pass 3:** clippy `manual_inspect`, `unnecessary_sort_by` (×3).
4. **Pass 4:** unused test import.
5. **Pass 5:** 4 genuine test bugs caught by CI: filename normalisation
   dropped a dot; learning-step graduation expectation wrong (Good advances
   to the next step, graduation happens at the last step); easy-bonus test
   ignored the ease update order; elapsed-time unit mistake (t(n) was
   seconds, not minutes).
6. **Pass 6:** daily-limit test didn't account for the learning step's
   +10-minute due offset.

### Reviewer pass (2026-10-23) and its fixes
Findings, all fixed and awaiting CI verification on the `opencode/ci` PR:

| Sev | Finding | Fix |
|---|---|---|
| HIGH | CSV round-trip corrupts multi-line/quoted card content | quote-aware record scanner in `data::export` + regression test |
| MED | daily limits were per session-build, not per day | per-calendar-day accounting from the review log (D-014) + tests |
| MED-LOW | panic + O(n²) in CSV export | single-pass export, no `expect` |
| LOW-MED | deck-path/ancestor walks could freeze on corrupt cycles | guards + unit test |
| LOW | autosave retried every frame while failing | 5s backoff, explicit Save stays immediate |
| LOW | dialogs survived Open/New library | dialogs closed on library switch |
| LOW | JSON backup export was non-atomic | atomic temp+rename |
| nit | numpad keys | void — egui has no numpad key variants |

### Process change (2026-10-23)
The guide was updated to the CTO.new model (Chat → Task → PR → merge).
Engram adopted it: agent roles aligned (`orchestrator`/`coder`/`explorer`/
`reviewer`), `AGENTS.md` / `docs/development.md` rewritten, and the owner
delegated merge authority (D-013) — PRs merge when CI is green.

### Open items (next session)
1. Manual QC walk of `docs/qc/checklist.md` UI sections on a real binary
   (download the release artefact from CI; no local builds).
2. Clone the CI-generated `Cargo.lock` back into the repo.
3. Deferred from review: per-frame recomputation of deck summaries /
   filtered ids (fine at current scale; memoise if libraries grow).

---

## 2026-10-23 — Discovery + Phases 0–3 initial implementation (failure history kept for the record; status superseded by the green run above)

### Discovery (guide §0)
Owner answered the discovery set for a spaced-repetition app inspired by
synaps.app (capability-level inspiration only). Key answers recorded in
`docs/project-brief.md` / `docs/scope.md`: Rust + egui/eframe, cross-platform
desktop first, fully offline, CI-first build policy, V1 scope = deck/tag
management, spaced-repetition engine, rich cards (incl. cloze), review mode +
analytics, offline card generation (AI remote API explicitly optional/deferred).

### Naming
Working name "SynapseFlow" was rejected by the owner as too close to the
inspiration product. Renamed to **Engram** (D-001): GitHub repo renamed to
`Perchemop/engram`, all docs/sources updated.

### Implemented (all committed together; CI is the verifier)
- **Repo scaffold**: public GitHub repo, CI workflow (fmt → clippy
  `-D warnings` → tests → release build; Linux/Windows/macOS matrix),
  `.ci-required.txt`, `.gitignore` refusing the local guide, agent
  definitions (`.opencode/agents/*`), MIT licence, AGENTS.md, tracking docs.
- **Domain** (`src/domain/`, pure, serde, headless): cards (Basic/Cloze with
  `{{cN::…}}` rendering), deck forest with colour tokens, SM-2 scheduler
  (time-injected, sanitised params, previews), library commands (decks,
  cards, reviews, search, counts, session building), sessions, statistics
  (heatmap/retention/streak/forecast), offline note-to-cards generator.
  Unit tests throughout.
- **Data** (`src/data/`): atomic JSON persistence, prefs, CSV import/export,
  JSON backup; disk + CSV roundtrip integration tests.
- **UI** (`src/ui/`): app shell with menus/tabs/status, deck tree + card
  table (search, tag filters, multi-select, delete confirmations), card
  editor, deck dialog, settings dialog, study page (reveal + 1–4 grading
  with interval previews + summary), stats page (tiles, heatmap, forecast),
  generate page, procedural app icon, dark/light theme, keyboard support
  (Ctrl+S, Space, 1–4, Esc). Auto-save on every mutation.

### Status
| Signal | State |
| --- | --- |
| Compile/test/clippy | ✅ **CI GREEN** — run `39889290098` (see top entry) |
| `cargo fmt --all -- --check` | clean (local, allowed) |
| Working tree | clean, pushed to `main` |

### Open items (next session)
1. Push and confirm the **first green CI run**; route any failures through
   `fixer` per the pipeline.
2. Reviewer pass over the full diff.
3. Walk `docs/qc/checklist.md` manual UI sections on a real build.
4. Update traceability rows from `DONE (pending CI proof)` to `DONE` with
   the CI run id as evidence.
