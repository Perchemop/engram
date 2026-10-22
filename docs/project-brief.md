# Project Brief - Engram

## A. Product Identity
1. **Working Title / Purpose:** Engram is a fast, cross-platform, clean-room spaced repetition and flashcard application inspired by synaps.app to help users learn and retain knowledge effortlessly using science-backed memory algorithms.
2. **Primary User:** Students, lifelong learners, language learners, and professionals preparing for exams or building long-term knowledge retention.
3. **Problem Solved:** Traditional studying and passive reading lead to fast decay of memory. Existing tools are often overly bloated, cloud-locked, or lack seamless offline capability and intuitive active recall interfaces.

## B. Inspiration & Scope
4. **Inspiration:** Inspired by synaps.app (clean-room black-box functionality only).
5. **Key Capabilities:**
   - Deck & Tag Management with hierarchical organizing.
   - Spaced Repetition Engine (science-backed interval scheduling: SM-2 and flexible FSRS-like parameters).
   - Rich Card Formatting (Markdown text, code blocks, active recall prompt/answer fields, cloze deletions).
   - Interactive Active Recall Study Mode with keyboard shortcuts and session statistics.
   - Offline-first local persistence with JSON/SQLite storage.
   - Optional AI Card Generation via configurable LLM API integration.
6. **Out of Scope for V1:** Proprietary subscription servers, forced account sign-ups, closed sync protocols.

## C. Technical Preferences
7. **Stack:** Pure Rust (`egui`/`eframe`, `serde`, `rusqlite` / `serde_json`).
8. **Target Platforms:** Cross-platform Desktop (Windows, macOS, Linux) + Web (WASM via `eframe`/`trunk`).
9. **Offline:** 100% functional offline.
10. **Licenses:** MIT OR Apache-2.0 permissively licensed.

## D. Quality & Process
11. **UI Polish:** Premium desktop design with modern dark/light visual tokens, responsive layout, and zero-clutter workflow.
12. **Build Safety:** CI-First execution model — all heavy builds and workspace testing run on GitHub Actions to protect local machine resources.
13. **Accessibility:** Full keyboard navigation during active recall sessions (Space/Enter to flip card, 1/2/3/4 for ratings).
