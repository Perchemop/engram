# Progress

Running status of the project. Newest entries at the top. Append per slice;
never rewrite history.

**Last updated:** 2026-10-23

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

### Open items (next session)
1. Reviewer subagent pass over the full diff (guide §8 / development.md
   pipeline) — recorded here when done.
2. Manual QC walk of `docs/qc/checklist.md` UI sections on a real binary
   (download the release artefact from CI; no local builds).
3. Clone the CI-generated `Cargo.lock` back into the repo.

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
