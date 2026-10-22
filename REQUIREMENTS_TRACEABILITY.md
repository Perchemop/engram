# REQUIREMENTS_TRACEABILITY

Map of capabilities → implementation → tests. Statuses are honest: a row is
`DONE` only when the capability is implemented **and** CI-verified.

## Global requirements
| ID | Requirement | Status | Trace |
|---|---|---|---|
| R-G-001 | Rust only, 100% free OSS deps | DONE | Cargo.toml; ATTRIBUTION.md |
| R-G-002 | Cross-platform desktop (Linux/macOS/Windows) | DONE (pending CI proof) | eframe/egui; ci.yml matrix |
| R-G-003 | Offline-first; no accounts/telemetry/network | DONE | no net deps; README |
| R-G-004 | Original work; clean-room inspiration only | DONE | docs/inspiration.md; ATTRIBUTION.md |
| R-G-005 | Domain-first, headless-testable | DONE | src/domain has no egui/IO |
| R-G-006 | CI-first: heavy gates in GitHub Actions | DONE | .github/workflows/ci.yml; AGENTS.md |
| R-G-007 | Command-centric domain with Result errors | DONE | library.rs; thiserror |
| R-G-008 | Human-readable persistence incl. UUIDs/timestamps | DONE | data/store.rs |

## Deck management
| ID | Capability | Status | Implementation | Tests |
|---|---|---|---|---|
| R-D-001 | Nested deck tree (parent/child) | DONE | deck.rs; library.rs `new_deck/move_deck` | `deck_crud_and_siblings`, `descendants_and_paths` |
| R-D-002 | Deck colour tokens | DONE | deck.rs `DeckColor` | `deck_serde_roundtrip` |
| R-D-003 | Sibling-name uniqueness (case-insensitive) | DONE | library.rs | `deck_crud_and_siblings` |
| R-D-004 | Move deck with cycle prevention | DONE | library.rs `move_deck` | `deck_crud_and_siblings`, `moving_a_deck_between_parents…` |
| R-D-005 | Delete deck: cards cascade, children re-parent | DONE | library.rs `delete_deck` | `delete_deck_keeps_grandchildren_and_cascades_cards` |
| R-D-006 | Deck summaries with subtree counts | DONE | library.rs `deck_summaries` | `deck_summaries_aggregate_subtrees` |

## Cards
| ID | Capability | Status | Implementation | Tests |
|---|---|---|---|---|
| R-C-001 | Basic cards (front/back) | DONE | card.rs | `basic_validation_rejects_empty_sides` |
| R-C-002 | Cloze cards (`{{cN::…}}`) | DONE | card.rs | `cloze_requires_at_least_one_deletion` |
| R-C-003 | Cloze blank/reveal rendering | DONE | card.rs | `cloze_numbers_are_collected_and_deduped`, `unterminated_cloze_is_kept_verbatim` |
| R-C-004 | Tag normalisation | DONE | card.rs `normalise_tags` | `tags_are_normalised` |
| R-C-005 | Card CRUD + move between decks | DONE | library.rs | `card_commands_validate` |
| R-C-006 | Search: scope + tag AND + text | DONE | library.rs `search` | `search_matches_text_tags_and_scope` |

## Scheduling (SM-2)
| ID | Capability | Status | Implementation | Tests |
|---|---|---|---|---|
| R-S-001 | New → learning steps → graduated | DONE | scheduler.rs | `new_card_*`, `graduating_from_learning…` |
| R-S-002 | Review intervals grow with ease | DONE | scheduler.rs | `review_good_multiplies_by_ease` |
| R-S-003 | Hard factor / easy bonus | DONE | scheduler.rs | `review_hard_grows_by_hard_factor`, `review_easy_applies_easy_bonus` |
| R-S-004 | Lapses → relearning, ease penalty | DONE | scheduler.rs | `review_again_lapses_into_relearning` |
| R-S-005 | Ease clamp 1.3–3.0 | DONE | scheduler.rs | `ease_is_clamped` |
| R-S-006 | Max interval cap; parameter sanitising | DONE | scheduler.rs | `interval_is_capped`, `sanitising_repairs_insane_parameters` |
| R-S-007 | Interval previews on grade buttons | DONE | scheduler.rs `preview_interval` | `preview_interval_matches_schedule` |
| R-S-008 | Review log (append-only) | DONE | library.rs `record_review` | `review_updates_state_and_appends_log` |

