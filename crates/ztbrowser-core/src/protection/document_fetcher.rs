//! How the protection feed reaches the network, without knowing which HTTP stack does it.
//!
//! The browser's own downloads (protection data, filter lists) are plain `GET`s with no cookies and no
//! identifier; the engine adapter provides the implementation, and the tests provide a table.

/// What a server answered to a request that may have carried the validator (`ETag`) of the copy already held.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchedDocument {
    /// The copy the validator names is still current.
    NotModified,
    Body {
        bytes: Vec<u8>,
        etag: Option<String>,
    },
}

/// Why a document could not be fetched, in words for a log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchFailure {
    reason: String,
}

impl FetchFailure {
    #[must_use]
    pub fn new(reason: &str) -> Self {
        Self { reason: reason.to_owned() }
    }

    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// `Send + Sync` because refreshes run on a worker thread while the interface thread keeps going.
pub trait DocumentFetcher: Send + Sync {
    /// `GET url`, with `If-None-Match: etag` when given. A body longer than `max_bytes` is a failure.
    ///
    /// # Errors
    /// Network failures, answers other than success or not-modified, and bodies that are too large.
    fn fetch(&self, url: &str, etag: Option<&str>, max_bytes: usize) -> Result<FetchedDocument, FetchFailure>;
}
