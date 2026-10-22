//! Stats page: activity heatmap, retention, maturity and due forecast.

use chrono::{Datelike, NaiveDate, Utc};
use egui::{Color32, RichText, Sense, Ui, Vec2};

use crate::domain::{stats, Theme};
use crate::ui::{app::EngramApp, theme};

/// The whole stats page.
pub fn show(ui: &mut Ui, app: &mut EngramApp) {
    let t = app.prefs.theme;
    let now = Utc::now();
    let s = stats::compute(
        &app.library,
        now,
        app.library.settings.sched.mature_threshold_days,
    );

    ui.add_space(8.0);
    ui.heading("Statistics");
    ui.add_space(8.0);

    // Top summary tiles.
    ui.horizontal_wrapped(|ui| {
        tile(ui, "Total cards", s.total_cards.to_string(), t);
        tile(
            ui,
            "Due today",
            s.forecast.first().map(|d| d.due).unwrap_or(0).to_string(),
            t,
        );
        tile(ui, "New", s.new_cards.to_string(), t);
        tile(ui, "Learning", s.learning_cards.to_string(), t);
        tile(ui, "Review", s.review_cards.to_string(), t);
        tile(ui, "Mature", s.mature_cards.to_string(), t);
        tile(ui, "Streak", format!("{} day(s)", s.streak_days), t);
        tile(
            ui,
            "Retention (30d)",
            s.retention_30d
                .map(|r| format!("{:.0}%", r * 100.0))
                .unwrap_or_else(|| "—".to_string()),
            t,
        );
        tile(ui, "Reviews today", s.reviews_today.to_string(), t);
    });

    ui.add_space(16.0);
    ui.strong("Review activity (last 5 weeks)");
    ui.add_space(4.0);
    heatmap(ui, &s.activity, t);

    ui.add_space(16.0);
    ui.strong("Due forecast (next 14 days)");
    ui.add_space(4.0);
    forecast(ui, &s.forecast, t);
}

fn tile(ui: &mut Ui, label: &str, value: String, t: Theme) {
    ui.vertical(|ui| {
        ui.label(RichText::new(value).size(20.0).strong());
        ui.label(RichText::new(label).weak().color(theme::muted(t)));
    });
    ui.add_space(12.0);
}

fn heat_color(reviews: u32, t: Theme) -> Color32 {
    let bg = match t {
        Theme::Light => Color32::from_rgb(235, 235, 242),
        Theme::Dark => Color32::from_rgb(48, 46, 64),
    };
    if reviews == 0 {
        return bg;
    }
    let intensity = (reviews as f32 / 10.0).clamp(0.25, 1.0);
    let base = theme::ACCENT;
    let mix = |a: u8, b: u8| (f32::from(a) + (f32::from(b) - f32::from(a)) * intensity) as u8;
    Color32::from_rgb(
        mix(bg.r(), base.r()),
        mix(bg.g(), base.g()),
        mix(bg.b(), base.b()),
    )
}

/// 5-week heatmap. Weeks are columns; days run Monday → Sunday top to
/// bottom, aligned to the real weekday of each date.
fn heatmap(ui: &mut Ui, activity: &[stats::DailyCount], t: Theme) {
    let cell = 20.0;
    let gap = 4.0;
    let rows = 7usize;
    // Pad the front so the first day lands on its weekday row.
    let leading_blanks = activity
        .first()
        .map(|d| (d.date.weekday().num_days_from_monday()) as usize)
        .unwrap_or(0);
    let slots = leading_blanks + activity.len();
    let weeks = slots.div_ceil(rows).max(1);

    // Weekday labels.
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.add_space(12.0);
            for wd in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
                ui.set_row_height(cell);
                ui.label(RichText::new(wd).weak().size(9.0));
            }
        });
        ui.add_space(2.0);

        let width = weeks as f32 * (cell + gap);
        let height = rows as f32 * (cell + gap);
        let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let painter = ui.painter_at(rect);

        let mut tooltip: Option<(NaiveDate, u32)> = None;
        for (i, day) in activity.iter().enumerate() {
            let slot = leading_blanks + i;
            let col = slot / rows;
            let row = slot % rows;
            let r = egui::Rect::from_min_size(
                rect.min + Vec2::new(col as f32 * (cell + gap), row as f32 * (cell + gap)),
                Vec2::splat(cell),
            );
            painter.rect_filled(r, 4.0, heat_color(day.reviews, t));
            if r.contains(resp.hover_pos()) {
                tooltip = Some((day.date, day.reviews));
            }
        }
        match tooltip {
            Some((date, reviews)) => resp.on_hover_text(format!(
                "{} · {} review(s)",
                date.format("%d %b %Y"),
                reviews
            )),
            None => resp.on_hover_text("Reviews per day"),
        };
    });
}

fn forecast(ui: &mut Ui, days: &[stats::DailyDue], _t: Theme) {
    let max = days.iter().map(|d| d.due).max().unwrap_or(0).max(1);
    for day in days {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(day.date.format("%a %d %b").to_string())
                    .weak()
                    .size(11.0),
            );
            let w = 8.0 + 180.0 * (day.due as f32 / max as f32);
            let (rect, _) = ui.allocate_exact_size(Vec2::new(w, 14.0), Sense::hover());
            ui.painter().rect_filled(rect, 3.0, theme::ACCENT);
            ui.label(day.due.to_string());
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heat_color_distinguishes_empty_and_busy_days() {
        let empty = heat_color(0, Theme::Dark);
        let busy = heat_color(20, Theme::Dark);
        assert_ne!(empty, busy);
        assert_eq!(heat_color(0, Theme::Light), heat_color(0, Theme::Light));
    }
}
