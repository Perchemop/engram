//! Decks page: the deck tree (left panel) and the card list (central).

use chrono::{DateTime, Utc};
use egui::{Align, Color32, Layout, RichText, Sense, Ui, Vec2};

use crate::domain::{
    format_interval, render_cloze_blank, Card, CardId, CardKind, LearnPhase, Scope, Theme,
};
use crate::ui::{app::EngramApp, editor, theme};

/// Maximum rows rendered in the card table before a note is shown.
const MAX_ROWS: usize = 400;

/// Left panel: the deck tree.
pub fn show_tree(ui: &mut Ui, app: &mut EngramApp) {
    let now = Utc::now();
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.strong("Decks");
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.small_button("+ Deck").clicked() {
                let parent = app.selected_deck;
                app.deck_dialog = Some(editor::DeckDialog::new_deck(parent));
            }
        });
    });
    ui.separator();

    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            // Virtual "All cards" root.
            let all = app.library.due_counts(Scope::All, now);
            let selected = app.selected_deck.is_none();
            let label = format!("All cards ({})", all.learning + all.review);
            if ui
                .add(
                    egui::Button::new(if selected {
                        RichText::new(&label).strong()
                    } else {
                        RichText::new(&label)
                    })
                    .fill(Color32::TRANSPARENT),
                )
                .clicked()
            {
                app.selected_deck = None;
            }
            ui.add_space(4.0);

            let summaries = app.library.deck_summaries(now);
            for summary in summaries {
                let indent = 14.0 * summary.depth as f32;
                ui.horizontal(|ui| {
                    ui.add_space(indent);
                    let dot = summary
                        .color
                        .map(theme::deck_color)
                        .unwrap_or_else(|| Color32::from_rgb(120, 120, 140));
                    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                    ui.painter().circle_filled(rect.center(), 4.0, dot);
                    resp.on_hover_text(summary.name.clone());

                    let name = if summary.due_count > 0 {
                        format!("{} ({})", summary.name, summary.due_count)
                    } else {
                        summary.name.clone()
                    };
                    if ui
                        .add(
                            egui::Button::new(RichText::new(&name).size(13.0))
                                .fill(Color32::TRANSPARENT),
                        )
                        .on_hover_text(format!("{} card(s)", summary.card_count))
                        .clicked()
                    {
                        app.selected_deck = Some(summary.id);
                    }

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button("+").on_hover_text("New sub-deck").clicked() {
                            app.deck_dialog = Some(editor::DeckDialog::new_deck(Some(summary.id)));
                        }
                        if ui
                            .small_button("▶")
                            .on_hover_text("Study this deck")
                            .clicked()
                        {
                            app.start_study(Scope::Deck(summary.id));
                        }
                    });
                });
            }

            if app.library.decks.is_empty() {
                ui.label(RichText::new("No decks yet — create one with “+ Deck”.").weak());
            }
        });
}

