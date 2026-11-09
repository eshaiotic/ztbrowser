//! One document ZTBrowser downloads at run time and keeps in its cache directory: DuckDuckGo's protection data
//! or a community filter list.
//!
//! Refreshes are conditional (`ETag`), so an unchanged document costs one small request; the Tor window's
//! client sends no `ETag`, and a download identical to the cached copy counts as unchanged. A download
//! replaces the cached copy only after it was recognised as the expected kind of document, so a captive
//! portal's page or a truncated body never replaces a good copy.

use std::error::Error;
use std::path::PathBuf;

use crate::http_client::Fetched;
use crate::http_client::HttpClient;
use crate::stored_file::StoredFile;

pub struct FeedDocument<T> {
    url: &'static str,
    content: StoredFile,
    etag: StoredFile,
    /// The document a download holds, `None` when it is not the expected kind.
    parse: fn(&[u8]) -> Option<T>,
}

impl<T> FeedDocument<T> {
    const MAX_BYTES: usize = 32 * 1024 * 1024;

    /// The document at `url`, cached as `content` with its `ETag` next to it (`tracker-data.etag`).
    pub fn new(content: PathBuf, url: &'static str, parse: fn(&[u8]) -> Option<T>) -> Self {
        Self { url, etag: StoredFile::new(content.with_extension("etag")), content: StoredFile::new(content), parse }
    }

    /// The last good copy, if one was ever downloaded.
    pub fn cached(&self) -> Option<T> {
        (self.parse)(&self.content.read_bytes()?)
    }

    /// Downloads a newer copy; `Ok(false)` when the cached copy is still current.
    ///
    /// # Errors
    /// Network or HTTP failures, or a download that is not the expected document; the cached copy is kept.
    pub async fn refresh(&self, http: &HttpClient) -> Result<bool, Box<dyn Error>> {
        let etag = self.etag.read_text().filter(|_| self.content.exists());
        match http.get(self.url, Self::MAX_BYTES, etag.as_deref()).await? {
            Fetched::NotModified => Ok(false),
            Fetched::Body { bytes, etag } => {
                let is_unchanged = self.content.read_bytes().is_some_and(|cached| cached == bytes);
                if !is_unchanged {
                    if (self.parse)(&bytes).is_none() {
                        return Err(tr!("{url} did not return the expected document", url = self.url).into());
                    }
                    self.content.write(&bytes)?;
                }
                match etag {
                    Some(etag) => self.etag.write(etag.as_bytes())?,
                    None => self.etag.remove()?,
                }
                Ok(!is_unchanged)
            }
        }
    }
}
