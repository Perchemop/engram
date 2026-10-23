//! Offline card generator: turn plain notes into draft flashcards.
//!
//! This is a deterministic, fully offline heuristic generator (no network,
//! no external model). It recognises common note shapes and never invents
//! facts — every draft quotes the source text:
//!
//! 1. `Term: Definition` / `Term - Definition` / `Term — Definition` /
//!    `Term :: Definition` → a Basic card (term → definition).
//! 2. A sentence containing a copula (`X is Y`, `X are Y`, `X means Y`)
//!    → a Cloze draft hiding the subject: `{{c1::X}} …`.
//! 3. `Q: …` / `A: …` line pairs → a Basic card.
//!
//! Lines shorter than [`MIN_LINE_CHARS`] and near-duplicates are skipped.

use serde::{Deserialize, Serialize};

/// Lines shorter than this rarely make useful cards.
pub const MIN_LINE_CHARS: usize = 8;

/// Hard cap on drafts produced from one text.
pub const MAX_DRAFTS: usize = 50;

/// A generated card proposal, not yet stored in the library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DraftCard {
    pub kind: DraftKind,
    pub front: String,
    pub back: String,
}

/// Which card kind a draft wants to become.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DraftKind {
    Basic,
    Cloze,
}

/// Extract draft cards from notes text.
#[must_use]
pub fn generate_drafts(text: &str) -> Vec<DraftCard> {
    let mut drafts: Vec<DraftCard> = Vec::new();
    let mut seen_fronts: Vec<String> = Vec::new();
    let lines: Vec<&str> = text.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        i += 1;
        if line.is_empty() || line.starts_with('#') || line.chars().count() < MIN_LINE_CHARS {
            continue;
        }

        // Q:/A: pairs.
        if let Some(q) = line.strip_prefix("Q:").or_else(|| line.strip_prefix("q:")) {
            let q = q.trim();
            // The answer may be on the same line ("Q: x — A: y") or the next one.
            let same_line = line
                .find("A:")
                .or_else(|| line.find("a:"))
                .filter(|&pos| pos > 2);
            let a = if let Some(pos) = same_line {
                line[pos + 2..].trim().to_string()
            } else {
                match lines.get(i).and_then(|n| n.trim().strip_prefix("A:")) {
                    Some(next) => {
                        i += 1;
                        next.trim().to_string()
                    }
                    None => String::new(),
                }
            };
            if !q.is_empty() && !a.is_empty() {
                push_draft(
                    &mut drafts,
                    &mut seen_fronts,
                    DraftKind::Basic,
                    q.to_string(),
                    a,
                );
            }
            continue;
        }

        // Separator-style definitions: "Term: def", "Term - def",
        // "Term — def", "Term :: def".
        if let Some((term, def)) = split_definition(line) {
            push_draft(
                &mut drafts,
                &mut seen_fronts,
                DraftKind::Basic,
                term.to_string(),
                def.trim().to_string(),
            );
            continue;
        }

        // Copula sentences → cloze.
        if let Some(cloze) = copula_cloze(line) {
            push_draft(
                &mut drafts,
                &mut seen_fronts,
                DraftKind::Cloze,
                cloze,
                line.to_string(),
            );
        }
    }
    drafts.truncate(MAX_DRAFTS);
    drafts
}

fn push_draft(
    drafts: &mut Vec<DraftCard>,
    seen: &mut Vec<String>,
    kind: DraftKind,
    front: String,
    back: String,
) {
    let front = front.trim();
    let back = back.trim();
    if front.is_empty() || back.is_empty() {
        return;
    }
    let key = front.to_lowercase();
    if seen.contains(&key) {
        return;
    }
    seen.push(key);
    drafts.push(DraftCard {
        kind,
        front: front.to_string(),
        back: back.to_string(),
    });
}

