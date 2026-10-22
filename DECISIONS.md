# DECISIONS

Architecture and process decisions, newest first. Each entry states the
decision, the rationale, and the rejected alternatives.

## 2026-10-23

### D-001 — Product name: **Engram**
"SynapseFlow" (working name) was judged too similar to the inspiration
product's name and replaced with **Engram** — the neuroscience term for the
physical trace a memory leaves. Original, short, and unrelated to any
existing SRS product name. *Rejected:* Loci, Rote, Memorable (weaker or more
generic), keeping SynapseFlow (clean-room naming hygiene).

### D-002 — Stack: pure Rust + egui/eframe 0.36, JSON persistence
Matches the guide's default recommendation for desktop tools; gives
cross-platform builds (Win/macOS/Linux) from one codebase and headless-testable
domain code. SQLite was rejected for V1 — human-readable JSON keeps the data
format auditable and diffable, and atomic writes plus small libraries make a
database unnecessary. WASM/web is deferred (phase 5) with browser-storage
concerns (see assumptions Q-1).

### D-003 — CI-first workflow; all heavy gates in GitHub Actions
The local machine is memory-constrained (guide §4). `cargo test/clippy/check`
and release builds run only in CI on Linux+Windows+macOS runners; locally only
`cargo fmt` and read-only git are used. Verification truth = the CI result.

### D-004 — Scheduling: SM-2 with learning/relearning steps
The publicly documented SuperMemo-2 algorithm, extended with short-interval
learning steps (1m, 10m defaults), an ease clamp (1.3–3.0), an easy bonus
(1.3×) and a hard factor (1.2×). All parameters are user-configurable and
sanitised at the boundary. *Rejected:* a from-scratch adaptive algorithm
(unproven, unreviewable), FSRS (heavier state, harder to test in one slice;
candidate for post-V1).

### D-005 — Time is injected, never read, inside the domain
Every scheduling/statistics/session function takes `now: DateTime<Utc>`.
Makes the whole domain deterministic and testable (fixed timestamps in
tests), and keeps real-clock reads at the UI boundary. *Rejected:* domain
calling `Utc::now()` (untestable, hides mutation of due dates).

### D-006 — Review log is append-only and central
Reviews are logged once per answer in `Library::reviews` (id, card, deck,
timestamp, grade, before/after interval) instead of per-card histories.
Simplifies statistics, keeps cards small, and makes deletion semantics clean:
logs die with their card or deck.

### D-007 — Delete-deck semantics: cards cascade, sub-decks re-parent
Deleting a deck deletes its cards (and their logs) but re-parents child decks
to the deleted deck's parent, so no part of the tree is silently lost. The UI
states this explicitly in the confirmation prompt. *Rejected:* refusing to
delete non-empty decks (annoying), full cascade of sub-decks (data loss).

### D-008 — Session queue order: learning → review → new, deterministic
Due learning steps first (soonest due), then due reviews (soonest due), then
new cards in creation order capped by the daily new limit. No randomisation
in V1 so tests are deterministic and users can reason about the queue.
*Rejected:* interleaving (deferred; noted as a future setting).

### D-009 — Offline heuristic generator instead of an LLM API
The "AI card generation" capability ships as a fully offline rule-based
generator (definition lines, Q/A pairs, copula sentences → cloze). It never
invents facts because it only quotes source text. *Rejected:* shipping an
HTTP LLM client in V1 (network dependency, key management, non-determinism,
untestable offline; explicitly deferred).

### D-010 — Auto-save on every mutation, atomically
The library is written to disk (temp file + rename) whenever a command
succeeds; no explicit save is required and no data can sit unsaved beyond one
frame. *Rejected:* manual-save-only (violates "never lose user data"), debounced
autosave (window for loss on crash).

### D-011 — Cloze syntax: `{{cN::…}}` public convention
Reading/writing the long-standing public cloze markup keeps card text
portable as plain text. V1 studies all deletion groups on one face and grades
the card as a whole; per-deletion faces are a post-V1 item (traceability
notes this honestly).

### D-012 — CSV as the exchange format, JSON for backups
CSV (with RFC-4180 quoting, `Deck,Type,Front,Back,Tags`) interchanges with
spreadsheets and other tools; whole-library JSON backups are exact copies.
*Rejected:* only-native-format (no interop), Anki's proprietary binary
`.anki2` (not a public format; clean-room rules forbid reading its internals).
