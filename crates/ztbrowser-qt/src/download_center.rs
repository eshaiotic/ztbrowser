//! The downloads of one profile: where each file goes, how far it has got, and what the list shows.
//!
//! The decisions come from the core (`DownloadSettings`, `DownloadHistory`, `DownloadFileName`); this type keeps
//! them in `downloads.json` and `download-settings.json`, the files the GTK edition uses, so a person's
//! history and choices carry over.

use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::download::download_file_name::DownloadFileName;
use ztbrowser_core::download::download_history::DownloadHistory;
use ztbrowser_core::download::download_history::DownloadId;
use ztbrowser_core::download::download_history::DownloadState;
use ztbrowser_core::download::download_settings::DownloadSettings;
use ztbrowser_core::storage::owned_json_file::OwnedJsonFile;

pub struct DownloadCenter {
    history: DownloadHistory,
    settings: DownloadSettings,
    history_file: OwnedJsonFile,
    settings_file: OwnedJsonFile,
    /// The system's Downloads folder, used until the person chooses another.
    default_folder: PathBuf,
}

impl DownloadCenter {
    /// How many rows the toolbar list shows.
    pub const LIST_LENGTH: usize = 5;

    /// Reads the profile's stored history and settings from `directory`.
    #[must_use]
    pub fn open(directory: &Path, default_folder: PathBuf) -> Self {
        let history_file = OwnedJsonFile::new(directory.join("downloads.json"));
        let settings_file = OwnedJsonFile::new(directory.join("download-settings.json"));
        Self {
            history: history_file.read().map(|document| DownloadHistory::from_json(&document)).unwrap_or_default(),
            settings: settings_file.read().map(|document| DownloadSettings::from_json(&document)).unwrap_or_default(),
            history_file,
            settings_file,
            default_folder,
        }
    }

    #[must_use]
    pub fn asks_where_to_save(&self) -> bool {
        self.settings.asks_where_to_save()
    }

    pub fn set_asks_where_to_save(&mut self, asks: bool) {
        self.settings = self.settings.clone().with_asking(asks);
        self.save_settings();
    }

    #[must_use]
    pub fn folder(&self) -> &Path {
        self.settings.folder_or(&self.default_folder)
    }

    pub fn set_folder(&mut self, folder: PathBuf) {
        self.settings = self.settings.clone().with_folder(folder);
        self.save_settings();
    }

    /// The path a file the site calls `suggested_name` would be saved at: in the download folder, under a
    /// cleaned-up name that no existing file has.
    #[must_use]
    pub fn proposed_path(&self, suggested_name: &str) -> PathBuf {
        DownloadFileName::from_suggestion(suggested_name).first_free_path_in(self.folder(), Path::exists)
    }

    /// Lists a download that starts now at `path`.
    pub fn begin(&mut self, path: PathBuf) -> DownloadId {
        self.history.begin(path)
    }

    pub fn progress(&mut self, id: DownloadId, received: u64, total: Option<u64>) {
        self.history.progress(id, received, total);
    }

    pub fn finish(&mut self, id: DownloadId) {
        self.history.finish(id);
        self.save_history();
    }

    pub fn fail(&mut self, id: DownloadId) {
        self.history.fail(id);
    }

    /// Forgets one download (cancelled, or removed from the history); its file stays.
    pub fn remove(&mut self, id: DownloadId) {
        self.history.discard(id);
        self.save_history();
    }

    pub fn clear_history(&mut self) {
        self.history.forget_all();
        self.save_history();
    }

    /// The `count` newest downloads after `cleared_through` as a JSON array for the interface.
    #[must_use]
    pub fn rows_json(&self, cleared_through: Option<DownloadId>, count: usize) -> Value {
        Value::Array(self.history.latest_after(cleared_through, count).into_iter().map(Self::row).collect())
    }

    fn row(entry: &ztbrowser_core::download::download_history::DownloadEntry) -> Value {
        let (state, received, total) = match entry.state() {
            DownloadState::InProgress { received, total } => ("progress", *received, *total),
            DownloadState::Finished => ("finished", 0, None),
            DownloadState::Failed => ("failed", 0, None),
        };
        json!({
            "id": entry.id().value(),
            "name": entry.name(),
            "path": entry.path(),
            "state": state,
            "received": received,
            "total": total,
            "missing": *entry.state() == DownloadState::Finished && !entry.path().exists(),
        })
    }

    fn save_history(&self) {
        // A history that cannot be written is lost at exit, never worth interrupting a download for.
        let _ = self.history_file.write(&self.history.to_json());
    }

    fn save_settings(&self) {
        let _ = self.settings_file.write(&self.settings.to_json());
    }
}
