//! What burning data deletes, so the engine adapter can carry it out.

use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BurnPlan {
    /// Cookies of these sites are deleted; empty together with `delete_all_cookies` means none.
    pub cookie_sites: Vec<String>,
    pub delete_all_cookies: bool,
    pub clear_cache: bool,
    /// Directories deleted while no page uses them: the engine's whole storage directory and the cache. Empty
    /// for a single site: engines offer no per-site deletion for these, so burning one site clears cookies only.
    pub storage_directories: Vec<PathBuf>,
}