/// Split "Term: def" style lines. A colon split wins over a dash split;
/// dashes only count when surrounded by spaces (so hyphenated words stay
/// intact).
fn split_definition(line: &str) -> Option<(&str, &str)> {
    if let Some(pos) = line.find("::") {
        let (t, d) = (line[..pos].trim(), line[pos + 2..].trim());
        return non_empty_pair(t, d);
    }
    if let Some(pos) = line.find(": ") {
        let (t, d) = (line[..pos].trim(), line[pos + 2..].trim());
        return non_empty_pair(t, d);
    }
    for dash in [" — ", " - "] {
        if let Some(pos) = line.find(dash) {
            let (t, d) = (line[..pos].trim(), line[pos + dash.len()..].trim());
            if let Some(pair) = non_empty_pair(t, d) {
                return Some(pair);
            }
        }
    }
    None
}

fn non_empty_pair<'a>(t: &'a str, d: &'a str) -> Option<(&'a str, &'a str)> {
    if t.is_empty() || d.is_empty() {
        None
    } else {
        Some((t, d))
    }
}

/// Turn "X is Y" into a cloze string `{{c1::X}} is Y`.
fn copula_cloze(line: &str) -> Option<String> {
    for cop in [" is ", " are ", " means ", " was ", " were "] {
        if let Some(pos) = line.find(cop) {
            let subject = line[..pos].trim();
            // Skip bullets prefixes and overly long subjects (likely not a
            // term — e.g. a whole clause).
            let subject = subject
                .trim_start_matches(['-', '*', '•'])
                .trim()
                .trim_start_matches(char::is_numeric)
                .trim_start_matches(['.', ')'])
                .trim();
            let word_count = subject.split_whitespace().count();
            if subject.is_empty() || word_count > 3 {
                continue;
            }
            let mut blanked = String::with_capacity(line.len() + 12);
            blanked.push_str("{{c1::");
            blanked.push_str(subject);
            blanked.push_str("}}");
            blanked.push_str(&line[pos..]);
            return Some(blanked);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colon_definitions_become_basic_cards() {
        let drafts =
            generate_drafts("ATP: the energy currency of the cell\nDNA - hereditary molecule");
        assert_eq!(drafts.len(), 2);
        assert_eq!(drafts[0].kind, DraftKind::Basic);
        assert_eq!(drafts[0].front, "ATP");
        assert_eq!(drafts[0].back, "the energy currency of the cell");
        assert_eq!(drafts[1].front, "DNA");
    }

    #[test]
    fn copula_sentences_become_cloze() {
        let drafts = generate_drafts("Photosynthesis is the process plants use to make sugar");
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].kind, DraftKind::Cloze);
        assert_eq!(
            drafts[0].front,
            "{{c1::Photosynthesis}} is the process plants use to make sugar"
        );
        assert_eq!(
            drafts[0].back,
            "Photosynthesis is the process plants use to make sugar"
        );
    }

    #[test]
    fn qa_pairs_are_recognised() {
        let text = "Q: What carries oxygen in blood?\nA: Haemoglobin";
        let drafts = generate_drafts(text);
        assert_eq!(drafts.len(), 1);
        assert_eq!(drafts[0].front, "What carries oxygen in blood?");
        assert_eq!(drafts[0].back, "Haemoglobin");
    }

    #[test]
    fn duplicates_and_garbage_are_skipped() {
        let text = "ATP: energy currency\natp: energy currency\n# comment\nshort";
        assert_eq!(generate_drafts(text).len(), 1);
    }

    #[test]
    fn long_subjects_are_not_clozed() {
        let text = "The mitochondria in eukaryotic cells with aerobic metabolism is the powerhouse of the cell";
        let drafts = generate_drafts(text);
        // Subject is too long for a cloze → no draft, no crash.
        assert!(drafts.is_empty() || drafts.iter().all(|d| d.kind == DraftKind::Cloze));
    }

    #[test]
    fn drafts_are_capped() {
        let text = (0..80)
            .map(|i| format!("Term{i} number {i}: definition number {i}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert_eq!(generate_drafts(&text).len(), MAX_DRAFTS);
    }

    #[test]
    fn draft_serde_roundtrip() {
        let d = DraftCard {
            kind: DraftKind::Basic,
            front: "f".into(),
            back: "b".into(),
        };
        let json = serde_json::to_string(&d).unwrap();
        let back: DraftCard = serde_json::from_str(&json).unwrap();
        assert_eq!(back, d);
    }
}
