//! DuckDuckGo's Tracker Data Set and privacy configuration, as last downloaded.
//!
//! The documents are CC BY-NC-SA 4.0 data that is downloaded at run time and never shipped; this reads the
//! copies the downloader left in the cache directory (written by the GTK edition today).

use std::fs;
use std::path::Path;

use serde_json::Value;

pub struct ProtectionDocuments {
    pub tracker_data: Value,
    pub configuration: Value,
}

impl ProtectionDocuments {
    /// The cached copies, or `None` when either was never downloaded or is unreadable.
    #[must_use]
    pub fn from_cache(directory: &Path) -> Option<Self> {
        Some(Self {
            tracker_data: Self::read(&directory.join("tracker-data.json"))?,
            configuration: Self::read(&directory.join("configuration.json"))?,
        })
    }

    fn read(path: &Path) -> Option<Value> {
        serde_json::from_slice(&fs::read(path).ok()?).ok()
    }
}