/// Central panel: the card list of the selected scope.
pub fn show_cards(ui: &mut Ui, app: &mut EngramApp) {
    let now = Utc::now();
    let scope = Scope::from_option(app.selected_deck);
    let scope_name = match app.selected_deck {
        None => "All cards".to_string(),
        Some(id) => app
            .library
            .deck_display_name(id)
            .unwrap_or_else(|_| "Unknown deck".to_string()),
    };
    let t = app.prefs.theme;
    let has_decks = !app.library.decks.is_empty();

    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.heading(&scope_name);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .add_enabled(has_decks, egui::Button::new("＋ New Card"))
                .clicked()
            {
                let deck = app
                    .selected_deck
                    .or_else(|| app.library.decks.first().map(|d| d.id));
                if let Some(deck) = deck {
                    app.card_editor = Some(editor::CardEditor::new_card(deck));
                }
            }
            if ui
                .add_enabled(app.selected_deck.is_some(), egui::Button::new("Edit deck"))
                .clicked()
            {
                if let Some(id) = app.selected_deck {
                    app.deck_dialog = editor::DeckDialog::edit_deck(id, &app.library);
                }
            }
            if ui
                .add_enabled(
                    app.selected_deck.is_some(),
                    egui::Button::new("Delete deck"),
                )
                .clicked()
            {
                app.confirm_delete_deck = app.selected_deck;
            }
            if ui.button("Study").clicked() {
                app.start_study(scope);
            }
        });
    });

    if let Some(id) = app.confirm_delete_deck {
        let name = app
            .library
            .deck(id)
            .map(|d| d.name.clone())
            .unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!(
                    "Delete “{name}” and its cards? Sub-decks are kept and re-parented."
                ))
                .color(theme::warn(t)),
            );
            if ui.button("Confirm").clicked() {
                match app.library.delete_deck(id) {
                    Ok(n) => {
                        app.status = format!("Deck deleted ({n} card(s) removed)");
                        app.selected_deck = None;
                        app.confirm_delete_deck = None;
                        app.dirty = true;
                    }
                    Err(e) => {
                        app.status = format!("Delete failed: {e}");
                        app.confirm_delete_deck = None;
                    }
                }
            }
            if ui.button("Cancel").clicked() {
                app.confirm_delete_deck = None;
            }
        });
    }

    ui.separator();

    // Filter row: search + tag toggles.
    ui.horizontal(|ui| {
        ui.label("Search:");
        if ui
            .add(
                egui::TextEdit::singleline(&mut app.filter.query)
                    .hint_text("front, back or cloze text…")
                    .desired_width(240.0),
            )
            .changed()
        {
            app.selected_cards.clear();
        }
        let count = filtered_ids(app, now).len();
        ui.label(RichText::new(format!("{count} card(s)")).weak());
    });

    let tags = app.library.all_tags();
    if !tags.is_empty() {
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Tags:").weak());
            for tag in tags {
                let mut active = app.filter.tags.contains(&tag);
                if ui.toggle_value(&mut active, &tag).changed() {
                    if active {
                        app.filter.tags.push(tag.clone());
                    } else {
                        app.filter.tags.retain(|t| *t != tag);
                    }
                }
            }
            if !app.filter.tags.is_empty() && ui.small_button("clear").clicked() {
                app.filter.tags.clear();
            }
        });
    }

    ui.add_space(4.0);

    // Bulk actions row.
    ui.horizontal(|ui| {
        if !app.selected_cards.is_empty() {
            ui.label(format!("{} selected", app.selected_cards.len()));
            if ui.button("Delete selected").clicked() {
                let ids = app.selected_cards.clone();
                for id in ids {
                    let _ = app.library.delete_card(id);
                }
                app.status = "Selected card(s) deleted".to_string();
                app.selected_cards.clear();
                app.dirty = true;
            }
            if ui
                .button("Suspend")
                .on_hover_text("Pause the selected cards")
                .clicked()
            {
                let now = Utc::now();
                let ids = app.selected_cards.clone();
                for id in ids {
                    let _ = app.library.set_card_suspended(id, true, now);
                }
                app.status = "Selected card(s) suspended".to_string();
                app.dirty = true;
            }
            if ui
                .button("Resume")
                .on_hover_text("Un-pause the selected cards")
                .clicked()
            {
                let now = Utc::now();
                let ids = app.selected_cards.clone();
                for id in ids {
                    let _ = app.library.set_card_suspended(id, false, now);
                }
                app.status = "Selected card(s) resumed".to_string();
                app.dirty = true;
            }
            if ui
                .button("Bury 1 day")
                .on_hover_text("Postpone the selected cards for one day")
                .clicked()
            {
                let now = Utc::now();
                let until = now + chrono::Duration::days(1);
                let ids = app.selected_cards.clone();
                for id in ids {
                    let _ = app.library.bury_card_until(id, until, now);
                }
                app.status = "Selected card(s) buried for a day".to_string();
                app.dirty = true;
            }
            if ui.button("Clear selection").clicked() {
                app.selected_cards.clear();
            }
        }
    });

    ui.separator();

    // Card table.
    let ids = filtered_ids(app, now);
    egui::ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| {
            egui::Grid::new("card_table")
                .num_columns(6)
                .spacing([10.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.strong("");
                    ui.strong("Type");
                    ui.strong("Card");
                    ui.strong("Tags");
                    ui.strong("Due");
                    ui.strong("");
                    ui.end_row();

                    for id in ids.iter().take(MAX_ROWS).copied() {
                        let Some(card) = app.library.card(id) else {
                            continue;
                        };
                        let checked = app.selected_cards.contains(&id);
                        let mut row_checked = checked;
                        if ui.checkbox(&mut row_checked, "").changed() && row_checked != checked {
                            if row_checked {
                                app.selected_cards.push(id);
                            } else {
                                app.selected_cards.retain(|c| *c != id);
                            }
                        }
                        ui.label(RichText::new(card.kind.label()).weak());
                        let prompt = match &card.kind {
                            CardKind::Basic { front, .. } => front.clone(),
                            CardKind::Cloze { text } => render_cloze_blank(text),
                        };
                        ui.label(truncate(&prompt, 72));
                        ui.label(
                            RichText::new(card.tags.join(", "))
                                .weak()
                                .color(theme::muted(t)),
                        );
                        ui.label(due_label(&card, now, t));
                        ui.horizontal(|ui| {
                            if ui.small_button("Edit").clicked() {
                                app.card_editor = editor::CardEditor::edit_card(id, &app.library);
                            }
                            if ui
                                .small_button("Log")
                                .on_hover_text("Review history")
                                .clicked()
                            {
                                app.history_card = Some(id);
                            }
                            if ui.small_button("✕").on_hover_text("Delete card").clicked() {
                                let _ = app.library.delete_card(id);
                                app.selected_cards.retain(|c| *c != id);
                                app.dirty = true;
                                app.status = "Card deleted".to_string();
                            }
                        });
                        ui.end_row();
                    }
                });
            if ids.len() > MAX_ROWS {
                ui.label(
                    RichText::new(format!(
                        "Showing the first {MAX_ROWS} of {} cards — refine the search to see more.",
                        ids.len()
                    ))
                    .weak(),
                );
            }
            if ids.is_empty() {
                ui.label(RichText::new("No cards match. Create one with “＋ New Card”.").weak());
            }
        });
}