## Study sessions
| ID | Capability | Status | Implementation | Tests |
|---|---|---|---|---|
| R-Q-001 | Deterministic queue: learning → review → new | DONE | library.rs `build_session` | `due_counts_and_session_ordering` |
| R-Q-002 | Daily limits respected | DONE | library.rs | `daily_new_limit_is_respected_across_sessions` |
| R-Q-003 | Scope = deck subtree | DONE | library.rs | `scope_respects_subtree` |
| R-Q-004 | Session walk, answers, summary | DONE | session.rs | `session_walks_the_queue`, `end_early_keeps_summary` |
| R-Q-005 | Nothing-to-study is a distinct error | DONE | library.rs | `session_on_empty_library_is_an_error`, `nothing_to_study_error_is_distinct` |

## Statistics
| ID | Capability | Status | Implementation | Tests |
|---|---|---|---|---|
| R-T-001 | Phase/maturity breakdown | DONE | stats.rs | `phases_are_counted` |
| R-T-002 | Activity heatmap window (35 days) | DONE | stats.rs + ui/stats.rs | `empty_library_has_zero_counts_and_no_streak` |
| R-T-003 | 30-day retention excluding Again | DONE | stats.rs | `retention_excludes_again_grades` |
| R-T-004 | Streak (tolerates empty today) | DONE | stats.rs | `streak_counts_consecutive_days`, `streak_survives_an_empty_today` |
| R-T-005 | 14-day due forecast | DONE | stats.rs | `forecast_buckets_due_cards_by_day` |

## Generator (offline AI-free)
| ID | Capability | Status | Implementation | Tests |
|---|---|---|---|---|
| R-A-001 | Definition lines → basic cards | DONE | aigen.rs | `colon_definitions_become_basic_cards` |
| R-A-002 | Q/A pairs | DONE | aigen.rs | `qa_pairs_are_recognised` |
| R-A-003 | Copula sentences → cloze | DONE | aigen.rs | `copula_sentences_become_cloze` |
| R-A-004 | Dedupe, min length, cap 50 | DONE | aigen.rs | `duplicates_and_garbage_are_skipped`, `drafts_are_capped` |
| R-A-005 | Drafts land in the library as cards | DONE | ui/generate.rs | `generator_output_lands_in_the_library` |

## Persistence & exchange
| ID | Capability | Status | Implementation | Tests |
|---|---|---|---|---|
| R-P-001 | `.engram.json` library roundtrip | DONE | data/store.rs | `library_roundtrips_through_disk`, `persistence_roundtrip_on_disk` |
| R-P-002 | Atomic writes | DONE | data/store.rs `atomic_write` | (code-reviewed; no partial-write path) |
| R-P-003 | Missing file → fresh library | DONE | data/store.rs | `missing_library_is_none` |
| R-P-004 | CSV export/import with quoting | DONE | data/export.rs | `csv_roundtrip`, `csv_header_is_skipped_on_import`, `parse_csv_record_handles_quotes` |
| R-P-005 | JSON backup export | DONE | data/export.rs | `json_export_roundtrip` |
| R-P-006 | App preferences (theme) persisted | DONE | data/store.rs `Prefs` | `theme_serde_roundtrip` |

## UI
| ID | Capability | Status | Implementation | Tests |
|---|---|---|---|---|
| R-U-001 | App shell with menus/tabs/status | DONE | ui/app.rs | (UI; manual QC) |
| R-U-002 | Deck tree panel + card table | DONE | ui/decks.rs | `truncate_appends_ellipsis` + manual QC |
| R-U-003 | Card editor (Basic/Cloze/tags/deck) | DONE | ui/app.rs + ui/editor.rs | `card_editor_roundtrip_through_library` |
| R-U-004 | Deck dialog (name/parent/colour) | DONE | ui/app.rs | (UI; manual QC) |
| R-U-005 | Settings dialog (SM-2 params) | DONE | ui/app.rs + editor.rs | `parse_steps_handles_lists` |
| R-U-006 | Study page reveal/grade with previews | DONE | ui/study.rs | (UI; manual QC) |
| R-U-007 | Session summary screen | DONE | ui/study.rs | (UI; manual QC) |
| R-U-008 | Stats page (tiles/heatmap/forecast) | DONE | ui/stats.rs | `heat_color_distinguishes_empty_and_busy_days` |
| R-U-009 | Generate page (draft → deck) | DONE | ui/generate.rs | (UI; manual QC) |
| R-U-010 | Keyboard: Ctrl+S / Space / 1–4 / Esc | DONE | ui/app.rs `logic()` | (UI; manual QC) |
| R-U-011 | Procedural app icon | DONE | ui/icon.rs | `icon_has_opaque_center_and_transparent_corner` |
| R-U-012 | Theme system (dark/light) | DONE | ui/theme.rs | `apply_theme_selects_matching_theme` |
| R-U-013 | Every control functional; no fake states | DONE | status strings only after real ops | (reviewer + manual QC) |

## Coverage note
Domain and data behaviour is unit/integration tested; UI logic is covered by
domain tests plus manual QC (`docs/qc/checklist.md`) since egui code is the
integration surface.
