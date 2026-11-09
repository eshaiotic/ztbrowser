//! Works out what to delete when the person burns a profile's data (the Fire Button).

use crate::profile::profile_directories::ProfileLocation;
use crate::storage::burn_plan::BurnPlan;
use crate::storage::burn_scope::BurnScope;

pub struct BurnPlanner {
    location: ProfileLocation,
}

impl BurnPlanner {
    /// Everything Chromium keeps for a profile lives in this one directory (the adapter points the engine at
    /// it), so burning deletes the directory as a whole instead of naming what Chromium stores today.
    /// A list of names would silently miss each new kind of storage a later engine version adds.
    pub const ENGINE_DIRECTORY: &'static str = "engine";

    #[must_use]
    pub fn new(location: ProfileLocation) -> Self {
        Self { location }
    }

    #[must_use]
    pub fn plan(&self, scope: &BurnScope) -> BurnPlan {
        match scope {
            BurnScope::Everything => BurnPlan {
                delete_all_cookies: true,
                clear_cache: true,
                storage_directories: vec![self.location.data.join(Self::ENGINE_DIRECTORY), self.location.cache.clone()],
                ..BurnPlan::default()
            },
            BurnScope::Site(site) => BurnPlan { cookie_sites: vec![site.to_ascii_lowercase()], ..BurnPlan::default() },
        }
    }
}