fn filtered_ids(app: &EngramApp, _now: DateTime<Utc>) -> Vec<CardId> {
    if app.filter.is_empty() {
        app.library
            .cards_in_scope(Scope::from_option(app.selected_deck))
            .iter()
            .map(|c| c.id)
            .collect()
    } else {
        let mut f = app.filter.clone();
        f.deck = app.selected_deck;
        app.library.search(&f)
    }
}

fn due_label(card: &Card, now: DateTime<Utc>, t: Theme) -> RichText {
    if card.suspended {
        return RichText::new("suspended").color(theme::muted(t));
    }
    if card.buried_until.is_some_and(|until| until > now) {
        return RichText::new("buried").color(theme::muted(t));
    }
    match card.sched.phase {
        LearnPhase::New => RichText::new("new").color(theme::muted(t)),
        _ => {
            if card.sched.due <= now {
                RichText::new("due").color(theme::warn(t))
            } else {
                let days = (card.sched.due - now).num_seconds() as f64 / 86_400.0;
                RichText::new(format!("in {}", format_interval(days)))
            }
        }
    }
}

/// Shorten text for table cells (char-safe, appends an ellipsis).
#[must_use]
pub fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        format!("{cut}…")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_appends_ellipsis() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("abcdefghijk", 5), "abcd…");
        assert_eq!(truncate("", 5), "");
    }
}
