//! Import dialog state and helpers.

use std::path::PathBuf;

use uuid::Uuid;

use crate::data::export::delimited_records;
use crate::data::{self, import::detect_separator, ImportMapping, ImportReport};

#[derive(Debug, Clone)]
pub struct ImportDialog {
    pub path: Option<PathBuf>,
    pub text: String,
    pub separator: char,
    pub records: Vec<Vec<String>>,
    pub default_deck: Uuid,
    pub deck_id: Uuid,
    pub front: usize,
    pub back: usize,
    pub tags: Option<usize>,
    pub deck: Option<usize>,
    pub report: Option<ImportReport>,
    pub result: Option<String>,
    pub error: Option<String>,
    /// Per-file aggregates cached at pick time so the 2 Hz immediate-mode
    /// repaint never rescans every record (audit finding
    /// import-dialog-unbounded-ui-thread).
    pub cached_columns: usize,
    pub cached_header: bool,
    pub cached_cloze: bool,
}

impl ImportDialog {
    #[must_use]
    pub fn new(default_deck: Uuid) -> Self {
        Self {
            path: None,
            text: String::new(),
            separator: ',',
            records: Vec::new(),
            default_deck,
            deck_id: default_deck,
            front: 0,
            back: 1,
            tags: Some(2),
            deck: None,
            report: None,
            result: None,
            error: None,
            cached_columns: 0,
            cached_header: false,
            cached_cloze: false,
        }
    }

    pub fn pick_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Text / CSV / TSV", &["txt", "csv", "tsv"])
            .pick_file()
        else {
            return;
        };
        // Size cap (audit finding import-dialog-unbounded-ui-thread): the
        // read and parse run on the UI thread, so refuse absurdly large
        // files with a clear message instead of hanging the app.
        const MAX_IMPORT_FILE_BYTES: u64 = 10 * 1024 * 1024;
        match std::fs::metadata(&path) {
            Ok(meta) if meta.len() > MAX_IMPORT_FILE_BYTES => {
                self.error = Some(format!(
                    "File is {} MB; the import limit is 10 MB",
                    meta.len() / (1024 * 1024)
                ));
                return;
            }
            Err(e) => {
                self.error = Some(format!("Could not read file: {e}"));
                return;
            }
            Ok(_) => {}
        }
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                self.separator = detect_separator(&text);
                self.records = delimited_records(&text, self.separator);
                self.text = text;
                self.path = Some(path);
                self.report = None;
                self.result = None;
                self.error = None;
                // Cache the per-file aggregates once: the immediate-mode UI
                // repaints at 2 Hz and these used to rescan every record
                // every frame.
                self.cached_columns = self.compute_columns();
                self.cached_header = self.compute_header();
                self.cached_cloze = self.compute_cloze();
                let cols = self.cached_columns;
                if cols == 0 {
                    self.front = 0;
                    self.back = 0;
                    self.tags = None;
                } else {
                    self.front = 0;
                    self.back = if cols > 1 { 1 } else { 0 };
                    self.tags = if cols >= 3 { Some(2) } else { None };
                    self.deck = if cols >= 4 { Some(3) } else { None };
                }
            }
            Err(e) => {
                self.error = Some(format!("Could not read file: {e}"));
            }
        }
    }

    /// Columns in the widest record (cached at pick time).
    #[must_use]
    pub fn column_count(&self) -> usize {
        self.cached_columns
    }

    fn compute_columns(&self) -> usize {
        self.records.iter().map(Vec::len).max().unwrap_or(0)
    }

    #[must_use]
    pub fn header_detected(&self) -> bool {
        self.cached_header
    }

    fn compute_header(&self) -> bool {
        let Some(row) = self
            .records
            .iter()
            .find(|r| !r.iter().all(|f| f.trim().is_empty()))
        else {
            return false;
        };
        let first = row.first().map_or("", String::as_str).trim().to_lowercase();
        first.starts_with("front") || first.starts_with("question")
    }

    #[must_use]
    pub fn preview_rows(&self) -> Vec<&[String]> {
        self.records
            .iter()
            .filter(|r| !r.iter().all(|f| f.trim().is_empty()))
            .take(10)
            .map(Vec::as_slice)
            .collect()
    }

    fn compute_cloze(&self) -> bool {
        self.records.iter().any(|r| {
            r.get(self.front)
                .map(|s| s.contains("{{c"))
                .unwrap_or(false)
        })
    }

    #[must_use]
    pub fn cloze_hint(&self) -> bool {
        self.cached_cloze
    }

    pub fn run_import(
        &self,
        lib: &mut crate::domain::library::Library,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<ImportReport, crate::data::DataError> {
        let mapping = ImportMapping {
            front: self.front,
            back: self.back,
            tags: self.tags,
            deck: self.deck,
        };
        data::import_text_from_str(&self.text, lib, self.deck_id, &mapping, now)
    }
}
