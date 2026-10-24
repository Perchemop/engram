//! Study page: active-recall sessions with keyboard-first grading.

use chrono::Utc;
use egui::{Align, Layout, RichText, Ui};

use crate::domain::{
    preview_interval, render_cloze_blank, render_cloze_revealed, Grade, QueueKind, SchedulingState,
    Scope, Theme,
};
use crate::ui::{app::EngramApp, theme};

/// The whole study page.
pub fn show(ui: &mut Ui, app: &mut EngramApp) {
    let has_session = app.session.is_some();
    if !has_session {
        start_screen(ui, app);
        return;
    }
    let snapshot = {
        let s = app.session.as_ref().expect("checked above");
        (s.position, s.total(), s.finished, s.current())
    };
    let (position, total, finished, current) = snapshot;
    if finished {
        summary_screen(ui, app);
        return;
    }
    let Some((card_id, queue_kind)) = current else {
        app.session = None;
        app.status = "Session ended".to_string();
        return;
    };

    let t = app.prefs.theme;
    let now = Utc::now();

    // Snapshot card data so the borrow is released before mutations.
    let (prompt, answer, deck_name, tags, sched) = {
        let Some(card) = app.library.card(card_id) else {
            app.session = None;
            app.status = "Card no longer exists — session ended".to_string();
            return;
        };
        (
            card.kind.prompt(),
            card.kind.answer(),
            app.library
                .deck_display_name(card.deck_id)
                .unwrap_or_default(),
            card.tags.join(", "),
            card.sched.clone(),
        )
    };
    let params = app.library.settings.sched.clone();
    let is_cloze = matches!(
        app.library.card(card_id).map(|c| &c.kind),
        Some(crate::domain::CardKind::Cloze { .. })
    );

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{} · {}", deck_name, queue_kind.label()))
                .weak()
                .color(theme::muted(t)),
        );
        if !tags.is_empty() {
            ui.label(RichText::new(tags).weak().color(theme::muted(t)));
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.small_button("End session").clicked() {
                app.end_session();
            }
        });
    });
    ui.add_space(4.0);
    ui.add(
        egui::ProgressBar::new(if total == 0 {
            1.0
        } else {
            position as f32 / total as f32
        })
        .show_percentage()
        .text(format!("{} / {}", position + 1, total)),
    );
    ui.add_space(16.0);

    // Prompt.
    ui.vertical_centered(|ui| {
        let prompt_text = if is_cloze {
            render_cloze_blank(&prompt)
        } else {
            prompt.clone()
        };
        ui.label(RichText::new(prompt_text).size(24.0).strong());
        ui.add_space(12.0);

        if !app.revealed {
            if ui
                .add(egui::Button::new(
                    RichText::new("Show answer  (Space)").size(16.0),
                ))
                .clicked()
            {
                app.revealed = true;
            }
        } else {
            let answer_text = if is_cloze {
                render_cloze_revealed(&answer)
            } else {
                answer.clone()
            };
            ui.separator();
            ui.label(RichText::new(answer_text).size(20.0));
            ui.add_space(12.0);
            ui.label(
                RichText::new("How well did you recall it?")
                    .weak()
                    .color(theme::muted(t)),
            );
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.with_layout(
                    egui::Layout::top_down_justified(egui::Align::Center),
                    |ui| {
                        grade_buttons(
                            ui,
                            app,
                            &sched,
                            &params,
                            now,
                            t,
                            queue_kind == QueueKind::Cram,
                        );
                    },
                );
            });
        }
    });
    ui.add_space(8.0);
    if queue_kind == QueueKind::Cram {
        ui.label(
            RichText::new("Cram: answers are logged but your schedule is not changed.")
                .weak()
                .color(theme::warn(t)),
        );
    }
    ui.label(
        RichText::new("Space: reveal · 1 Again · 2 Hard · 3 Good · 4 Easy · Esc: end session")
            .weak()
            .color(theme::muted(t)),
    );
}

fn grade_buttons(
    ui: &mut Ui,
    app: &mut EngramApp,
    sched: &SchedulingState,
    params: &crate::domain::SchedParams,
    now: chrono::DateTime<Utc>,
    t: Theme,
    cram: bool,
) {
    let previews: Vec<(Grade, String)> = if cram {
        // Cram answers never move the schedule, so interval previews would
        // be misleading — show plain grade labels.
        Grade::ALL.iter().map(|g| (*g, String::new())).collect()
    } else {
        Grade::ALL
            .iter()
            .map(|g| (*g, preview_interval(now, params, sched, *g)))
            .collect()
    };
    let colors = [
        theme::danger(t),
        theme::warn(t),
        theme::ok(t),
        theme::ACCENT,
    ];
    for (i, (grade, preview)) in previews.iter().enumerate() {
        let digit = i + 1;
        let text = if preview.is_empty() {
            format!("{digit} · {}", grade.label())
        } else {
            format!("{digit} · {} ({preview})", grade.label())
        };
        let label = RichText::new(text).color(colors[i]);
        if ui
            .add(egui::Button::new(label).min_size(egui::vec2(110.0, 34.0)))
            .clicked()
        {
            app.grade_current(*grade);
        }
    }
}

