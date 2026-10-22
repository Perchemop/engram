# Progress

Running status of the project. Newest entries at the top. Append per slice;
never rewrite history.

**Last updated:** 2026-10-23

---

## 2026-10-23 — Discovery + Phase 0–3: full V1 implementation (first CI run pending)

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
| Compile/test/clippy | **Unknown — first CI run pending** (no local builds per guide §4) |
| `cargo fmt --all -- --check` | formatted locally, clean |
| Working tree | uncommitted (about to push to `main`) |
| GitHub Actions | not yet run |

### Open items (next session)
1. Push and confirm the **first green CI run**; route any failures through
   `fixer` per the pipeline.
2. Reviewer pass over the full diff.
3. Walk `docs/qc/checklist.md` manual UI sections on a real build.
4. Update traceability rows from `DONE (pending CI proof)` to `DONE` with
   the CI run id as evidence.
