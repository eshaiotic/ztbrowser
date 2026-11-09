//! DuckDuckGo's Tracker Data Set and privacy configuration, and the community filter lists Guard reads,
//! downloaded at run time into the cache directory.
//!
//! The Tracker Data Set is licensed CC BY-NC-SA 4.0 and the lists have their own licences, so none of them is
//! bundled: ZTBrowser downloads them (the macOS edition of DuckDuckGo's data, from DuckDuckGo's CDN) and keeps
//! the last good copy of each. A copy that cannot be refreshed stays in use; the others still refresh.

use std::path::Path;
use std::thread;

use crate::guard::filter_list_source::FilterListSource;
use crate::protection::cached_document::CachedDocument;
use crate::protection::document_fetcher::DocumentFetcher;
use crate::protection::document_fetcher::FetchFailure;

pub struct ProtectionFeed {
    documents: Vec<(String, CachedDocument)>,
}

/// What a refresh did: whether anything changed, and what could not be refreshed.
#[derive(Debug, Default)]
pub struct FeedRefresh {
    changed: bool,
    failures: Vec<String>,
}

impl FeedRefresh {
    #[must_use]
    pub fn changed(&self) -> bool {
        self.changed
    }

    /// One line per document that could not be refreshed, naming it and the reason.
    #[must_use]
    pub fn failures(&self) -> &[String] {
        &self.failures
    }
}

impl ProtectionFeed {
    pub const TRACKER_DATA_URL: &'static str =
        "https://staticcdn.duckduckgo.com/trackerblocking/v6/current/macos-tds.json";
    pub const CONFIGURATION_URL: &'static str =
        "https://staticcdn.duckduckgo.com/trackerblocking/config/v4/macos-config.json";

    #[must_use]
    pub fn new(cache_directory: &Path) -> Self {
        let mut documents = vec![
            (
                "tracker data".to_owned(),
                CachedDocument::new(
                    cache_directory.join("tracker-data.json"),
                    Self::TRACKER_DATA_URL,
                    Self::is_tracker_data,
                ),
            ),
            (
                "privacy configuration".to_owned(),
                CachedDocument::new(
                    cache_directory.join("configuration.json"),
                    Self::CONFIGURATION_URL,
                    Self::is_configuration,
                ),
            ),
        ];
        let lists = cache_directory.join("filter-lists");
        documents.extend(FilterListSource::ALL.into_iter().map(|source| {
            let content = lists.join(format!("{}.txt", source.id()));
            (source.name().to_owned(), CachedDocument::new(content, source.url(), Self::is_filter_list))
        }));
        Self { documents }
    }

    /// Refreshes every document, all at once (there are eleven, from different servers). A failure is
    /// recorded and the others carry on.
    pub fn refresh(&self, fetcher: &dyn DocumentFetcher) -> FeedRefresh {
        let results: Vec<_> = thread::scope(|scope| {
            let running: Vec<_> = self
                .documents
                .iter()
                .map(|(name, document)| (name, scope.spawn(move || document.refresh(fetcher))))
                .collect();
            running
                .into_iter()
                .map(|(name, task)| {
                    (name, task.join().unwrap_or_else(|_| Err(FetchFailure::new("the download stopped unexpectedly"))))
                })
                .collect()
        });
        let mut outcome = FeedRefresh::default();
        for (name, result) in results {
            match result {
                Ok(changed) => outcome.changed |= changed,
                Err(failure) => outcome.failures.push(Self::describe(name, &failure)),
            }
        }
        outcome
    }

    fn describe(name: &str, failure: &FetchFailure) -> String {
        format!("{name}: {}", failure.reason())
    }

    fn is_tracker_data(bytes: &[u8]) -> bool {
        Self::is_json_object_with(bytes, "trackers")
    }

    fn is_configuration(bytes: &[u8]) -> bool {
        Self::is_json_object_with(bytes, "features")
    }

    /// A JSON document with the top-level object `key`, which every genuine document of its kind has.
    fn is_json_object_with(bytes: &[u8], key: &str) -> bool {
        serde_json::from_slice::<serde_json::Value>(bytes)
            .is_ok_and(|document| document.get(key).is_some_and(serde_json::Value::is_object))
    }

    fn is_filter_list(bytes: &[u8]) -> bool {
        std::str::from_utf8(bytes).is_ok_and(FilterListSource::accepts)
    }
}
