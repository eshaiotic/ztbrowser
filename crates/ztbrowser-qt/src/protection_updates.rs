//! Keeps DuckDuckGo's protection data and Guard's filter lists up to date, and puts what arrives to use.
//!
//! The first start has nothing cached, so the data is downloaded before the window opens: a page must never
//! load unprotected because the rules were still on their way. Later starts use the cached copies at once and
//! check for newer ones in the background, again every few hours; whatever changed replaces the rules in
//! force without a restart.

use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use ztbrowser_core::guard::shared_guard_engine::SharedGuardEngine;
use ztbrowser_core::protection::document_fetcher::DocumentFetcher;
use ztbrowser_core::protection::protection_feed::FeedRefresh;
use ztbrowser_core::protection::protection_feed::ProtectionFeed;

use crate::guard_lists::GuardLists;
use crate::profile_policies::ProfilePolicies;
use crate::protection_documents::ProtectionDocuments;

pub struct ProtectionUpdates {
    feed: ProtectionFeed,
    fetcher: Arc<dyn DocumentFetcher>,
    cache: PathBuf,
}

/// The protection data a start begins with.
pub struct StartingProtection {
    /// `None` when there was nothing cached and the download failed: the policies then run without tracker
    /// blocking until a later refresh succeeds.
    pub documents: Option<ProtectionDocuments>,
    /// Nothing was cached, so the data was downloaded just now.
    pub is_first_start: bool,
}

impl ProtectionUpdates {
    /// How long the browser runs between two checks for newer data.
    const CHECK_INTERVAL: Duration = Duration::from_hours(6);

    #[must_use]
    pub fn new(cache: PathBuf, fetcher: Arc<dyn DocumentFetcher>) -> Self {
        Self { feed: ProtectionFeed::new(&cache), fetcher, cache }
    }

    /// The cached documents, read once. With nothing cached (the first start) they are downloaded first, so
    /// the caller waits for the network only then.
    ///
    /// Guard's engine is built from the lists only after that download, so on the first start it is built from
    /// the lists just fetched and not from an empty cache; the build itself runs on its own thread, and pages
    /// loaded before it finishes are not filtered.
    #[must_use]
    pub fn starting_protection(&self, guard: &Arc<SharedGuardEngine>) -> StartingProtection {
        let cached = ProtectionDocuments::from_cache(&self.cache);
        let is_first_start = cached.is_none();
        if is_first_start {
            self.refresh();
        }
        GuardLists::in_cache(&self.cache).build_into(Arc::clone(guard));
        let documents = if is_first_start { ProtectionDocuments::from_cache(&self.cache) } else { cached };
        StartingProtection { documents, is_first_start }
    }

    /// Downloads what is missing or newer. Failures are reported on the error stream; the copies already held
    /// stay in use.
    pub fn refresh(&self) -> FeedRefresh {
        let outcome = self.feed.refresh(self.fetcher.as_ref());
        for failure in outcome.failures() {
            eprintln!("ztbrowser: protection data not refreshed: {failure}");
        }
        outcome
    }

    /// Checks for newer data now and then for as long as the browser runs, applying what changed to `policies`
    /// and `guard`. With `check_now` false the first check waits one interval (the data was just downloaded).
    pub fn keep_current(
        self: Arc<Self>,
        policies: Arc<ProfilePolicies>,
        guard: Arc<SharedGuardEngine>,
        check_now: bool,
    ) {
        thread::spawn(move || {
            let mut pause = !check_now;
            loop {
                if pause {
                    thread::sleep(Self::CHECK_INTERVAL);
                }
                pause = true;
                if self.refresh().changed() {
                    if let Some(documents) = ProtectionDocuments::from_cache(&self.cache) {
                        policies.use_documents(&documents);
                    }
                    GuardLists::in_cache(&self.cache).install_into(&guard);
                }
            }
        });
    }
}
