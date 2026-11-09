//! Where each profile keeps its data on disk.
//!
//! Isolation is physical: WebKit gets a different data directory (cookies, local storage, IndexedDB,
//! service workers, history) and cache directory per profile, so two profiles share nothing.

use std::path::Path;
use std::path::PathBuf;

use crate::profile::profile_id::ProfileId;

/// The two directories of one profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileLocation {
    pub data: PathBuf,
    pub cache: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ProfileDirectories {
    data_root: PathBuf,
    cache_root: PathBuf,
}

impl ProfileDirectories {
    const PROFILES: &'static str = "profiles";

    /// `data_root` is typically `~/.local/share/ztbrowser`, `cache_root` `~/.cache/ztbrowser`.
    #[must_use]
    pub fn new(data_root: &Path, cache_root: &Path) -> Self {
        Self { data_root: data_root.to_owned(), cache_root: cache_root.to_owned() }
    }

    /// Safe by construction: a [`ProfileId`] cannot contain path separators or `..`.
    #[must_use]
    pub fn for_profile(&self, id: &ProfileId) -> ProfileLocation {
        ProfileLocation {
            data: self.data_root.join(Self::PROFILES).join(id.as_str()),
            cache: self.cache_root.join(Self::PROFILES).join(id.as_str()),
        }
    }
}
