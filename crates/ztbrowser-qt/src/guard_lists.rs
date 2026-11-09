//! ZTBrowser Guard's filter lists as the downloader left them in the cache, and the engine built from them.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use ztbrowser_core::guard::filter_list_source::FilterListSource;
use ztbrowser_core::guard::guard_engine::GuardEngine;
use ztbrowser_core::guard::shared_guard_engine::SharedGuardEngine;

pub struct GuardLists {
    directory: PathBuf,
}

impl GuardLists {
    /// `directory` is the protection cache; the lists are in its `filter-lists` folder, one `<id>.txt` each.
    #[must_use]
    pub fn in_cache(directory: &Path) -> Self {
        Self { directory: directory.join("filter-lists") }
    }

    /// The text of every list that was downloaded; lists never downloaded or unreadable are left out.
    #[must_use]
    pub fn downloaded(&self) -> Vec<String> {
        FilterListSource::ALL
            .iter()
            .filter_map(|source| fs::read_to_string(self.directory.join(format!("{}.txt", source.id()))).ok())
            .collect()
    }

    /// Builds the engine on a worker thread and installs it in `engine` when ready; pages loaded before that
    /// are not filtered.
    pub fn build_into(self, engine: Arc<SharedGuardEngine>) {
        thread::spawn(move || self.install_into(&engine));
    }

    /// Builds the engine from the lists now and makes it the one in use, replacing the previous one.
    pub fn install_into(&self, engine: &SharedGuardEngine) {
        let lists = self.downloaded();
        let texts: Vec<&str> = lists.iter().map(String::as_str).collect();
        engine.install(GuardEngine::from_lists(&texts));
    }
}
