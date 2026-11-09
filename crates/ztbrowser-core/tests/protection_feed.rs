//! DuckDuckGo's protection data and the community filter lists are downloaded, validated and kept.
//!
//! A download replaces the cached copy only after it was recognised as the expected kind of document, so a
//! captive portal's page or a truncated body never replaces a good copy; a refresh costs one small request
//! when nothing changed.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

use tempfile::TempDir;
use ztbrowser_core::guard::filter_list_source::FilterListSource;
use ztbrowser_core::protection::document_fetcher::DocumentFetcher;
use ztbrowser_core::protection::document_fetcher::FetchFailure;
use ztbrowser_core::protection::document_fetcher::FetchedDocument;
use ztbrowser_core::protection::protection_feed::ProtectionFeed;

const TRACKER_DATA: &str = "https://staticcdn.duckduckgo.com/trackerblocking/v6/current/macos-tds.json";
const CONFIGURATION: &str = "https://staticcdn.duckduckgo.com/trackerblocking/config/v4/macos-config.json";
const TDS_BODY: &str = r#"{"trackers": {"tracker.example": {"default": "block"}}}"#;
const CONFIGURATION_BODY: &str = r#"{"features": {"contentBlocking": {"state": "enabled"}}}"#;
const LIST_BODY: &str = "[Adblock Plus 2.0]\n||ads.example^\n";

/// What an address serves: a body with its validator, or the reason it fails.
type Served = Result<(Vec<u8>, Option<String>), String>;

/// Serves what it was told to for each address, and remembers the validators it was sent.
struct FakeServer {
    served: Mutex<HashMap<String, Served>>,
    validators_seen: Mutex<Vec<(String, Option<String>)>>,
}

impl FakeServer {
    fn serving_everything() -> Self {
        let server = Self { served: Mutex::new(HashMap::new()), validators_seen: Mutex::new(vec![]) };
        server.serve(TRACKER_DATA, TDS_BODY, Some("\"tds-1\""));
        server.serve(CONFIGURATION, CONFIGURATION_BODY, Some("\"config-1\""));
        for source in FilterListSource::ALL {
            server.serve(source.url(), LIST_BODY, None);
        }
        server
    }

    fn serve(&self, url: &str, body: &str, etag: Option<&str>) {
        self.served.lock().unwrap().insert(url.to_owned(), Ok((body.as_bytes().to_vec(), etag.map(str::to_owned))));
    }

    fn fail(&self, url: &str, reason: &str) {
        self.served.lock().unwrap().insert(url.to_owned(), Err(reason.to_owned()));
    }

    fn validator_sent_for(&self, url: &str) -> Option<String> {
        self.validators_seen
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|(address, _)| address == url)
            .and_then(|(_, etag)| etag.clone())
    }
}

impl DocumentFetcher for FakeServer {
    fn fetch(&self, url: &str, etag: Option<&str>, _max_bytes: usize) -> Result<FetchedDocument, FetchFailure> {
        self.validators_seen.lock().unwrap().push((url.to_owned(), etag.map(str::to_owned)));
        match self.served.lock().unwrap().get(url) {
            Some(Ok((_, Some(current)))) if etag == Some(current.as_str()) => Ok(FetchedDocument::NotModified),
            Some(Ok((bytes, etag))) => Ok(FetchedDocument::Body { bytes: bytes.clone(), etag: etag.clone() }),
            Some(Err(reason)) => Err(FetchFailure::new(reason)),
            None => Err(FetchFailure::new("no such address")),
        }
    }
}

struct FeedScenario {
    directory: TempDir,
    server: FakeServer,
}

impl FeedScenario {
    fn given_an_empty_cache_and_a_server_with_everything() -> Self {
        Self { directory: TempDir::new().unwrap(), server: FakeServer::serving_everything() }
    }

    fn feed(&self) -> ProtectionFeed {
        ProtectionFeed::new(self.directory.path())
    }

    fn cached(&self, relative: &str) -> Option<String> {
        fs::read_to_string(self.directory.path().join(relative)).ok()
    }

