# Engram Brand

Original visual identity for Engram. Nothing here is derived from another
product's mark; the name "engram" (the physical trace a memory leaves) and
the design below were created for this project.

## Name

**Engram** — short, memorable, and directly meaningful: an engram is the
trace a memory leaves in the brain, which is what active recall builds.
Used as: product name "Engram", binary/crate `engram`, window title
"Engram — spaced repetition & active recall".

## The mark

A **violet rounded square** carrying a **white ring with a highlighted arc
and a solid nucleus dot** — an abstract "memory trace lit up by recall".

| Element | Meaning |
|---|---|
| Violet rounded square | The app surface; violet is Engram's accent colour throughout the UI (`#7C5CFF` family) |
| White ring | The memory trace itself — an engram |
| Bright arc (upper right) | The moment of successful recall, lighting up part of the trace |
| Solid nucleus dot | The memory at the centre of the trace |

## Colour tokens

| Token | RGB | Use |
|---|---|---|
| Accent (violet) | `124, 92, 255` | Selection, links, primary buttons, heatmap |
| Accent strong | `100, 70, 230` | Pressed/hover states |
| Dark surface | `24, 22, 34` / `34, 32, 47` | Dark theme panels |
| Paper | `248, 248, 251` | Light theme panels |

Deck colour tokens (violet, blue, teal, green, amber, orange, red, slate)
are defined in `src/domain/deck.rs` (`DeckColor`) and mapped to egui colours
in `src/ui/theme.rs`.

## Assets

- `assets/engram.svg` — master vector icon (rounded square + trace mark).
- The window icon is generated procedurally from the same design in
  `src/ui/icon.rs` (no raster assets shipped).
- The Linux desktop entry uses the SVG installed to
  `~/.local/share/icons/hicolor/scalable/apps/engram.svg`.

Licence for all brand assets: same as the project (MIT OR Apache-2.0),
recorded in `ATTRIBUTION.md`.
