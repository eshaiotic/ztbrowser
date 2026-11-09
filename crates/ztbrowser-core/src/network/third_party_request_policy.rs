//! Strict mode: requests to another site than the page's own are blocked unless the person allowed them.

use std::sync::Arc;
use std::sync::RwLock;

use crate::network::network_request::NetworkRequest;
use crate::network::request_decision::RequestDecision;
use crate::network::request_policy::RequestPolicy;
use crate::network::site_allow_list::SiteAllowList;
use crate::web_app::registrable_domain_lookup::RegistrableDomainLookup;

pub struct ThirdPartyRequestPolicy {
    is_strict: bool,
    allow_list: Arc<RwLock<SiteAllowList>>,
    sites: Arc<dyn RegistrableDomainLookup + Send + Sync>,
}

impl ThirdPartyRequestPolicy {
    /// The allow list is shared with the interface, which edits it while requests are being decided.
    #[must_use]
    pub fn new(
        is_strict: bool,
        allow_list: Arc<RwLock<SiteAllowList>>,
        sites: Arc<dyn RegistrableDomainLookup + Send + Sync>,
    ) -> Self {
        Self { is_strict, allow_list, sites }
    }
}

impl RequestPolicy for ThirdPartyRequestPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        if !self.is_strict || !request.goes_over_the_network() || !request.is_third_party(self.sites.as_ref()) {
            return RequestDecision::Pass;
        }
        let page_is_allowed = request.first_party_host().is_some_and(|page| {
            self.allow_list.read().unwrap_or_else(std::sync::PoisonError::into_inner).allows_third_parties_on(page)
        });
        if page_is_allowed { RequestDecision::Pass } else { RequestDecision::Block }
    }
}
