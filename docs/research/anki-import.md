# Research note — Anki text import

**Status:** APPROVED — human approval granted 2026-10-24 (guide §2.3/§8.4).
**Milestone target:** v0.3.0.
**Sources (public, capability-level only):**
- Anki manual, importing text files (tab/semicolon separated, one card per
  line, optional tags): <https://docs.ankiweb.net/importing.html>
- Engram's own CSV exchange: `src/data/export.rs`, `docs/data-model.md`

## Behaviour to implement

1. **Text import (TSV/CSV)**: read a delimited text file where each line
   is one card; auto-detect separator (TAB preferred, then `,`), support
   RFC-4180 quoting (Engram's CSV parser already does).
2. Column mapping dialog (preview first 10 rows): front column, back
   column, optional tags column, optional deck column (created on demand).
3. Cloze detection: if the front column contains `{{cN::…}}`, import as a
   Cloze card regardless of chosen type.
4. Duplicate handling: skip a row when an identical front already exists
   in the target deck (case-insensitive, whitespace-trimmed); report
   skipped count.
5. HTML: strip `<br>` → newline, strip remaining simple tags
   (`<b>/<i>` → keep text). Engram cards are plain text in V1.
6. Everything goes through the existing `Library::add_card` validation
   (invalid rows are skipped and counted, not fatal).

## Acceptance criteria

- [ ] An Anki-style TAB-separated export (`front<TAB>back<TAB>tags`)
      imports into a chosen deck with tags applied.
- [ ] Separator auto-detection: TAB file with commas inside fields imports
      correctly; comma file with quoted fields imports correctly.
- [ ] Cloze markup in the front column produces Cloze cards.
- [ ] Duplicates (identical trimmed/lowercased front in the target deck)
      are skipped and reported ("imported 120, skipped 8 duplicates").
- [ ] A header row starting with `Front`/`Question` is auto-detected and
      skipped.
- [ ] Deck column: `Parent::Child` names create decks on demand with the
      same uniqueness rules as manual creation.

## Edge cases

- Mixed line endings (CRLF) and a trailing newline: no phantom rows.
- Empty back field → row skipped as invalid (card validation).
- Very long lines (100k chars) — importer reads line-buffered; no
  unbounded memory.
- Non-UTF8 files: error reported honestly ("file is not UTF-8 text").

## Design notes (original)

- Data layer: `import_text(path, lib, deck_id, mapping, now)` in
  `data::import`; reuses `csv_records` (quote-aware) with a TAB separator
  variant. UI: import dialog with mapping preview. Importing Anki's
  binary `.anki2`/`.apkg` formats stays out of scope (binary SQLite
  internals — not a public interchange spec).
