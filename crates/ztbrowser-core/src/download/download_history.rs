//! The downloads of a profile, newest first: the long history the downloads manager lists, from which each
//! window's downloads list shows the latest few.
//!
//! Only finished downloads outlive the process (`downloads.json`): a download that was running or failed
//! when the browser closed has no file worth offering.

use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;
use serde_json::json;

/// Names one download while it runs; never reused within a process.
/// A later download always has a greater id, so ids also say which of two downloads came first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DownloadId(u64);

impl DownloadId {
    /// The number a user interface keeps to name this download.
    #[must_use]
    pub fn value(self) -> u64 {
        self.0
    }

    /// The id a user interface got from [`Self::value`].
    #[must_use]
    pub fn from_value(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadState {
    /// `total` is `None` when the server did not say how large the file is.
    InProgress {
        received: u64,
        total: Option<u64>,
    },
    Finished,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadEntry {
    id: DownloadId,
    path: PathBuf,
    state: DownloadState,
}

impl DownloadEntry {
    #[must_use]
    pub fn id(&self) -> DownloadId {
        self.id
    }

    /// The file's full path.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The file's name, as the list shows it.
    #[must_use]
    pub fn name(&self) -> &str {
        self.path.file_name().and_then(|name| name.to_str()).unwrap_or_default()
    }

    #[must_use]
    pub fn state(&self) -> &DownloadState {
        &self.state
    }
}

#[derive(Debug, Clone, Default)]
pub struct DownloadHistory {
    entries: Vec<DownloadEntry>,
    next_id: u64,
}

impl DownloadHistory {
    /// How many downloads the history keeps; the oldest are forgotten beyond it.
    pub const CAPACITY: usize = 500;

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reads the stored document: every entry with an absolute path is a finished download.
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let mut downloads = Self::new();
        let stored = document.get("downloads").and_then(Value::as_array).map(Vec::as_slice).unwrap_or_default();
        for entry in stored.iter().rev() {
            let path = entry.get("path").and_then(Value::as_str).map(PathBuf::from).filter(|path| path.is_absolute());
            if let Some(path) = path {
                let id = downloads.begin(path);
                downloads.finish(id);
            }
        }
        downloads
    }

    /// The finished downloads only, newest first.
    #[must_use]
    pub fn to_json(&self) -> Value {
        let finished: Vec<Value> = self
            .entries
            .iter()
            .filter(|entry| entry.state == DownloadState::Finished)
            .map(|entry| json!({"path": entry.path}))
            .collect();
        json!({"downloads": finished})
    }

    /// Newest first.
    #[must_use]
    pub fn entries(&self) -> &[DownloadEntry] {
        &self.entries
    }

    /// The newest `count` downloads that came after `cleared_through` (all of them when it is `None`), plus any
    /// download still running: what a window's downloads list shows after its person cleared that list.
    #[must_use]
    pub fn latest_after(&self, cleared_through: Option<DownloadId>, count: usize) -> Vec<&DownloadEntry> {
        self.entries
            .iter()
            .filter(|entry| {
                matches!(entry.state, DownloadState::InProgress { .. })
                    || cleared_through.is_none_or(|cleared| entry.id > cleared)
            })
            .take(count)
            .collect()
    }

    /// Adds a download that has just started as the newest, dropping the oldest beyond [`Self::CAPACITY`].
    pub fn begin(&mut self, path: PathBuf) -> DownloadId {
        let id = DownloadId(self.next_id);
        self.next_id += 1;
        self.entries
            .insert(0, DownloadEntry { id, path, state: DownloadState::InProgress { received: 0, total: None } });
        self.entries.truncate(Self::CAPACITY);
        id
    }

    pub fn progress(&mut self, id: DownloadId, received: u64, total: Option<u64>) {
        self.set_state(id, DownloadState::InProgress { received, total });
    }

    pub fn finish(&mut self, id: DownloadId) {
        self.set_state(id, DownloadState::Finished);
    }

    pub fn fail(&mut self, id: DownloadId) {
        self.set_state(id, DownloadState::Failed);
    }

    /// Forgets a download: one the person cancelled, or removed from the history. Its file is left alone.
    pub fn discard(&mut self, id: DownloadId) {
        self.entries.retain(|entry| entry.id != id);
    }

    /// Forgets every download, running ones included, for a profile whose browsing data was burned. A running
    /// one that finishes later is not listed (nor stored): it has no entry to update.
    pub fn forget_all(&mut self) {
        self.entries.clear();
    }

    fn set_state(&mut self, id: DownloadId, state: DownloadState) {
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id) {
            entry.state = state;
        }
    }
}
