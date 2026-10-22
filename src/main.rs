//! Binary entry point: launches the eframe app.

use eframe::{egui, NativeOptions, Result};

use engram::ui::{icon, theme};
use engram::{data, EngramApp};

fn main() -> Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .compact()
        .init();

    let prefs = data::load_prefs();
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Engram — spaced repetition & active recall")
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([880.0, 600.0])
            .with_icon(icon::icon_data()),
        ..Default::default()
    };

    eframe::run_native(
        "Engram",
        native_options,
        Box::new(|cc| {
            theme::apply_theme(&cc.egui_ctx, prefs.theme);
            Ok(Box::new(EngramApp::new(cc)))
        }),
    )
}
