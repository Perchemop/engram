//! The main egui application: shell, routing, menus, dialogs, keyboard.

use std::path::PathBuf;

use chrono::Utc;
use eframe::{App, Frame};
use egui::{Align, Layout, RichText, Ui};
use uuid::Uuid;

use crate::crash_report;
use crate::data::{self, Prefs};
use crate::domain::{CardFilter, CardId, DeckColor, Grade, Library, Scope, StudySession, Theme};
use crate::ui::{decks, editor, generate, stats, study, theme};
use crate::update_check;

/// Top-level pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Decks,
    Study,
    Stats,
    Generate,
}

impl Page {
    /// Tab label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Decks => "Decks",
            Self::Study => "Study",
            Self::Stats => "Stats",
            Self::Generate => "Generate",
        }
    }

    /// All pages in tab order.
    pub const ALL: [Page; 4] = [Page::Decks, Page::Study, Page::Stats, Page::Generate];
}

/// The Engram application.
pub struct EngramApp {
    pub(crate) library: Library,
    pub(crate) path: Option<PathBuf>,
    pub(crate) prefs: Prefs,
    pub(crate) page: Page,
    pub(crate) status: String,
    pub(crate) dirty: bool,
    title_dirty: bool,

    // Decks page state.
    pub(crate) selected_deck: Option<Uuid>,
    pub(crate) filter: CardFilter,
    pub(crate) selected_cards: Vec<CardId>,
    pub(crate) confirm_delete_deck: Option<Uuid>,

    // Dialogs.
    pub(crate) card_editor: Option<editor::CardEditor>,
    pub(crate) deck_dialog: Option<editor::DeckDialog>,
    pub(crate) settings_dialog: Option<editor::SettingsDialog>,

    // Study state.
    pub(crate) session: Option<StudySession>,
    pub(crate) revealed: bool,
    pub(crate) study_scope: Scope,

    // Autosave health: a persistent write failure must not retry every
    // frame (disk full / read-only dir); retries back off to every 5s.
    save_failed: bool,
    last_save_attempt: Option<std::time::Instant>,

    // Crash reporting: a report from a previous run, awaiting user consent
    // (guide §7.1 — nothing is sent without an explicit click).
    pending_crash: Option<crash_report::CrashReport>,
    // Update check: background check result channel + last known status.
    update_rx: Option<std::sync::mpsc::Receiver<update_check::UpdateStatus>>,
    update_status: Option<update_check::UpdateStatus>,

    // Generate page state.
    pub(crate) gen: generate::GenerateState,
}

impl EngramApp {
    /// Build the app: load prefs and the default library (or start fresh).
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let prefs = data::load_prefs();
        theme::apply_theme(&cc.egui_ctx, prefs.theme);

        // Automatic 24-hour update check (guide §7.2): background thread,
        // result polled on the UI thread, fails soft offline.
        let mut prefs = prefs;
        let mut update_rx = None;
        if update_check::should_auto_check(prefs.last_update_check_secs, update_check::now_secs()) {
            prefs.last_update_check_secs = Some(update_check::now_secs());
            let _ = data::save_prefs(&prefs);
            let (tx, rx) = std::sync::mpsc::channel();
            update_rx = Some(rx);
            let ctx = cc.egui_ctx.clone();
            std::thread::spawn(move || {
                let _ = tx.send(update_check::check_now());
                ctx.request_repaint();
            });
        }

        let (library, path, status) = match data::default_library_path() {
            Some(p) => match data::load_library(&p) {
                Ok(Some(lib)) => (lib, Some(p), "Library loaded".to_string()),
                Ok(None) => (
                    Library::new("My Engram", Utc::now()),
                    Some(p),
                    "New library created".to_string(),
                ),
                Err(e) => (
                    Library::new("My Engram", Utc::now()),
                    None,
                    format!("Library failed to load ({e}); started a new one"),
                ),
            },
            None => (
                Library::new("My Engram", Utc::now()),
                None,
                "No data directory available; library is not persisted yet".to_string(),
            ),
        };

