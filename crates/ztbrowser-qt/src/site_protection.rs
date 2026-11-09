//! Which sites the person switched protection off for: the shield button's state, kept in the profile.

use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;

use ztbrowser_core::profile::unprotected_sites::UnprotectedSites;
use ztbrowser_core::storage::owned_json_file::OwnedJsonFile;

/// The list the request policies read, shared with them, and the file that keeps it.
pub struct SiteProtection {
    sites: Arc<RwLock<UnprotectedSites>>,
    file: OwnedJsonFile,
}

impl SiteProtection {
    #[must_use]
    pub fn open(directory: &Path) -> Self {
        let file = OwnedJsonFile::new(directory.join("unprotected-sites.json"));
        let sites = file.read().map(|document| UnprotectedSites::from_json(&document)).unwrap_or_default();
        Self { sites: Arc::new(RwLock::new(sites)), file }
    }

    /// The list the request policies consult; a change here takes effect on the next request.
    #[must_use]
    pub fn shared_list(&self) -> Arc<RwLock<UnprotectedSites>> {
        Arc::clone(&self.sites)
    }

    #[must_use]
    pub fn is_protected(&self, host: &str) -> bool {
        !self.sites.read().unwrap_or_else(std::sync::PoisonError::into_inner).is_unprotected(host)
    }

    pub fn set_protected(&self, host: &str, protected: bool) {
        let mut sites = self.sites.write().unwrap_or_else(std::sync::PoisonError::into_inner);
        if protected {
            sites.switch_protection_on(host);
        } else {
            sites.switch_protection_off(host);
        }
        // A list that cannot be written is lost at exit; the switch still works for this session.
        let _ = self.file.write(&sites.to_json());
    }
}
