//! One document ZTBrowser downloads at run time and keeps in its cache directory.
//!
//! Refreshes are conditional (`ETag`), so an unchanged document costs one small request, and a download
//! identical to the cached copy counts as unchanged. A download replaces the cached copy only after it was
//! recognised as the expected kind of document, so a captive portal's page or a truncated body never replaces
//! a good copy.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use crate::protection::document_fetcher::DocumentFetcher;
use crate::protection::document_fetcher::FetchFailure;
use crate::protection::document_fetcher::FetchedDocument;

pub struct CachedDocument {
    url: &'static str,
    content: PathBuf,
    etag: PathBuf,
    is_expected_kind: fn(&[u8]) -> bool,
}

impl CachedDocument {
    const MAX_BYTES: usize = 32 * 1024 * 1024;

    /// The document at `url`, cached as `content` with its `ETag` next to it (`tracker-data.etag`).
    #[must_use]
    pub fn new(content: PathBuf, url: &'static str, is_expected_kind: fn(&[u8]) -> bool) -> Self {
        Self { url, etag: content.with_extension("etag"), content, is_expected_kind }
    }

    /// Downloads a newer copy; `Ok(true)` when the cached copy changed.
    ///
    /// # Errors
    /// The download failed, was not the expected document, or could not be stored; the cached copy is kept.
    pub fn refresh(&self, fetcher: &dyn DocumentFetcher) -> Result<bool, FetchFailure> {
        let validator = fs::read_to_string(&self.etag).ok().filter(|_| self.content.exists());
        match fetcher.fetch(self.url, validator.as_deref(), Self::MAX_BYTES)? {
            FetchedDocument::NotModified => Ok(false),
            FetchedDocument::Body { bytes, etag } => {
                let is_unchanged = fs::read(&self.content).is_ok_and(|cached| cached == bytes);
                if !is_unchanged {
                    if !(self.is_expected_kind)(&bytes) {
                        return Err(FetchFailure::new(&format!("{} did not return the expected document", self.url)));
                    }
                    Self::store(&self.content, &bytes).map_err(|error| Self::storage_failure(self.url, &error))?;
                }
                match etag {
                    Some(etag) => Self::store(&self.etag, etag.as_bytes()),
                    None => fs::remove_file(&self.etag)
                        .or_else(|error| if error.kind() == io::ErrorKind::NotFound { Ok(()) } else { Err(error) }),
                }
                .map_err(|error| Self::storage_failure(self.url, &error))?;
                Ok(!is_unchanged)
            }
        }
    }

    fn store(path: &Path, bytes: &[u8]) -> io::Result<()> {
        if let Some(directory) = path.parent() {
            fs::create_dir_all(directory)?;
        }
        // Written beside the target and moved into place, so a crash never leaves half a document.
        let partial = path.with_extension("partial");
        fs::write(&partial, bytes)?;
        fs::rename(partial, path)
    }

    fn storage_failure(url: &str, error: &io::Error) -> FetchFailure {
        FetchFailure::new(&format!("{url} could not be stored: {error}"))
    }
}
