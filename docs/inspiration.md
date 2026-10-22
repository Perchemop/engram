# Inspiration Document (Clean-Room Statement)

## Reference product: synaps.app (capability-level only)

Engram was inspired by synaps.app **only at the level of publicly
observable capabilities**. No proprietary source code, decompilation,
internal assets, strings, layouts, schemas or algorithms from that product
(or any other) were accessed, read, copied or derived.

### Capabilities inspired (black-box, user-facing concepts)
1. **AI-assisted flashcard creation from notes** — turning study material
   into question/answer and cloze cards with minimal manual typing.
   Engram's implementation is a fully offline rule-based generator (D-009).
2. **Adaptive spaced repetition** — review intervals that adapt to
   self-rated recall. Engram implements the public SuperMemo-2 algorithm.
3. **Active-recall-first study flow** — prompt before answer, immediate
   self-grading, short learning steps for new material.
4. **Structured knowledge organisation** — decks (hierarchical) and tags
   with fast retrieval.
5. **Progress and consistency feedback** — activity history, due forecast,
   streaks and retention metrics.

## Other general influences (public knowledge)
- The SM-2 algorithm family (SuperMemo, publicly documented since 1990).
- The public `{{cN::…}}` cloze text convention.

## Attestation
All Engram code, documentation, naming, visual design, icon and interaction
design are original work created for this project. Where a capability
coincides with a commercial product, it coincides only at the level of
"the same user-visible capability", implemented from public standards and
this project's own specifications.