    fn plant(&self, relative: &str, content: &str) {
        let path = self.directory.path().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
}

#[test]
fn should_download_the_protection_data_and_every_filter_list_into_the_cache() {
    let scenario = FeedScenario::given_an_empty_cache_and_a_server_with_everything();
    let outcome = scenario.feed().refresh(&scenario.server);
    assert!(outcome.changed());
    assert_eq!(scenario.cached("tracker-data.json").as_deref(), Some(TDS_BODY));
    assert_eq!(scenario.cached("configuration.json").as_deref(), Some(CONFIGURATION_BODY));
    for source in FilterListSource::ALL {
        assert_eq!(scenario.cached(&format!("filter-lists/{}.txt", source.id())).as_deref(), Some(LIST_BODY));
    }
}

#[test]
fn should_ask_whether_the_copy_it_holds_is_still_current() {
    let scenario = FeedScenario::given_an_empty_cache_and_a_server_with_everything();
    scenario.feed().refresh(&scenario.server);
    let outcome = scenario.feed().refresh(&scenario.server);
    assert_eq!(scenario.server.validator_sent_for(TRACKER_DATA).as_deref(), Some("\"tds-1\""));
    assert!(!outcome.changed());
    assert_eq!(scenario.cached("tracker-data.json").as_deref(), Some(TDS_BODY));
}

#[test]
fn should_report_a_change_when_the_server_has_a_newer_document() {
    let scenario = FeedScenario::given_an_empty_cache_and_a_server_with_everything();
    scenario.feed().refresh(&scenario.server);
    scenario.server.serve(TRACKER_DATA, r#"{"trackers": {"other.example": {"default": "block"}}}"#, Some("\"tds-2\""));
    let outcome = scenario.feed().refresh(&scenario.server);
    assert!(outcome.changed());
    assert!(scenario.cached("tracker-data.json").unwrap().contains("other.example"));
}

#[test]
fn should_not_count_a_download_identical_to_the_copy_as_a_change() {
    let scenario = FeedScenario::given_an_empty_cache_and_a_server_with_everything();
    scenario.feed().refresh(&scenario.server);
    scenario.server.serve(TRACKER_DATA, TDS_BODY, None);
    let outcome = scenario.feed().refresh(&scenario.server);
    assert!(!outcome.changed());
}

#[test]
fn should_keep_the_good_copy_when_a_download_is_not_the_expected_document() {
    let scenario = FeedScenario::given_an_empty_cache_and_a_server_with_everything();
    scenario.feed().refresh(&scenario.server);
    scenario.server.serve(TRACKER_DATA, "<html>Log in to the hotel Wi-Fi</html>", Some("\"portal\""));
    let outcome = scenario.feed().refresh(&scenario.server);
    assert_eq!(scenario.cached("tracker-data.json").as_deref(), Some(TDS_BODY));
    assert_eq!(outcome.failures().len(), 1);
}

#[test]
fn should_keep_the_good_copy_and_the_others_refreshing_when_one_download_fails() {
    let scenario = FeedScenario::given_an_empty_cache_and_a_server_with_everything();
    scenario.feed().refresh(&scenario.server);
    scenario.server.fail(FilterListSource::EASYLIST.url(), "connection reset");
    scenario.server.serve(FilterListSource::EASYPRIVACY.url(), "[Adblock Plus 2.0]\n||new.example^\n", None);
    let outcome = scenario.feed().refresh(&scenario.server);
    assert_eq!(scenario.cached("filter-lists/easylist.txt").as_deref(), Some(LIST_BODY));
    assert!(scenario.cached("filter-lists/easyprivacy.txt").unwrap().contains("new.example"));
    assert!(outcome.changed());
    assert_eq!(outcome.failures().len(), 1);
}

#[test]
fn should_reject_a_filter_list_that_does_not_start_like_one() {
    let scenario = FeedScenario::given_an_empty_cache_and_a_server_with_everything();
    scenario.server.serve(FilterListSource::EASYLIST.url(), "Sign in to continue", None);
    scenario.feed().refresh(&scenario.server);
    assert_eq!(scenario.cached("filter-lists/easylist.txt"), None);
}

#[test]
fn should_not_send_a_validator_for_a_copy_that_is_gone() {
    let scenario = FeedScenario::given_an_empty_cache_and_a_server_with_everything();
    scenario.feed().refresh(&scenario.server);
    fs::remove_file(scenario.directory.path().join("tracker-data.json")).unwrap();
    scenario.feed().refresh(&scenario.server);
    assert_eq!(scenario.server.validator_sent_for(TRACKER_DATA), None);
    assert_eq!(scenario.cached("tracker-data.json").as_deref(), Some(TDS_BODY));
}

#[test]
fn should_report_every_failure_when_the_network_is_down() {
    let scenario = FeedScenario::given_an_empty_cache_and_a_server_with_everything();
    scenario.plant("tracker-data.json", TDS_BODY);
    scenario.server.served.lock().unwrap().clear();
    let outcome = scenario.feed().refresh(&scenario.server);
    assert!(!outcome.changed());
    assert_eq!(outcome.failures().len(), 2 + FilterListSource::ALL.len());
    assert!(Path::new(&scenario.directory.path().join("tracker-data.json")).exists());
}
