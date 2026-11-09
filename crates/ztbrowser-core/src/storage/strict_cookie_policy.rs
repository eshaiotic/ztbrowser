//! Strict profile: no cookie for a third party, unless the person allowed third parties on the page's site.

use std::sync::Arc;
use std::sync::RwLock;

use crate::network::site_allow_list::SiteAllowList;
use crate::storage::cookie_access::CookieAccess;
use crate::storage::cookie_policy::CookiePolicy;

pub struct StrictCookiePolicy {
    allow_list: Arc<RwLock<SiteAllowList>>,
}

impl StrictCookiePolicy {
    #[must_use]
    pub fn new(allow_list: Arc<RwLock<SiteAllowList>>) -> Self {
        Self { allow_list }
    }
}

impl CookiePolicy for StrictCookiePolicy {
    fn allows(&self, access: &CookieAccess) -> bool {
        if !access.is_third_party() {
            return true;
        }
        access.first_party_host().is_some_and(|page| {
            self.allow_list.read().unwrap_or_else(std::sync::PoisonError::into_inner).allows_third_parties_on(page)
        })
    }
}
