//! Downloads documents with Qt's network stack, for the protection feed.

use ztbrowser_core::protection::document_fetcher::DocumentFetcher;
use ztbrowser_core::protection::document_fetcher::FetchFailure;
use ztbrowser_core::protection::document_fetcher::FetchedDocument;

use crate::engine_bridge::ffi;

pub struct QtDocumentFetcher;

impl QtDocumentFetcher {
    /// A download that makes no progress for this long is given up.
    const TIMEOUT_MILLISECONDS: i32 = 20_000;
    const STATUS_OK: i32 = 200;
    const STATUS_NOT_MODIFIED: i32 = 304;
}

impl DocumentFetcher for QtDocumentFetcher {
    fn fetch(&self, url: &str, etag: Option<&str>, max_bytes: usize) -> Result<FetchedDocument, FetchFailure> {
        let answer = ffi::fetch_bytes(url, etag.unwrap_or_default(), max_bytes as u64, Self::TIMEOUT_MILLISECONDS);
        match answer.status {
            Self::STATUS_NOT_MODIFIED if etag.is_some() => Ok(FetchedDocument::NotModified),
            Self::STATUS_OK => Ok(FetchedDocument::Body {
                bytes: answer.body,
                etag: Some(answer.etag).filter(|etag| !etag.is_empty()),
            }),
            0 => Err(FetchFailure::new(&format!("{url}: {}", answer.error))),
            status => Err(FetchFailure::new(&format!("{url} answered {status}"))),
        }
    }
}
