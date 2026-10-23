# Known Limitations

Honest list of what Engram V1 does **not** do, and the known rough edges.
Each entry states the workaround or the plan. Anything marked *deferred* is
tracked in `IMPLEMENTATION_PLAN.md` (Phase 5 backlog) and is not a hidden
requirement.

## Data format

- **CSV cannot express a tag containing a comma.** The CSV exchange format
  separates tags with commas, so a tag like `a,b` round-trips as two tags
  (`a` and `b`). Workaround: rename such tags before export. Multi-word
  tags (`high yield`) and multi-line card content round-trip correctly.
- **Fresh installs create no library file until the first change.** The
  default library is created in memory on first launch and persisted at the
  first mutation (auto-save, D-010). Nothing is lost — there is simply
  nothing on disk yet.
- **`.engram.json` written by Engram 0.1 predates the `queue_kind` log
  field.** Older review logs load with `queue_kind = "Review"` via
  `#[serde(default)]`; daily-limit history for those old days counts every
  review as a non-introduction. New records always carry the field.

## Study semantics

- **A cloze card is one review face.** All `{{cN::…}}` groups in a sentence
  are blanked/shown together and the card is graded as a whole. Per-deletion
  faces (Anki-style) are deferred (Phase 5).
- **Daily limits are per calendar day (UTC).** Learning/relearning steps are
  never capped — by design, so cards are never stranded mid-step — which
  means a heavy day can exceed the review number if many steps are due.
- **Queue order is deterministic, not randomised.** Learning → review → new,
  each by due/creation order. Interleaving/shuffling is a candidate setting
  for a later slice.
- **Timezone is UTC for all day boundaries.** "Today" flips at 00:00 UTC,
  not local midnight. Local-day boundaries are a candidate improvement.

## Platform / packaging

- **No code signing, installers or auto-update.** Binaries are plain
  executables attached to CI runs. Packaging (installer/bundle formats) is
  deferred.
- **Web/WASM target deferred.** The UI uses native file dialogs and a
  filesystem store; a WASM build needs a browser storage backend first
  (assumption Q-1).
- **No cross-device sync.** Files are local by design; export/import is the
  bridge between machines (D-002, out-of-scope in `docs/scope.md`).

## Card generation

- **The offline generator is rule-based, not an LLM.** It recognises
  definition lines, `Q:/A:` pairs and copula sentences only; text outside
  those shapes yields no drafts. It never invents facts (D-009). A
  bring-your-own-API-key generator is a possible later slice (Q-3).

## Performance

- **The deck tree, card table and statistics recompute per frame.** Fine at
  the current target scale (thousands of cards); per-frame memoisation is
  deferred until libraries grow (reviewer note, 2026-10-23).
- **Software rasterisers render slowly.** On machines without a GPU, egui
  falls back to llvmpipe (observed in the VM smoke test) — the app runs but
  animations feel heavy. This is environmental, not an app defect.
