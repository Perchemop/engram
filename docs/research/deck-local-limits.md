# Research note — Deck-local daily-limit overrides

**Status:** DRAFT — awaiting human approval (guide §2.3/§8.4).
**Milestone target:** v0.2.0.
**Sources (public, capability-level only):**
- Anki manual, deck options (per-deck new/review limits):
  <https://docs.ankiweb.net/deck-options.html>
- Engram's own limit accounting: DECISIONS D-014, `docs/data-model.md`

## Behaviour to implement

1. `Deck` gains optional per-deck limits:
   `new_limit_override: Option<u32>`, `review_limit_override: Option<u32>`.
2. **Effective limit resolution** for a scope: the *most specific*
   override found walking UP the tree from the studied deck (deck → parent
   → … → library default). A deck with `None` inherits its parent.
3. The session builder counts introductions/reviews **per deck subtree**
   against that subtree's effective limit (the studied deck's tree is one
   unit — matches today's single-account model).
4. UI: Settings stays library-wide; the deck dialog gains two optional
   numeric fields with an "inherit" state (`None` = inherit).
5. Display: the study start screen shows the effective limits for the
   chosen scope and where they came from ("cap from deck 'Japanese'").

## Acceptance criteria

- [ ] No overrides anywhere → behaviour identical to v0.1.0 (library
      defaults; identical queue on the same library and time).
- [ ] Override on the studied deck replaces the library default for that
      scope only; other scopes unchanged.
- [ ] Override on a parent applies to the whole subtree unless a deeper
      deck overrides again.
- [ ] Per-subtree accounting: two sessions on sibling decks each get their
      own budget; a parent-scope session counts across the whole tree
      against the parent's effective limit.
- [ ] Serialization is additive (`Option` fields with `#[serde(default)]`);
      0.1.0 files load unchanged.

## Edge cases

- Override smaller than already-introduced today → budget 0 (no new cards;
  learning steps still always served, D-014).
- Override of 0 → deck serves no new cards (valid: "paused" deck for new
  cards only).
- Moving a deck re-evaluates limits (derived from the new ancestry).
- Cycle-corrupted ancestry (hand-edited file): walk stops at the existing
  guard limit; limits resolve from whatever chain was found.

## Design notes (original)

- Domain: `Deck` fields + `effective_limits(deck_id) -> (u32, u32)` walking
  `parent` with the existing cycle guard; `build_session` resolves limits
  from the scope's root deck.
- UI: deck dialog fields; start-screen caption.
