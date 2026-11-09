//! The community filter lists ZTBrowser Guard reads, downloaded at run time from their publishers and cached
//! next to DuckDuckGo's protection data (never bundled: see `FilterListSource`).

use std::collections::HashMap;
use std::path::Path;

use ztbrowser_core::guard::filter_list_source::FilterListSource;

use crate::feed_document::FeedDocument;
use crate::http_client::HttpClient;

pub struct FilterListFeed {
    lists: Vec<(FilterListSource, FeedDocument<String>)>,
}

impl FilterListFeed {
    const LOG_DOMAIN: &'static str = "ztbrowser";

    pub fn new(cache_directory: &Path) -> Self {
        let directory = cache_directory.join("filter-lists");
        let lists = FilterListSource::ALL
            .into_iter()
            .map(|source| {
                let content = directory.join(format!("{}.txt", source.id()));
                (source, FeedDocument::new(content, source.url(), Self::filter_list))
            })
            .collect();
        Self { lists }
    }

    /// The last good copy of every list downloaded so far.
    pub fn cached(&self) -> HashMap<FilterListSource, String> {
        self.lists.iter().filter_map(|(source, list)| Some((*source, list.cached()?))).collect()
    }

    /// Fresh lists when any changed, `None` when every cached copy is current. A list that cannot be
    /// refreshed is logged and keeps its cached copy; the others still refresh.
    pub async fn refreshed(&self, http: &HttpClient) -> Option<HashMap<FilterListSource, String>> {
        let mut changed = false;
        for (source, list) in &self.lists {
            match list.refresh(http).await {
                Ok(fresh) => changed |= fresh,
                Err(error) => glib::g_warning!(Self::LOG_DOMAIN, "{} refresh failed: {error}", source.name()),
            }
        }
        changed.then(|| self.cached())
    }

    fn filter_list(bytes: &[u8]) -> Option<String> {
        std::str::from_utf8(bytes).ok().filter(|text| FilterListSource::accepts(text)).map(str::to_owned)
    }
}
