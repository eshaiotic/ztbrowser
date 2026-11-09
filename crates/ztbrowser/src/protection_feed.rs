//! DuckDuckGo's Tracker Data Set and privacy configuration, downloaded at run time.
//!
//! The Tracker Data Set is licensed CC BY-NC-SA 4.0, so it is never bundled: ZTBrowser downloads the macOS
//! edition from DuckDuckGo's CDN and keeps the last good copy in the cache directory (see [`FeedDocument`]).

use std::error::Error;
use std::path::Path;

use serde_json::Value;

use crate::feed_document::FeedDocument;
use crate::http_client::HttpClient;

pub struct ProtectionDocuments {
    pub tracker_data: Value,
    pub configuration: Value,
}

pub struct ProtectionFeed {
    tracker_data: FeedDocument<Value>,
    configuration: FeedDocument<Value>,
}

impl ProtectionFeed {
    const TRACKER_DATA_URL: &'static str = "https://staticcdn.duckduckgo.com/trackerblocking/v6/current/macos-tds.json";
    const CONFIGURATION_URL: &'static str =
        "https://staticcdn.duckduckgo.com/trackerblocking/config/v4/macos-config.json";

    pub fn new(cache_directory: &Path) -> Self {
        Self {
            tracker_data: FeedDocument::new(
                cache_directory.join("tracker-data.json"),
                Self::TRACKER_DATA_URL,
                Self::tracker_data_document,
            ),
            configuration: FeedDocument::new(
                cache_directory.join("configuration.json"),
                Self::CONFIGURATION_URL,
                Self::configuration_document,
            ),
        }
    }

    /// The last good copies, if both were ever downloaded.
    pub fn cached(&self) -> Option<ProtectionDocuments> {
        Some(ProtectionDocuments {
            tracker_data: self.tracker_data.cached()?,
            configuration: self.configuration.cached()?,
        })
    }

    /// Fresh documents when either changed, `None` when both cached copies are current.
    ///
    /// # Errors
    /// Network, HTTP or parse failures; the cached copies are left untouched.
    pub async fn refreshed(&self, http: &HttpClient) -> Result<Option<ProtectionDocuments>, Box<dyn Error>> {
        let tracker_data_changed = self.tracker_data.refresh(http).await?;
        let configuration_changed = self.configuration.refresh(http).await?;
        if !(tracker_data_changed || configuration_changed) {
            return Ok(None);
        }
        Ok(self.cached())
    }

    fn tracker_data_document(bytes: &[u8]) -> Option<Value> {
        Self::document_with(bytes, "trackers")
    }

    fn configuration_document(bytes: &[u8]) -> Option<Value> {
        Self::document_with(bytes, "features")
    }

    /// A JSON document with the top-level object `key`, which every genuine document of its kind has.
    fn document_with(bytes: &[u8], key: &str) -> Option<Value> {
        serde_json::from_slice::<Value>(bytes).ok().filter(|document| document.get(key).is_some_and(Value::is_object))
    }
}
