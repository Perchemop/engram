# Data Model — Engram

All persisted state lives in one library file (JSON, human-readable).

## Library (root)

```jsonc
{
  "schema_version": 1,
  "name": "My Engram",
  "settings": {
    "sched": { /* SchedParams, see below */ },
    "daily_new_limit": 20,
    "daily_reviews_limit": 200
  },
  "decks": [ /* Deck */ ],
  "cards": [ /* Card */ ],
  "reviews": [ /* ReviewLog */ ]
}
```

## Deck

| Field | Type | Notes |
|---|---|---|
| id | uuid | stable identifier |
| parent | uuid? | `null` = top level; forest (single parent) |
| name | string | unique among siblings, case-insensitive |
| color | token? | one of violet/blue/teal/green/amber/orange/red/slate |
| collapsed | bool | UI-only, persisted for convenience |
| created | datetime | UTC |

## Card

| Field | Type | Notes |
|---|---|---|
| id | uuid | |
| deck_id | uuid | must reference an existing deck |
| kind | tagged enum | `Basic { front, back }` or `Cloze { text }` |
| tags | string[] | trimmed, lowercased, deduped, sorted |
| created / modified | datetime | UTC |
| sched | SchedulingState | see below |

## SchedulingState

| Field | Type | Notes |
|---|---|---|
| phase | enum | `New` / `Learning` / `Review` / `Relearning` |
| step | usize | index into the active step list |
| ease | f64 | clamped 1.3–3.0 |
| interval_days | f64 | current review interval (0 for new/learning) |
| due | datetime | UTC |
| reps / lapses | u32 | counters |

## SchedParams (defaults)

learning steps `[1, 10]` min · relearning steps `[10]` min · graduating
interval `1.0d` · easy interval `4.0d` · starting ease `2.5` · easy bonus
`1.3` · hard factor `1.2` · interval modifier `1.0` · ease deltas
`again 0.20 / hard 0.15 / easy 0.15` · mature threshold `21d` · max interval
`36500d`. All sanitised (clamped) before use.

## ReviewLog (append-only)

| Field | Notes |
|---|---|
| id, card_id, deck_id | identity |
| at | UTC timestamp of the answer |
| grade | Again/Hard/Good/Easy |
| queue_kind | Review / Learning / New — which queue part served the card; `New` marks a card introduction (used by per-day limits). `#[serde(default)]` so older files load as `Review` |
| interval_before_days / interval_after_days | audit of the interval change |

## App prefs (separate file, `prefs.json`)

`{ "theme": "Dark" | "Light" }` — UI-level, not library data.

## Invariants

- A card's `deck_id` always resolves; deleting a deck deletes (cascades) its
  cards and their logs first (D-007).
- `reviews` is append-only; entries die only with their card/deck.
- Dates are UTC everywhere; the domain never reads the clock (D-005).
- Writes are atomic (temp + rename); a crash never truncates the file.
