# Architecture Document — Engram

## Stack
- **Language:** Rust (2021 edition, stable; `rust-toolchain.toml` pins the
  toolchain with rustfmt + clippy).
- **UI:** `egui` + `eframe` 0.36 (immediate mode; native windows on
  Windows/macOS/Linux).
- **Persistence:** `serde_json` human-readable files with atomic writes.
- **IDs/time:** `uuid` (v4) and `chrono` (Utc, serde).
- **Shell conveniences:** `directories` (data dir), `rfd` (native file
  dialogs), `tracing` (logging).

## Layout

```
engram/
├── .github/workflows/ci.yml    # fmt → clippy -D warnings → tests → release (3 OSes)
├── .opencode/agents/           # domain-dev, ui-dev, test-writer, fixer,
│                               # reviewer, docs-writer, explorer, ci-runner
├── src/
│   ├── main.rs                 # binary entry: window, icon, app launch
│   ├── lib.rs                  # library root re-exports
│   ├── domain/                 # PURE: no egui, no I/O, no clock
│   │   ├── card.rs             #   Card, CardKind (Basic/Cloze), cloze render, tags
│   │   ├── deck.rs             #   Deck, DeckColor, forest summaries
│   │   ├── scheduler.rs        #   SM-2: Grade, SchedParams, SchedulingState, schedule()
│   │   ├── library.rs          #   Library aggregate: commands, search, counts, sessions
│   │   ├── session.rs          #   StudySession: queue walk, answers, summary
│   │   ├── stats.rs            #   LibraryStats: heatmap, retention, streak, forecast
│   │   ├── aigen.rs            #   offline note→draft generator
│   │   └── theme.rs            #   Theme enum (serde)
│   ├── data/                   # persistence + exchange
│   │   ├── store.rs            #   atomic JSON saves, default paths, Prefs
│   │   ├── export.rs           #   CSV export/import, JSON backup
│   │   └── error.rs            #   DataError (thiserror)
│   └── ui/                     # egui shell
│       ├── app.rs              #   EngramApp: shell, routing, menus, dialogs, keyboard
│       ├── decks.rs            #   deck tree panel + card table
│       ├── study.rs            #   study flow (reveal → grade → summary)
│       ├── stats.rs            #   tiles, heatmap, forecast
│       ├── generate.rs         #   notes → drafts → cards
│       ├── editor.rs           #   dialog state machines (card/deck/settings)
│       ├── theme.rs            #   palette + visuals application
│       └── icon.rs             #   procedural app icon (no assets)
└── tests/domain_tests.rs       # cross-cutting integration tests
```

## Principles

1. **Domain-first.** `src/domain/` compiles without egui and without the
   filesystem. All mutations are named commands returning
   `Result<T, LibraryError|CardError|DeckError>`; no panics in logic.
2. **Injected time.** Every scheduler/statistic/session function takes
   `now: DateTime<Utc>` (DECISIONS D-005); only the UI reads the clock.
3. **Command-centric mutations.** `Library` is the single aggregate owning
   decks + cards + review log; the review log is append-only (D-006).
4. **Deterministic ordering.** Queue order, deck forest order and search
   results are fully specified (D-008) so tests pin exact behaviour.
5. **UI is an integration surface.** Views borrow `&mut EngramApp`, snapshot
   the data they need, and call domain commands; they never compute
   scheduling or queue logic themselves.
6. **Atomic persistence.** Saves write a temp file then rename; every
   successful command triggers a quiet save (D-010).

## Data flow (one review)

```
Space pressed (App::logic) → revealed = true
digit 1–4 (App::logic) → EngramApp::grade_current(grade)
  └─ library.record_review(card, grade, now)      # domain: SM-2 + append log
     └─ session.answer(grade, now)                # domain: advance queue
        └─ dirty = true → ui() end → save_quiet() # data: atomic write
```

## Threading/async model

Single-threaded immediate mode; no background tasks in V1. Repaints are
requested every 0.5 s so due-state badges stay fresh.
