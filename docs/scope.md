# Scope Document - Engram

## In-Scope Capabilities (V1)

### 1. Deck & Card Organization
- Hierarchical deck structure (parent/child decks).
- Custom deck color tokens and icons.
- Tagging system with multi-tag filtering.
- Card management: Create, Edit, Duplicate, Move, Delete cards.
- Card Types: Basic Front/Back, Cloze Deletion, Multi-field active recall prompts.

### 2. Spaced Repetition Engine
- Implementation of SuperMemo 2 (SM-2) algorithm (Interval, Ease Factor, Repetitions, Due Dates).
- Configurable study parameters (learning steps, target retention, ease adjustment factors).
- Automatic calculation of next review date based on user quality ratings (1: Again, 2: Hard, 3: Good, 4: Easy).
- Queue management: Due reviews, New cards, Learning cards.

### 3. Study & Active Recall Interface
- Minimalist distraction-free presentation during review sessions.
- Front reveal animation / toggle.
- Full keyboard support (`Space` to reveal, `1`-`4` or `A`/`H`/`G`/`E` for ratings).
- Session progress indicators (cards remaining, elapsed time, current retention rate).
- Instant review history logging.

### 4. Search & Analytics
- Full-text search across decks, card fronts, and back contents.
- Visual statistics dashboard: Daily review heatmaps, retention curves, card breakdown (New / Learning / Mature).

### 5. Local Data Persistence & Import/Export
- Fast local storage with transactional integrity.
- JSON and Anki-compatible CSV deck export and import capabilities.
- Backup and restore user data snapshots.

### 6. Optional AI-Assisted Card Generation
- Simple API prompt wrapper for generating flashcard decks from raw text/notes using standard LLM API endpoints.

---

## Out-of-Scope (V1)

- Centralized closed server database synchronization.
- Mandatory user user registration or subscription paywalls.
- Real-time multi-user live collaborative deck editing.
- Native mobile builds (iOS/Android) — mobile access is available via Web WASM.
