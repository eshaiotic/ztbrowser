//! Default profile: third-party cookies of known trackers are refused (everything else is allowed).
//!
//! The request policy already blocks requests to these trackers; this catches cookies that a script running
//! on the page sets for a tracker domain without any request.

use std::sync::Arc;

use crate::protection::domain_name::DomainName;
use crate::protection::tracker_data_set::TrackerAction;
use crate::protection::tracker_data_set::TrackerDataSet;
use crate::storage::cookie_access::CookieAccess;
use crate::storage::cookie_policy::CookiePolicy;

pub struct TrackerCookiePolicy {
    data_set: Arc<TrackerDataSet>,
}

impl TrackerCookiePolicy {
    #[must_use]
    pub fn new(data_set: Arc<TrackerDataSet>) -> Self {
        Self { data_set }
    }

    fn is_blocked_tracker(&self, domain: &str) -> bool {
        DomainName::new(domain)
            .self_and_parent_domains()
            .find_map(|candidate| self.data_set.tracker(candidate))
            .is_some_and(|tracker| tracker.default_action == Some(TrackerAction::Block))
    }
}

impl CookiePolicy for TrackerCookiePolicy {
    fn allows(&self, access: &CookieAccess) -> bool {
        !(access.is_third_party() && self.is_blocked_tracker(access.cookie_domain()))
    }
}