        Self {
            library,
            path,
            prefs,
            page: Page::Decks,
            status,
            dirty: false,
            title_dirty: true,
            selected_deck: None,
            filter: CardFilter::default(),
            selected_cards: Vec::new(),
            confirm_delete_deck: None,
            card_editor: None,
            deck_dialog: None,
            settings_dialog: None,
            session: None,
            revealed: false,
            study_scope: Scope::All,
            save_failed: false,
            last_save_attempt: None,
            pending_crash: crash_report::pending_report(),
            update_rx,
            update_status: None,
            gen: generate::GenerateState::default(),
        }
    }

    // ------------------------------------------------------------------
    // Persistence
    // ------------------------------------------------------------------

    fn resolve_save_path(&self) -> Option<PathBuf> {
        self.path.clone().or_else(data::default_library_path)
    }

    fn save_now(&mut self) {
        let Some(path) = self.resolve_save_path() else {
            self.status = "No save location available".to_string();
            return;
        };
        self.last_save_attempt = Some(std::time::Instant::now());
        match data::save_library(&self.library, &path) {
            Ok(()) => {
                self.path = Some(path);
                self.dirty = false;
                self.save_failed = false;
                self.status = "Saved".to_string();
                self.title_dirty = true;
            }
            Err(e) => {
                self.save_failed = true;
                self.status = format!("Save failed: {e}");
            }
        }
    }

    /// Auto-save without touching the status bar (errors still surface).
    /// A failed attempt sets `save_failed` so the caller can back off
    /// instead of retrying a doomed write every frame.
    fn save_quiet(&mut self) {
        let Some(path) = self.resolve_save_path() else {
            return;
        };
        self.last_save_attempt = Some(std::time::Instant::now());
        match data::save_library(&self.library, &path) {
            Ok(()) => {
                self.path = Some(path);
                self.dirty = false;
                self.save_failed = false;
                self.title_dirty = true;
            }
            Err(e) => {
                self.save_failed = true;
                self.status = format!("Auto-save failed: {e}");
            }
        }
    }

    /// True when the autosave loop should attempt another write now.
    #[must_use]
    fn autosave_due(&self) -> bool {
        if !self.dirty {
            return false;
        }
        if !self.save_failed {
            return true;
        }
        // Back off: retry a failed save at most every 5 seconds.
        self.last_save_attempt
            .map(|t| t.elapsed() >= std::time::Duration::from_secs(5))
            .unwrap_or(true)
    }

    fn save_as_action(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Engram library", &["engram.json"])
            .set_file_name("library.engram.json")
            .save_file()
        {
            let path = PathBuf::from(data::normalise_filename(&path.to_string_lossy()));
            match data::save_library(&self.library, &path) {
                Ok(()) => {
                    self.path = Some(path);
                    self.dirty = false;
                    self.status = "Saved".to_string();
                    self.title_dirty = true;
                }
                Err(e) => self.status = format!("Save failed: {e}"),
            }
        }
    }

    fn open_library_action(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Engram library", &["engram.json", "json"])
            .pick_file()
        {
            match data::load_library(&path) {
                Ok(Some(lib)) => {
                    self.library = lib;
                    self.path = Some(path);
                    self.close_all_dialogs();
                    self.session = None;
                    self.selected_deck = None;
                    self.selected_cards.clear();
                    self.filter = CardFilter::default();
                    self.status = "Library opened".to_string();
                    self.title_dirty = true;
                }
                Ok(None) => self.status = "File not found".to_string(),
                Err(e) => self.status = format!("Open failed: {e}"),
            }
        }
    }

    fn new_library_action(&mut self) {
        self.library = Library::new("My Engram", Utc::now());
        self.path = None;
        self.close_all_dialogs();
        self.session = None;
        self.selected_deck = None;
        self.selected_cards.clear();
        self.filter = CardFilter::default();
        self.gen = generate::GenerateState::default();
        self.status = "New library created (unsaved)".to_string();
        self.dirty = true;
        self.title_dirty = true;
    }

    fn export_csv_action(&mut self) {
        let scope = Scope::from_option(self.selected_deck);
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .set_file_name("engram-export.csv")
            .save_file()
        {
            match data::export_csv(&path, &self.library, scope) {
                Ok(n) => self.status = format!("Exported {n} card(s) to CSV"),
                Err(e) => self.status = format!("Export failed: {e}"),
            }
        }
    }

    fn export_json_action(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON", &["json"])
            .set_file_name("engram-backup.json")
            .save_file()
        {
            match data::export_json(&path, &self.library) {
                Ok(()) => self.status = format!("Backup written to {}", path.display()),
                Err(e) => self.status = format!("Backup failed: {e}"),
            }
        }
    }

    fn import_csv_action(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("CSV", &["csv"])
            .pick_file()
        else {
            return;
        };
        let deck = self
            .selected_deck
            .or_else(|| self.library.decks.first().map(|d| d.id));
        let Some(deck) = deck else {
            self.status = "Create a deck before importing".to_string();
            return;
        };
        match data::import_csv(&path, &mut self.library, deck, Utc::now()) {
            Ok(n) => {
                self.status = format!("Imported {n} card(s)");
                self.dirty = true;
            }
            Err(e) => self.status = format!("Import failed: {e}"),
        }
    }

    // ------------------------------------------------------------------
    // Study
    // ------------------------------------------------------------------

    pub(crate) fn start_study(&mut self, scope: Scope) {
        self.study_scope = scope;
        match self.library.build_session(scope, Utc::now()) {
            Ok(session) => {
                self.session = Some(session);
                self.revealed = false;
                self.page = Page::Study;
                self.status = "Session started".to_string();
            }
            Err(e) => self.status = format!("Nothing to study: {e}"),
        }
    }

    pub(crate) fn grade_current(&mut self, grade: Grade) {
        let now = Utc::now();
        let Some((card_id, queue_kind)) = self.session.as_ref().and_then(|s| s.current()) else {
            return;
        };
        match self.library.record_review(card_id, grade, now, queue_kind) {
            Ok(_) => {
                if let Some(session) = self.session.as_mut() {
                    session.answer(grade, now);
                }
                self.revealed = false;
                self.dirty = true;
                if self.session.as_ref().is_some_and(|s| s.finished) {
                    if let Some(s) = self.session.as_ref().map(StudySession::summary) {
                        self.status = format!(
                            "Session complete: {} card(s), {:.0}% recalled",
                            s.total,
                            s.retention() * 100.0
                        );
                    }
                }
            }
            Err(e) => self.status = format!("Review failed: {e}"),
        }
    }

    pub(crate) fn end_session(&mut self) {
        if let Some(s) = self.session.as_ref().map(StudySession::summary) {
            self.status = format!("Session ended early: {} card(s) answered", s.total);
        }
        self.session = None;
        self.revealed = false;
    }

    /// Close every open dialog. Used when a library is opened/created so
    /// editors can never reference decks from the previous library, and by
    /// the Esc key.
    fn close_all_dialogs(&mut self) {
        self.card_editor = None;
        self.deck_dialog = None;
        self.settings_dialog = None;
        self.confirm_delete_deck = None;
    }

    // ------------------------------------------------------------------
    // Update check (guide §7.2)
    // ------------------------------------------------------------------

    /// Spawn a background update check; the result arrives on a channel the
    /// UI polls, so the network never blocks a frame. Fails soft.
    fn spawn_update_check(&mut self, ctx: &egui::Context, automatic: bool) {
        if self.update_rx.is_some() {
            return; // a check is already in flight
        }
        let (tx, rx) = std::sync::mpsc::channel();
        self.update_rx = Some(rx);
        if automatic {
            self.prefs.last_update_check_secs = Some(update_check::now_secs());
            let _ = data::save_prefs(&self.prefs);
            self.status = "Checking for updates…".to_string();
        }
        // Clone the context so the thread owns an 'static handle.
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let status = update_check::check_now();
            // The receiver may be gone (app closed) — that is fine.
            let _ = tx.send(status);
            ctx.request_repaint();
        });
    }

    fn poll_update_check(&mut self) {
        if let Some(rx) = &self.update_rx {
            if let Ok(status) = rx.try_recv() {
                self.update_status = Some(status);
                self.update_rx = None;
                match &self.update_status {
                    Some(update_check::UpdateStatus::Available { latest, .. }) => {
                        self.status = format!("Update available: {latest}");
                    }
                    Some(update_check::UpdateStatus::UpToDate) => {
                        self.status =
                            format!("You are up to date (v{})", update_check::current_version());
                    }
                    Some(update_check::UpdateStatus::Unknown(reason)) => {
                        self.status = format!("Update check unavailable: {reason}");
                    }
                    None => {}
                }
            }
        }
    }

    // ------------------------------------------------------------------
    // Theme
    // ------------------------------------------------------------------

    fn set_theme(&mut self, ctx: &egui::Context, t: Theme) {
        self.prefs.theme = t;
        theme::apply_theme(ctx, t);
        match data::save_prefs(&self.prefs) {
            Ok(()) => self.status = format!("Theme: {}", t.label()),
            Err(e) => self.status = format!("Theme changed, but saving failed: {e}"),
        }
    }

    // ------------------------------------------------------------------
    // Dialog commits
    // ------------------------------------------------------------------

    fn save_card_editor(&mut self) {
        let Some(ed) = self.card_editor.clone() else {
            return;
        };
        let now = Utc::now();
        let kind = ed.build_kind();
        let tags = ed.parse_tags();
        let result = match ed.card_id {
            Some(id) => self
                .library
                .set_card_kind(id, kind, now)
                .and_then(|_| self.library.set_card_tags(id, tags, now))
                .and_then(|_| {
                    let moves = self.library.card(id).map(|c| c.deck_id != ed.deck_id);
                    match moves {
                        Some(true) => self.library.move_card(id, ed.deck_id),
                        _ => Ok(()),
                    }
                }),
            None => self
                .library
                .add_card(ed.deck_id, kind, tags, now)
                .map(|_: Uuid| ()),
        };
        match result {
            Ok(()) => {
                self.card_editor = None;
                self.dirty = true;
                self.status = if ed.card_id.is_some() {
                    "Card updated".to_string()
                } else {
                    "Card added".to_string()
                };
            }
            Err(e) => {
                if let Some(dialog) = self.card_editor.as_mut() {
                    dialog.error = Some(e.to_string());
                }
            }
        }
    }

    fn save_deck_dialog(&mut self) {
        let Some(dialog) = self.deck_dialog.clone() else {
            return;
        };
        let now = Utc::now();
        let result = match dialog.deck_id {
            Some(id) => self
                .library
                .rename_deck(id, &dialog.name)
                .and_then(|_| self.library.set_deck_color(id, dialog.color))
                .map(|_| Some(id)),
            None => self
                .library
                .new_deck(dialog.parent, &dialog.name, dialog.color, now)
                .map(Some),
        };
        match result {
            Ok(maybe_id) => {
                self.deck_dialog = None;
                self.dirty = true;
                if dialog.deck_id.is_none() {
                    self.selected_deck = maybe_id;
                }
                self.status = if dialog.deck_id.is_some() {
                    "Deck updated".to_string()
                } else {
                    "Deck created".to_string()
                };
            }
            Err(e) => {
                if let Some(d) = self.deck_dialog.as_mut() {
                    d.error = Some(e.to_string());
                }
            }
        }
    }

    fn save_settings_dialog(&mut self) {
        let Some(mut dialog) = self.settings_dialog.clone() else {
            return;
        };
        dialog.sync_steps();
        if !editor::settings_valid(&dialog.settings.sched) {
            dialog.error = Some("Invalid scheduler parameter".to_string());
            self.settings_dialog = Some(dialog);
            return;
        }
        dialog.settings.sched = dialog.settings.sched.sanitised();
        self.library.settings = dialog.settings;
        self.settings_dialog = None;
        self.dirty = true;
        self.status = "Settings applied".to_string();
    }

    // ------------------------------------------------------------------
    // Chrome
    // ------------------------------------------------------------------

    fn menu_bar(&mut self, ui: &mut Ui) {
        use egui::MenuBar;
        MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Library").clicked() {
                    self.new_library_action();
                    ui.close();
                }
                if ui.button("Open Library…").clicked() {
                    self.open_library_action();
                    ui.close();
                }
                if ui.button("Save").clicked() {
                    self.save_now();
                    ui.close();
                }
                if ui.button("Save As…").clicked() {
                    self.save_as_action();
                    ui.close();
                }
                ui.separator();
                if ui.button("Import CSV…").clicked() {
                    self.import_csv_action();
                    ui.close();
                }
                if ui.button("Export CSV…").clicked() {
                    self.export_csv_action();
                    ui.close();
                }
                if ui.button("Export JSON Backup…").clicked() {
                    self.export_json_action();
                    ui.close();
                }
                ui.separator();
                if ui.button("Settings…").clicked() {
                    self.settings_dialog =
                        Some(editor::SettingsDialog::from_library(&self.library));
                    ui.close();
                }
                if ui.button("Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.menu_button("View", |ui| {
                for option in Theme::ALL {
                    let mut current = self.prefs.theme;
                    if ui
                        .radio_value(&mut current, option, option.label())
                        .clicked()
                        && current != self.prefs.theme
                    {
                        self.set_theme(ui.ctx(), current);
                    }
                }
            });
            ui.menu_button("Help", |ui| {
                if ui.button("Check for updates").clicked() {
                    self.spawn_update_check(ui.ctx(), false);
                    ui.close();
                }
                if let Some(status) = &self.update_status {
                    ui.separator();
                    match status {
                        update_check::UpdateStatus::UpToDate => {
                            ui.label(format!(
                                "You are up to date (v{})",
                                update_check::current_version()
                            ));
                        }
                        update_check::UpdateStatus::Available { latest, url } => {
                            ui.horizontal(|ui| {
                                ui.label(format!("Update available: {latest}"));
                                if ui.button("Open release page").clicked() {
                                    ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                                }
                            });
                        }
                        update_check::UpdateStatus::Unknown(reason) => {
                            ui.label(
                                RichText::new(format!("Update check unavailable: {reason}")).weak(),
                            );
                        }
                    }
                }
                ui.separator();
                ui.label(format!("Engram v{}", update_check::current_version()));
                ui.label(format!("Running on {}", crash_report::os_description()));
                ui.label("Free, open-source, offline-first.");
                if ui.button("Project page").clicked() {
                    ui.ctx().open_url(egui::OpenUrl::new_tab(format!(
                        "https://github.com/{}/releases",
                        crash_report::GITHUB_REPO
                    )));
                }
                ui.separator();
                ui.label("Keyboard shortcuts:");
                ui.label("Ctrl+S — save");
                ui.label("Space / Enter — show answer");
                ui.label("1 / 2 / 3 / 4 — grade: Again, Hard, Good, Easy");
                ui.label("Esc — end session / close dialog");
            });
        });
    }

    fn top_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            for page in Page::ALL {
                let selected = self.page == page;
                if ui.selectable_label(selected, page.label()).clicked() {
                    self.page = page;
                }
            }
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if self.dirty {
                    ui.label(RichText::new("• unsaved").color(theme::warn(self.prefs.theme)));
                }
                if !self.status.is_empty() {
                    ui.label(RichText::new(&self.status).weak());
                }
            });
        });
    }

    fn bottom_bar(&mut self, ui: &mut Ui) {
        let now = Utc::now();
        let counts = self.library.due_counts(Scope::All, now);
        ui.horizontal(|ui| {
            ui.small(format!(
                "{} due now ({} learning, {} review) · {} new",
                counts.learning + counts.review,
                counts.learning,
                counts.review,
                counts.new_cards
            ));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.small(format!("{} card(s)", self.library.cards.len()));
                ui.small(&self.library.name);
            });
        });
    }

    fn draw_dialogs(&mut self, ui: &mut Ui) {
        // Card editor.
        if self.card_editor.is_some() {
            let mut open = true;
            let title = self
                .card_editor
                .as_ref()
                .map(|ed| editor::window_id("Edit card", ed.card_id))
                .unwrap_or_default();
            egui::Window::new(title)
                .open(&mut open)
                .resizable(false)
                .show(ui.ctx(), |ui| self.card_editor_window(ui));
            if !open {
                self.card_editor = None;
            }
        }

        // Deck dialog.
        if self.deck_dialog.is_some() {
            let mut open = true;
            let title = self
                .deck_dialog
                .as_ref()
                .map(|d| editor::window_id("Edit deck", d.deck_id))
                .unwrap_or_default();
            egui::Window::new(title)
                .open(&mut open)
                .resizable(false)
                .show(ui.ctx(), |ui| self.deck_dialog_window(ui));
            if !open {
                self.deck_dialog = None;
            }
        }

        // Crash-report consent dialog (guide §7.1: preview + cancel).
        if self.pending_crash.is_some() {
            let mut open = true;
            egui::Window::new("Engram closed unexpectedly")
                .open(&mut open)
                .resizable(true)
                .default_width(560.0)
                .show(ui.ctx(), |ui| self.crash_dialog_window(ui));
            if !open {
                self.pending_crash = None;
            }
        }

        // Settings dialog.
        if self.settings_dialog.is_some() {
            let mut open = true;
            egui::Window::new("Settings")
                .open(&mut open)
                .resizable(false)
                .show(ui.ctx(), |ui| self.settings_dialog_window(ui));
            if !open {
                self.settings_dialog = None;
            }
        }
    }

    fn card_editor_window(&mut self, ui: &mut Ui) {
        let mut save = false;
        let mut cancel = false;
        {
            let Some(ed) = self.card_editor.as_mut() else {
                return;
            };
            let t = self.prefs.theme;
            let deck_options: Vec<(String, Uuid)> = self
                .library
                .decks
                .iter()
                .map(|d| (d.name.clone(), d.id))
                .collect();

            egui::Grid::new("card_editor_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Type");
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut ed.is_cloze, false, "Basic");
                        ui.radio_value(&mut ed.is_cloze, true, "Cloze");
                    });
                    ui.end_row();

                    if ed.is_cloze {
                        ui.label("Text");
                        ui.add(
                            egui::TextEdit::multiline(&mut ed.front)
                                .hint_text("The {{c1::hidden part}} stays visible")
                                .desired_rows(3)
                                .desired_width(420.0),
                        );
                        ui.end_row();
                        ui.label("Preview");
                        ui.label(
                            RichText::new(crate::domain::render_cloze_blank(&ed.front))
                                .color(theme::ACCENT),
                        );
                        ui.end_row();
                    } else {
                        ui.label("Front");
                        ui.add(
                            egui::TextEdit::multiline(&mut ed.front)
                                .hint_text("The prompt (question)")
                                .desired_rows(3)
                                .desired_width(420.0),
                        );
                        ui.end_row();
                        ui.label("Back");
                        ui.add(
                            egui::TextEdit::multiline(&mut ed.back)
                                .hint_text("The answer")
                                .desired_rows(3)
                                .desired_width(420.0),
                        );
                        ui.end_row();
                    }

                    ui.label("Tags");
                    ui.add(
                        egui::TextEdit::singleline(&mut ed.tags)
                            .hint_text("comma or space separated")
                            .desired_width(420.0),
                    );
                    ui.end_row();

                    ui.label("Deck");
                    egui::ComboBox::from_id_salt("card_editor_deck")
                        .selected_text(
                            deck_options
                                .iter()
                                .find(|(_, id)| *id == ed.deck_id)
                                .map(|(n, _)| n.clone())
                                .unwrap_or_else(|| "—".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            for (name, id) in &deck_options {
                                ui.selectable_value(&mut ed.deck_id, *id, name.clone());
                            }
                        });
                    ui.end_row();
                });

            if let Some(err) = &ed.error {
                ui.label(RichText::new(err).color(theme::danger(t)));
            }

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                save = ui.button("Save").clicked();
                cancel = ui.button("Cancel").clicked();
            });
        }
        if cancel {
            self.card_editor = None;
        } else if save {
            self.save_card_editor();
        }
    }

    fn deck_dialog_window(&mut self, ui: &mut Ui) {
        let mut save = false;
        let mut cancel = false;
        {
            let Some(dialog) = self.deck_dialog.as_mut() else {
                return;
            };
            let t = self.prefs.theme;

            egui::Grid::new("deck_dialog_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("Name");
                    ui.add(
                        egui::TextEdit::singleline(&mut dialog.name)
                            .hint_text("Deck name")
                            .desired_width(300.0),
                    );
                    ui.end_row();

                    ui.label("Parent");
                    let parents: Vec<(String, Option<Uuid>)> = {
                        let mut v = vec![("Top level".to_string(), None)];
                        for d in &self.library.decks {
                            if dialog.deck_id != Some(d.id) {
                                v.push((d.name.clone(), Some(d.id)));
                            }
                        }
                        v
                    };
                    egui::ComboBox::from_id_salt("deck_dialog_parent")
                        .selected_text(
                            parents
                                .iter()
                                .find(|(_, id)| *id == dialog.parent)
                                .map(|(n, _)| n.clone())
                                .unwrap_or_else(|| "—".to_string()),
                        )
                        .show_ui(ui, |ui| {
                            for (name, id) in &parents {
                                ui.selectable_value(&mut dialog.parent, *id, name.clone());
                            }
                        });
                    ui.end_row();

                    ui.label("Colour");
                    ui.horizontal(|ui| {
                        for color in DeckColor::ALL {
                            let dot = RichText::new("●").color(theme::deck_color(color));
                            let selected = dialog.color == Some(color);
                            let selection_fill = ui.visuals().selection.bg_fill;
                            if ui
                                .add(
                                    egui::Button::new(dot)
                                        .fill(if selected {
                                            selection_fill
                                        } else {
                                            egui::Color32::TRANSPARENT
                                        })
                                        .min_size(egui::vec2(28.0, 24.0)),
                                )
                                .on_hover_text(color.label())
                                .clicked()
                            {
                                dialog.color = if selected { None } else { Some(color) };
                            }
                        }
                    });
                    ui.end_row();
                });

            if let Some(err) = &dialog.error {
                ui.label(RichText::new(err).color(theme::danger(t)));
            }

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                save = ui.button("Save").clicked();
                cancel = ui.button("Cancel").clicked();
            });
        }
        if cancel {
            self.deck_dialog = None;
        } else if save {
            self.save_deck_dialog();
        }
    }

    fn crash_dialog_window(&mut self, ui: &mut Ui) {
        let Some(report) = self.pending_crash.clone() else {
            return;
        };
        ui.label(
            RichText::new(
                "Engram hit an unexpected error in a previous session. The report below was generated locally — nothing has been sent. You can review it, submit it to GitHub, or discard it.",
            )
            .weak(),
        );
        ui.add_space(6.0);
        egui::ScrollArea::vertical()
            .max_height(220.0)
            .show(ui, |ui| {
                ui.label(RichText::new(&report.body).monospace().size(11.0));
            });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Open GitHub issue").clicked() {
                let url = crash_report::build_issue_url(&report);
                ui.ctx().open_url(egui::OpenUrl::new_tab(url));
                self.pending_crash = None;
                crash_report::discard();
                self.status =
                    "Crash report: opening GitHub (sign-in happens in your browser)".to_string();
            }
            if ui.button("Copy report").clicked() {
                ui.ctx().copy_text(report.body.clone());
                self.status = "Crash report copied to clipboard".to_string();
            }
            if ui.button("Discard").clicked() {
                crash_report::discard();
                self.pending_crash = None;
                self.status = "Crash report discarded".to_string();
            }
        });
    }

    fn settings_dialog_window(&mut self, ui: &mut Ui) {
        let mut apply = false;
        let mut cancel = false;
        {
            let Some(dialog) = self.settings_dialog.as_mut() else {
                return;
            };
            let t = self.prefs.theme;

            ui.label(
                RichText::new("Scheduler (SM-2) parameters. Changes apply to future reviews.")
                    .weak(),
            );
            ui.add_space(4.0);
            egui::Grid::new("settings_grid")
                .num_columns(2)
                .spacing([10.0, 6.0])
                .show(ui, |ui| {
                    ui.label("New cards per day");
                    ui.add(
                        egui::DragValue::new(&mut dialog.settings.daily_new_limit).range(0..=500),
                    );
                    ui.end_row();

                    ui.label("Reviews per day");
                    ui.add(
                        egui::DragValue::new(&mut dialog.settings.daily_reviews_limit)
                            .range(10..=2000),
                    );
                    ui.end_row();

                    ui.label("Learning steps (minutes)");
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut dialog.learning_steps)
                                .desired_width(160.0),
                        )
                        .lost_focus()
                    {
                        dialog.sync_steps();
                    }
                    ui.end_row();

                    ui.label("Relearning steps (minutes)");
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut dialog.relearn_steps)
                                .desired_width(160.0),
                        )
                        .lost_focus()
                    {
                        dialog.sync_steps();
                    }
                    ui.end_row();

                    ui.label("Starting ease");
                    ui.add(
                        egui::DragValue::new(&mut dialog.settings.sched.starting_ease)
                            .speed(0.05)
                            .range(1.3..=3.0),
                    );
                    ui.end_row();

                    ui.label("Graduating interval (days)");
                    ui.add(
                        egui::DragValue::new(&mut dialog.settings.sched.graduating_interval_days)
                            .speed(0.2)
                            .range(1.0..=30.0),
                    );
                    ui.end_row();

                    ui.label("Easy interval (days)");
                    ui.add(
                        egui::DragValue::new(&mut dialog.settings.sched.easy_interval_days)
                            .speed(0.2)
                            .range(1.0..=60.0),
                    );
                    ui.end_row();

                    ui.label("Interval modifier");
                    ui.add(
                        egui::DragValue::new(&mut dialog.settings.sched.interval_modifier)
                            .speed(0.01)
                            .range(0.5..=2.0),
                    );
                    ui.end_row();

                    ui.label("Easy bonus");
                    ui.add(
                        egui::DragValue::new(&mut dialog.settings.sched.easy_bonus)
                            .speed(0.01)
                            .range(1.0..=2.0),
                    );
                    ui.end_row();

                    ui.label("Hard factor");
                    ui.add(
                        egui::DragValue::new(&mut dialog.settings.sched.hard_factor)
                            .speed(0.01)
                            .range(1.0..=1.5),
                    );
                    ui.end_row();

                    ui.label("Max interval (days)");
                    ui.add(
                        egui::DragValue::new(&mut dialog.settings.sched.max_interval_days)
                            .speed(10.0)
                            .range(30.0..=50_000.0),
                    );
                    ui.end_row();

                    ui.label("Mature threshold (days)");
                    ui.add(
                        egui::DragValue::new(&mut dialog.settings.sched.mature_threshold_days)
                            .speed(0.5)
                            .range(7.0..=90.0),
                    );
                    ui.end_row();
                });

            if let Some(err) = &dialog.error {
                ui.label(RichText::new(err).color(theme::danger(t)));
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                apply = ui.button("Apply").clicked();
                cancel = ui.button("Cancel").clicked();
            });
        }
        if cancel {
            self.settings_dialog = None;
        } else if apply {
            self.save_settings_dialog();
        }
    }
}

