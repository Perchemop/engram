//! UI theme and styling helpers (egui side of `domain::Theme`).
//!
//! Engram's shell uses an original palette: a violet accent over warm dark
//! surfaces (dark) or paper-white surfaces (light).

use egui::{Color32, Stroke};

use crate::domain::{DeckColor, Grade, Theme};

/// Brand accent (Engram violet).
pub const ACCENT: Color32 = Color32::from_rgb(124, 92, 255);
/// Accent pressed/hover variant.
pub const ACCENT_STRONG: Color32 = Color32::from_rgb(100, 70, 230);
/// Warm dark surface.
const INK_900: Color32 = Color32::from_rgb(24, 22, 34);
const INK_700: Color32 = Color32::from_rgb(34, 32, 47);
const INK_600: Color32 = Color32::from_rgb(44, 42, 60);

/// Muted secondary text colour for the current theme.
#[must_use]
pub fn muted(theme: Theme) -> Color32 {
    match theme {
        Theme::Light => Color32::from_rgb(110, 112, 125),
        Theme::Dark => Color32::from_rgb(158, 160, 178),
    }
}

/// Correct / positive colour.
#[must_use]
pub fn ok(theme: Theme) -> Color32 {
    match theme {
        Theme::Light => Color32::from_rgb(46, 145, 80),
        Theme::Dark => Color32::from_rgb(96, 205, 128),
    }
}

/// Warning colour (lapses, overdue).
#[must_use]
pub fn warn(theme: Theme) -> Color32 {
    match theme {
        Theme::Light => Color32::from_rgb(196, 116, 38),
        Theme::Dark => Color32::from_rgb(240, 180, 41),
    }
}

/// Danger colour.
#[must_use]
pub fn danger(theme: Theme) -> Color32 {
    match theme {
        Theme::Light => Color32::from_rgb(196, 70, 70),
        Theme::Dark => Color32::from_rgb(235, 100, 100),
    }
}

/// Colour for a grade, matching the study-screen buttons and the review
/// history rows.
#[must_use]
pub fn grade_color(grade: Grade, theme: Theme) -> Color32 {
    match grade {
        Grade::Again => danger(theme),
        Grade::Hard => warn(theme),
        Grade::Good => ok(theme),
        Grade::Easy => ACCENT,
    }
}

/// Map a [`DeckColor`] token to an egui colour.
#[must_use]
pub fn deck_color(c: DeckColor) -> Color32 {
    let (r, g, b) = c.rgb();
    Color32::from_rgb(r, g, b)
}

/// Apply the theme to the egui context.
pub fn apply_theme(ctx: &egui::Context, theme: Theme) {
    let mut visuals = match theme {
        Theme::Light => egui::Visuals::light(),
        Theme::Dark => egui::Visuals::dark(),
    };

    match theme {
        Theme::Light => {
            visuals.panel_fill = Color32::from_rgb(248, 248, 251);
            visuals.window_fill = Color32::from_rgb(252, 252, 254);
            visuals.extreme_bg_color = Color32::from_rgb(238, 238, 245);
            visuals.faint_bg_color = Color32::from_rgb(243, 243, 248);
        }
        Theme::Dark => {
            visuals.panel_fill = INK_700;
            visuals.window_fill = INK_700;
            visuals.extreme_bg_color = INK_900;
            visuals.faint_bg_color = INK_600;
            visuals.hyperlink_color = ACCENT;
        }
    }

    visuals.selection.bg_fill = ACCENT;
    visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    visuals.widgets.hovered.bg_fill = match theme {
        Theme::Light => Color32::from_rgb(230, 228, 242),
        Theme::Dark => INK_600,
    };
    visuals.widgets.active.bg_fill = match theme {
        Theme::Light => Color32::from_rgb(216, 214, 234),
        Theme::Dark => Color32::from_rgb(56, 53, 78),
    };
    visuals.widgets.open.bg_fill = visuals.widgets.active.bg_fill;

    let egui_theme = match theme {
        Theme::Light => egui::Theme::Light,
        Theme::Dark => egui::Theme::Dark,
    };
    ctx.set_theme(egui_theme);
    ctx.set_visuals_of(egui_theme, visuals);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_theme_selects_matching_theme() {
        let ctx = egui::Context::default();
        apply_theme(&ctx, Theme::Dark);
        assert_eq!(ctx.theme(), egui::Theme::Dark);
        assert!(ctx.style_of(egui::Theme::Dark).visuals.dark_mode);

        apply_theme(&ctx, Theme::Light);
        assert_eq!(ctx.theme(), egui::Theme::Light);
        assert!(!ctx.style_of(egui::Theme::Light).visuals.dark_mode);
    }

    #[test]
    fn deck_colors_map_to_distinct_rgb() {
        let mut seen = std::collections::HashSet::new();
        for c in DeckColor::ALL {
            assert!(seen.insert(deck_color(c)));
        }
    }
}