/// The screen shown when no session is running.
fn start_screen(ui: &mut Ui, app: &mut EngramApp) {
    let t = app.prefs.theme;
    let now = Utc::now();
    ui.add_space(16.0);
    ui.heading("Study");
    ui.add_space(8.0);

    // Scope picker.
    let options: Vec<(String, Scope)> = {
        let mut v = vec![("All cards".to_string(), Scope::All)];
        for summary in app.library.deck_summaries(now) {
            v.push((summary.name.clone(), Scope::Deck(summary.id)));
        }
        v
    };
    ui.horizontal(|ui| {
        ui.label("Study:");
        egui::ComboBox::from_id_salt("study_scope")
            .selected_text(scope_label(&options, app.study_scope))
            .show_ui(ui, |ui| {
                for (label, scope) in &options {
                    ui.selectable_value(&mut app.study_scope, *scope, label.clone());
                }
            });
    });

    let counts = app.library.due_counts(app.study_scope, now);
    let total_in_scope = app.library.cards_in_scope(app.study_scope).len();
    ui.add_space(4.0);
    let ((eff_new, eff_review), source) = app.library.effective_limits(app.study_scope);
    let limit_caption = match source {
        Some(id) => format!(
            " (cap from deck “{}”)",
            app.library
                .deck(id)
                .map(|d| d.name.clone())
                .unwrap_or_default()
        ),
        None => " (library default)".to_string(),
    };
    ui.label(format!(
        "{} due · {} in learning steps · {} new queued · {} card(s) in scope",
        counts.review, counts.learning, counts.new_cards, total_in_scope
    ));
    ui.label(
        RichText::new(format!(
            "Daily limits: {eff_new} new{limit_caption}, {eff_review} reviews · queue order: {}.",
            app.library.settings.queue_order.label()
        ))
        .weak()
        .color(theme::muted(t)),
    );
    ui.add_space(8.0);

    // Cram mode: study the whole scope ignoring the scheduler; answers are
    // logged but never mutate the schedule (docs/research/cram-sessions.md).
    ui.horizontal(|ui| {
        ui.checkbox(&mut app.cram_mode, "Cram whole scope (ignores schedule)");
        if app.cram_mode {
            ui.label("max cards:");
            ui.add(egui::DragValue::new(&mut app.cram_limit).range(1..=10_000));
        }
    });
    ui.add_space(6.0);

    let cram_ok = app.cram_mode && total_in_scope > 0;
    let can_start = counts.total() > 0;
    let button_text = if app.cram_mode {
        "Start cram"
    } else {
        "Start studying"
    };
    if ui
        .add_enabled(
            can_start || cram_ok,
            egui::Button::new(RichText::new(button_text).size(18.0))
                .min_size(egui::vec2(180.0, 40.0)),
        )
        .clicked()
    {
        if app.cram_mode {
            app.start_cram(app.study_scope, app.cram_limit);
        } else {
            app.start_study(app.study_scope);
        }
    }
    if !can_start && !cram_ok {
        ui.label(
            RichText::new("Nothing is due in this scope right now — try cram mode.")
                .weak()
                .color(theme::muted(t)),
        );
    }
}

fn scope_label(options: &[(String, Scope)], current: Scope) -> String {
    options
        .iter()
        .find(|(_, s)| *s == current)
        .map(|(l, _)| l.clone())
        .unwrap_or_else(|| "All cards".to_string())
}

/// The end-of-session screen.
fn summary_screen(ui: &mut Ui, app: &mut EngramApp) {
    let t = app.prefs.theme;
    let Some(session) = app.session.as_ref() else {
        return;
    };
    let summary = session.summary();

    ui.add_space(24.0);
    ui.vertical_centered(|ui| {
        ui.heading("Session complete");
        ui.add_space(8.0);
        ui.label(RichText::new(format!("{} card(s) reviewed", summary.total)).size(18.0));
        ui.add_space(8.0);
        egui::Grid::new("session_summary")
            .num_columns(2)
            .spacing([12.0, 4.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Again").color(theme::danger(t)));
                ui.label(summary.again.to_string());
                ui.end_row();
                ui.label(RichText::new("Hard").color(theme::warn(t)));
                ui.label(summary.hard.to_string());
                ui.end_row();
                ui.label(RichText::new("Good").color(theme::ok(t)));
                ui.label(summary.good.to_string());
                ui.end_row();
                ui.label("Easy");
                ui.label(summary.easy.to_string());
                ui.end_row();
                ui.label("Recall rate");
                ui.label(format!("{:.0}%", summary.retention() * 100.0));
                ui.end_row();
                ui.label("Time");
                ui.label(format!(
                    "{}m {:02}s",
                    summary.elapsed_seconds / 60,
                    summary.elapsed_seconds % 60
                ));
                ui.end_row();
            });
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui.button("Study again").clicked() {
                let scope = app.study_scope;
                app.session = None;
                app.start_study(scope);
            }
            if ui.button("Back to decks").clicked() {
                app.session = None;
                app.page = crate::ui::app::Page::Decks;
                app.status = "Session closed".to_string();
            }
        });
    });
}
