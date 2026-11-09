//! Blocks third-party requests to advertising and analytics trackers, with no exceptions for particular sites.
//!
//! DuckDuckGo's rules (see `ContentBlockerRequestPolicy`) exempt some tracker URLs on some sites, because
//! blocking them broke something there: a video player that waits for an advertising script, a bot check.
//! That is the right trade for an extension that must never break a page. ZTBrowser's purpose is privacy, so
//! this policy runs after DuckDuckGo's and blocks every tracker that is blocked by default and exists to serve
//! advertising or measure audiences, whatever the page. A person who finds a page broken switches protection
//! off for that site, which this policy, like every other, respects.
//!
//! Trackers that are also something a page may need (an online payment, a login service, a support chat) are
//! left to DuckDuckGo's rules.

use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use crate::network::network_request::NetworkRequest;
use crate::network::request_decision::RequestDecision;
use crate::network::request_policy::RequestPolicy;
use crate::protection::domain_name::DomainName;
use crate::protection::tracker_data_set::TrackerAction;
use crate::protection::tracker_data_set::TrackerDataSet;
use crate::web_app::registrable_domain_lookup::RegistrableDomainLookup;

pub struct AdvertisingTrackerRequestPolicy {
    /// Domain → the domains of the company that owns it (empty when the owner is unknown).
    domains: HashMap<String, HashSet<String>>,
    sites: Arc<dyn RegistrableDomainLookup + Send + Sync>,
}

impl AdvertisingTrackerRequestPolicy {
    /// What a tracker must be for to be blocked here.
    const TARGETED: [&'static str; 8] = [
        "Advertising",
        "Ad Motivated Tracking",
        "Ad Fraud",
        "Analytics",
        "Audience Measurement",
        "Third-Party Analytics Marketing",
        "Action Pixels",
        "Session Replay",
    ];
    /// What makes a tracker something pages need, so that it is left to DuckDuckGo's exceptions.
    const NEEDED_BY_PAGES: [&'static str; 6] =
        ["Online Payment", "Federated Login", "SSO", "Support Chat Widget", "Content Delivery", "Embedded Content"];

    #[must_use]
    pub fn new(data_set: &TrackerDataSet, sites: Arc<dyn RegistrableDomainLookup + Send + Sync>) -> Self {
        let domains = data_set
            .trackers()
            .filter(|tracker| tracker.default_action == Some(TrackerAction::Block))
            .filter(|tracker| Self::has_any(&tracker.categories, &Self::TARGETED))
            .filter(|tracker| !Self::has_any(&tracker.categories, &Self::NEEDED_BY_PAGES))
            .filter_map(|tracker| {
                let owned = data_set
                    .related_domains(tracker.owner_name.as_deref())
                    .map(|owned| owned.iter().cloned().collect())
                    .unwrap_or_default();
                Some((tracker.domain.clone()?, owned))
            })
            .collect();
        Self { domains, sites }
    }

    fn has_any(categories: &[String], wanted: &[&str]) -> bool {
        categories.iter().any(|category| wanted.contains(&category.as_str()))
    }

    fn is_owned_by_page_company(&self, owned: &HashSet<String>, page_host: &str) -> bool {
        let page_site = self.sites.site_of(page_host);
        owned.iter().any(|domain| self.sites.site_of(domain) == page_site)
    }
}

impl RequestPolicy for AdvertisingTrackerRequestPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        let Some(host) = request.host().filter(|_| request.goes_over_the_network() && !request.is_navigation()) else {
            return RequestDecision::Pass;
        };
        if !request.is_third_party(self.sites.as_ref()) {
            return RequestDecision::Pass;
        }
        let tracker = DomainName::new(host).self_and_parent_domains().find_map(|candidate| self.domains.get(candidate));
        match tracker {
            Some(owned)
                if !request.first_party_host().is_some_and(|page| self.is_owned_by_page_company(owned, page)) =>
            {
                RequestDecision::Block
            }
            _ => RequestDecision::Pass,
        }
    }
}
