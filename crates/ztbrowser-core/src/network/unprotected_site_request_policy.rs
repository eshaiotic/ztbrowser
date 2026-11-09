//! Switches every protection off on the sites the person (or DuckDuckGo's exception list) excluded.

use std::sync::Arc;
use std::sync::RwLock;

use crate::network::network_request::NetworkRequest;
use crate::network::request_decision::RequestDecision;
use crate::network::request_policy::RequestPolicy;
use crate::profile::unprotected_sites::UnprotectedSites;

pub struct UnprotectedSiteRequestPolicy {
    inner: Arc<dyn RequestPolicy>,
    unprotected_sites: Arc<RwLock<UnprotectedSites>>,
}

impl UnprotectedSiteRequestPolicy {
    #[must_use]
    pub fn new(inner: Arc<dyn RequestPolicy>, unprotected_sites: Arc<RwLock<UnprotectedSites>>) -> Self {
        Self { inner, unprotected_sites }
    }

    /// A navigation is judged by where it goes; a sub-resource by the page that makes it. A sub-resource of an
    /// unknown page is never exempt.
    fn is_on_unprotected_site(&self, request: &NetworkRequest) -> bool {
        let site = if request.is_navigation() { request.host() } else { request.first_party_host() };
        let sites = self.unprotected_sites.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        site.is_some_and(|host| sites.is_unprotected(host))
    }
}

impl RequestPolicy for UnprotectedSiteRequestPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        if self.is_on_unprotected_site(request) { RequestDecision::Pass } else { self.inner.decide(request) }
    }
}
