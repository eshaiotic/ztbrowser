//! What a profile's downloads do: whether the person is asked where to save each file, and the folder used
//! when they are not asked (kept per profile, in `downloads.json`).

use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;
use serde_json::json;

/// Asking first is the default: a download then never lands somewhere unexpected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadSettings {
    asks_where_to_save: bool,
    folder: Option<PathBuf>,
}

impl Default for DownloadSettings {
    fn default() -> Self {
        Self { asks_where_to_save: true, folder: None }
    }
}

impl DownloadSettings {
    /// Reads the stored document; a missing or malformed value keeps its default, and a folder that is not an
    /// absolute path is ignored.
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let defaults = Self::default();
        let asks_where_to_save =
            document.get("asksWhereToSave").and_then(Value::as_bool).unwrap_or(defaults.asks_where_to_save);
        let folder =
            document.get("folder").and_then(Value::as_str).map(PathBuf::from).filter(|folder| folder.is_absolute());
        Self { asks_where_to_save, folder }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        match &self.folder {
            Some(folder) => json!({"asksWhereToSave": self.asks_where_to_save, "folder": folder}),
            None => json!({"asksWhereToSave": self.asks_where_to_save}),
        }
    }

    #[must_use]
    pub fn asks_where_to_save(&self) -> bool {
        self.asks_where_to_save
    }

    /// The folder the person chose, or `default` (the system's Downloads folder) when they chose none.
    #[must_use]
    pub fn folder_or<'a>(&'a self, default: &'a Path) -> &'a Path {
        self.folder.as_deref().unwrap_or(default)
    }

    #[must_use]
    pub fn with_asking(self, asks_where_to_save: bool) -> Self {
        Self { asks_where_to_save, ..self }
    }

    /// Ignores a folder that is not an absolute path.
    #[must_use]
    pub fn with_folder(self, folder: PathBuf) -> Self {
        if folder.is_absolute() { Self { folder: Some(folder), ..self } } else { self }
    }
}
