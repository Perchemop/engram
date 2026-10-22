//! Generate page: offline heuristic card generation from plain notes.

use chrono::Utc;
use egui::{RichText, Ui};

use crate::domain::{generate_drafts, CardKind, DraftCard, DraftKind};
use crate::ui::{app::EngramApp, decks::truncate, theme};

/// Generator page state (session-only).
#[derive(Debug, Default, Clone, PartialEq)]
pub struct GenerateState {
    pub input: String,
    pub drafts: Vec<(DraftCard, bool)>,
    pub generated: bool,
    pub target: Option<uuid::Uuid>,
}

/// The whole generate page.
pub fn show(ui: &mut Ui, app: &mut EngramApp) {
    let t = app.prefs.theme;
    ui.add_space(8.0);
    ui.heading("Generate cards from notes");
    ui.add_space(4.0);
    ui.label(
        RichText::new(
            "Paste plain notes and Engram drafts flashcards offline (no AI service, no network). \
Recognised shapes:\n\
• “Term: Definition”, “Term - Definition”, “Term — Definition”, “Term :: Definition” → basic cards\n\
• “Q: …” / “A: …” line pairs → basic cards\n\
• Sentences like “X is Y” → cloze cards with the subject hidden",
        )
        .weak()
        .color(theme::muted(t)),
    );
    ui.add_space(8.0);

    ui.add(
        egui::TextEdit::multiline(&mut app.gen.input)
            .hint_text("Paste your notes here…\n\nATP: the energy currency of the cell\nPhotosynthesis is the process plants use to make sugar")
            .desired_rows(8),
    );
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        if ui.button("Generate draft cards").clicked() {
            let drafts = generate_drafts(&app.gen.input);
            app.gen.drafts = drafts.into_iter().map(|d| (d, true)).collect();
            app.gen.generated = true;
            app.status = format!(
                "Generated {} draft(s)",
                app.gen.drafts.iter().filter(|(_, on)| *on).count()
            );
        }
        if !app.gen.drafts.is_empty() && ui.button("Clear drafts").clicked() {
            app.gen.drafts.clear();
            app.gen.generated = false;
        }
    });

    if app.gen.drafts.is_empty() {
        if app.gen.generated {
            ui.label(
                RichText::new(
                    "No draft cards found — try “Term: Definition” lines or “X is Y” sentences.",
                )
                .weak()
                .color(theme::warn(t)),
            );
        }
        return;
    }

    ui.separator();
    ui.add_space(4.0);

    // Target deck picker.
    let decks: Vec<(String, uuid::Uuid)> = app
        .library
        .decks
        .iter()
        .map(|d| (d.name.clone(), d.id))
        .collect();
    if app.gen.target.is_none() {
        app.gen.target = app
            .selected_deck
            .or_else(|| app.library.decks.first().map(|d| d.id));
    }
    ui.horizontal(|ui| {
        ui.label("Add to deck:");
        egui::ComboBox::from_id_salt("gen_target_deck")
            .selected_text(
                app.gen
                    .target
                    .and_then(|id| decks.iter().find(|(_, did)| *did == id))
                    .map(|(n, _)| n.clone())
                    .unwrap_or_else(|| "—".to_string()),
            )
            .show_ui(ui, |ui| {
                for (name, id) in &decks {
                    let mut sel = app.gen.target == Some(*id);
                    if ui.selectable_label(sel, name.clone()).clicked() {
                        sel = true;
                    }
                    if sel {
                        app.gen.target = Some(*id);
                    }
                }
            });
    });

    ui.add_space(8.0);
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            for (i, (draft, on)) in app.gen.drafts.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.checkbox(on, "");
                    let kind = match draft.kind {
                        DraftKind::Cloze => "Cloze",
                        DraftKind::Basic => "Basic",
                    };
                    ui.label(RichText::new(kind).weak());
                    match draft.kind {
                        DraftKind::Cloze => {
                            ui.label(truncate(&draft.front, 90));
                        }
                        DraftKind::Basic => {
                            ui.label(truncate(&draft.front, 40));
                            ui.label(RichText::new("→").weak());
                            ui.label(truncate(&draft.back, 40));
                        }
                    }
                });
                let _ = i;
            }
        });

    ui.add_space(8.0);
    let selected = app.gen.drafts.iter().filter(|(_, on)| *on).count();
    ui.horizontal(|ui| {
        let enabled = selected > 0 && app.gen.target.is_some();
        if ui
            .add_enabled(
                enabled,
                egui::Button::new(format!("Add {selected} card(s)")),
            )
            .clicked()
        {
            add_drafts_to_library(app);
        }
        if ui.button("Select all").clicked() {
            for (_, on) in &mut app.gen.drafts {
                *on = true;
            }
        }
        if ui.button("Deselect all").clicked() {
            for (_, on) in &mut app.gen.drafts {
                *on = false;
            }
        }
    });
}

fn add_drafts_to_library(app: &mut EngramApp) {
    let now = Utc::now();
    let Some(deck) = app.gen.target else { return };
    let mut added = 0usize;
    let drafts: Vec<DraftCard> = app
        .gen
        .drafts
        .iter()
        .filter(|(_, on)| *on)
        .map(|(d, _)| d.clone())
        .collect();
    for draft in drafts {
        let kind = match draft.kind {
            DraftKind::Cloze => CardKind::Cloze { text: draft.front },
            DraftKind::Basic => CardKind::Basic {
                front: draft.front,
                back: draft.back,
            },
        };
        if app.library.add_card(deck, kind, vec![], now).is_ok() {
            added += 1;
        }
    }
    app.gen.drafts.retain(|(_, on)| !*on);
    app.gen.generated = app.gen.drafts.is_empty();
    app.dirty = true;
    app.status = format!("Added {added} card(s) to the deck");
}
