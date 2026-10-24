# Research note — Multi-cloze-per-card study faces

**Status:** DRAFT — awaiting human approval (guide §2.3/§8.4).
**Milestone target:** v0.2.0.
**Sources (public, capability-level only):**
- Anki manual, cloze deletion: <https://docs.ankiweb.net/editing.html#cloze-deletion>
- SM-2 scheduling unchanged by this feature (local design, D-004)

## Behaviour to implement

Engram 0.1.0 studies **one face per cloze card**: every `{{cN::…}}` group
is blanked together and the card is graded as a whole
(`docs/known-limitations.md`). This note specifies the standard behaviour:

1. A cloze card with groups `c1..cN` produces **N study faces**. Each face
   blanks exactly one group (others are shown as their revealed text).
2. Each face carries its own scheduling state (a face can be learned while
   another is still new).
3. Faces are numbered in the UI: "Face 2 of 3".
4. Editing the sentence re-derives faces; deleting a group deletes its face
   (with its schedule) on save — stated in the save confirmation.
5. Tags apply to the whole card (not per face).

## Acceptance criteria

- [ ] A card with 3 groups creates exactly 3 faces; a card with 0 groups
      is rejected at validation (unchanged).
- [ ] Face prompt shows only the selected group blanked: `The [heart] has
      {{c2::four}} chambers` style rendering for group 1.
- [ ] Each face has an independent `SchedulingState`; answering face 1
      does not move face 2's due date.
- [ ] Queue includes each face as a separate entry (respects daily limits).
- [ ] Deleting a group on edit removes that face and its review-log rows
      only (other faces keep history).
- [ ] Statistics count faces, not cards, in new/learning/review buckets.

## Edge cases

- Group numbers non-contiguous (`c1`, `c3`): treat as faces 1 and 2 (order
  by number).
- Duplicate group numbers (`c2` twice): the two regions form ONE face
  (both blanked together) — standard convention.
- Card edited to fewer groups while faces are mid-learning: removed faces
  are dropped from any open session queue.
- Import (CSV/JSON) of 0.1.0 files: single-state cloze cards become face 1
  only; no data migration needed (faces derive from the sentence).

## Design notes (original)

- Domain: `CardKind::Cloze` gains derived `faces: Vec<ClozeFace>` computed
  from the text (group number → face index); scheduling moves to the face
  (`card.face_states: Vec<SchedulingState>` keyed by group number).
  Migration: 0.1.0 cards get one state covering all groups (index 0).
- The `{{cN::…}}` convention itself is a public interchange format (already
  documented in `docs/inspiration.md`).
