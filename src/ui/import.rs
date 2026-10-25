//! Import dialog state and helpers.

use std::path::PathBuf;

use uuid::Uuid;

use crate::data::{
    self,
    ImportMapping,
    ImportReport,
    import::detect_separator,
};
use crate::data::export::delimited_records;

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
        }
    }

    pub fn pick_file(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Text / CSV / TSV", &["txt", "csv", "tsv"])
            .pick_file()
        else {
            return;
        };
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                self.separator = detect_separator(&text);
                self.records = delimited_records(&text, self.separator);
                self.text = text;
                self.path = Some(path);
                self.report = None;
                self.result = None;
                self.error = None;
                let cols = self.column_count();
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

    #[must_use]
    pub fn column_count(&self) -> usize {
        self.records.iter().map(Vec::len).max().unwrap_or(0)
    }

    #[must_use]
    pub fn header_detected(&self) -> bool {
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

    #[must_use]
    pub fn cloze_hint(&self) -> bool {
        self.records
            .iter()
            .any(|r| r.get(self.front).map_or(false, |s| s.contains("{{c")))
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