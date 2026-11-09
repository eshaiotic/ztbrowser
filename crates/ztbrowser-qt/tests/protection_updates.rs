//! What protection data the browser starts with: the cached copies at once, or a download on the first start.

use std::fs;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use tempfile::TempDir;
use ztbrowser_core::guard::filter_list_source::FilterListSource;
use ztbrowser_core::guard::shared_guard_engine::SharedGuardEngine;
use ztbrowser_core::network::resource_type::ResourceType;
use ztbrowser_core::protection::document_fetcher::DocumentFetcher;
use ztbrowser_core::protection::document_fetcher::FetchFailure;
use ztbrowser_core::protection::document_fetcher::FetchedDocument;
use ztbrowser_core::protection::protection_feed::ProtectionFeed;
use ztbrowser_qt::protection_updates::ProtectionUpdates;

use ztbrowser_qt as _;

const ADS_LIST: &str = "[Adblock Plus 2.0]\n||ads.example^\n";
const AD_ADDRESS: &str = "https://ads.example/banner.png";

/// A server that has the two protection documents (or nothing at all) and counts the requests it got.
struct CountingServer {
    has_documents: bool,
    requests: AtomicUsize,
}

impl CountingServer {
    fn serving_the_documents() -> Arc<Self> {
        Arc::new(Self { has_documents: true, requests: AtomicUsize::new(0) })
    }

    fn unreachable() -> Arc<Self> {
        Arc::new(Self { has_documents: false, requests: AtomicUsize::new(0) })
    }

    fn requests(&self) -> usize {
        self.requests.load(Ordering::SeqCst)
    }
}

impl DocumentFetcher for CountingServer {
    fn fetch(&self, url: &str, _etag: Option<&str>, _max_bytes: usize) -> Result<FetchedDocument, FetchFailure> {
        self.requests.fetch_add(1, Ordering::SeqCst);
        let body = match url {
            ProtectionFeed::TRACKER_DATA_URL if self.has_documents => r#"{"trackers":{}}"#,
            ProtectionFeed::CONFIGURATION_URL if self.has_documents => r#"{"features":{}}"#,
            url if self.has_documents && url == FilterListSource::EASYLIST.url() => ADS_LIST,
            _ => return Err(FetchFailure::new("no route to host")),
        };
        Ok(FetchedDocument::Body { bytes: body.as_bytes().to_vec(), etag: None })
    }
}

fn cache_holding_both_documents() -> TempDir {
    let cache = TempDir::new().expect("cache");
    fs::write(cache.path().join("tracker-data.json"), r#"{"trackers":{}}"#).expect("tracker data");
    fs::write(cache.path().join("configuration.json"), r#"{"features":{}}"#).expect("configuration");
    cache
}

#[test]
fn should_start_with_the_cached_documents_without_downloading() {
    let cache = cache_holding_both_documents();
    let server = CountingServer::serving_the_documents();
    let guard = Arc::new(SharedGuardEngine::default());
    let updates = ProtectionUpdates::new(cache.path().to_path_buf(), Arc::clone(&server) as Arc<dyn DocumentFetcher>);

    let starting = updates.starting_protection(&guard);

    assert!(starting.documents.is_some());
    assert!(!starting.is_first_start);
    assert_eq!(server.requests(), 0);
}

#[test]
fn should_download_the_documents_first_when_nothing_is_cached() {
    let cache = TempDir::new().expect("cache");
    let server = CountingServer::serving_the_documents();
    let guard = Arc::new(SharedGuardEngine::default());
    let updates = ProtectionUpdates::new(cache.path().to_path_buf(), Arc::clone(&server) as Arc<dyn DocumentFetcher>);

    let starting = updates.starting_protection(&guard);

    assert!(starting.documents.is_some());
    assert!(starting.is_first_start);
}

#[test]
fn should_start_without_documents_when_the_first_download_fails() {
    let cache = TempDir::new().expect("cache");
    let server = CountingServer::unreachable();
    let guard = Arc::new(SharedGuardEngine::default());
    let updates = ProtectionUpdates::new(cache.path().to_path_buf(), server as Arc<dyn DocumentFetcher>);

    let starting = updates.starting_protection(&guard);

    assert!(starting.documents.is_none());
    assert!(starting.is_first_start);
}

/// The engine is built on a worker thread, so the test waits for it, up to a generous limit.
fn engine_blocks_the_advertisement(guard: &SharedGuardEngine) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if guard.get().is_some_and(|engine| engine.blocks(AD_ADDRESS, "https://news.test/", ResourceType::Image)) {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    false
}

#[test]
fn should_filter_with_the_lists_it_has_just_downloaded_on_the_first_start() {
    let cache = TempDir::new().expect("cache");
    let server = CountingServer::serving_the_documents();
    let guard = Arc::new(SharedGuardEngine::default());
    let updates = ProtectionUpdates::new(cache.path().to_path_buf(), server as Arc<dyn DocumentFetcher>);

    let _starting = updates.starting_protection(&guard);

    assert!(engine_blocks_the_advertisement(&guard));
}

#[test]
fn should_filter_with_the_lists_already_cached_on_a_later_start() {
    let cache = cache_holding_both_documents();
    let lists = cache.path().join("filter-lists");
    fs::create_dir_all(&lists).expect("lists folder");
    fs::write(lists.join(format!("{}.txt", FilterListSource::EASYLIST.id())), ADS_LIST).expect("cached list");
    let server = CountingServer::unreachable();
    let guard = Arc::new(SharedGuardEngine::default());
    let updates = ProtectionUpdates::new(cache.path().to_path_buf(), server as Arc<dyn DocumentFetcher>);

    let _starting = updates.starting_protection(&guard);

    assert!(engine_blocks_the_advertisement(&guard));
}