impl App for EngramApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        ctx.input(|i| {
            if i.modifiers.ctrl && i.key_pressed(egui::Key::S) {
                self.save_now();
            }
            let dialog_open = self.card_editor.is_some()
                || self.deck_dialog.is_some()
                || self.settings_dialog.is_some();
            if i.key_pressed(egui::Key::Escape) {
                if dialog_open {
                    self.close_all_dialogs();
                } else if self.page == Page::Study && self.session.is_some() {
                    self.end_session();
                }
            }
            if self.page == Page::Study && self.session.is_some() && !dialog_open {
                if !self.revealed
                    && (i.key_pressed(egui::Key::Space) || i.key_pressed(egui::Key::Enter))
                {
                    self.revealed = true;
                } else if self.revealed {
                    let grades = [
                        (egui::Key::Num1, Grade::Again),
                        (egui::Key::Num2, Grade::Hard),
                        (egui::Key::Num3, Grade::Good),
                        (egui::Key::Num4, Grade::Easy),
                    ];
                    for (key, grade) in grades {
                        if i.key_pressed(key) {
                            self.grade_current(grade);
                        }
                    }
                }
            }
        });
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut Frame) {
        self.poll_update_check();
        if self.title_dirty {
            let title = match &self.path {
                Some(p) => format!(
                    "{} — Engram",
                    p.file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| p.to_string_lossy().to_string())
                ),
                None => "Untitled — Engram".to_string(),
            };
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Title(title));
            self.title_dirty = false;
        }

        egui::Panel::top("menu").show(ui, |ui| self.menu_bar(ui));
        egui::Panel::top("topbar").show(ui, |ui| self.top_bar(ui));
        if self.page == Page::Decks {
            egui::Panel::left("deck_tree")
                .default_size(230.0)
                .min_size(180.0)
                .show(ui, |ui| decks::show_tree(ui, self));
        }
        egui::Panel::bottom("bottombar").show(ui, |ui| self.bottom_bar(ui));

        egui::CentralPanel::default().show(ui, |ui| match self.page {
            Page::Decks => decks::show_cards(ui, self),
            Page::Study => study::show(ui, self),
            Page::Stats => stats::show(ui, self),
            Page::Generate => generate::show(ui, self),
        });

        self.draw_dialogs(ui);

        // Persist promptly: user data must never sit unsaved for long —
        // but a failing save backs off instead of retrying every frame.
        if self.autosave_due() {
            self.save_quiet();
        }

        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f32(0.5));
    }
}
