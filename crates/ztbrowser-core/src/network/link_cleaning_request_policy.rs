//! Removes tracking parameters (`fbclid`, `utm_source`, ...) from the address a page is navigating to.

use std::sync::Arc;
use std::sync::RwLock;

use crate::network::network_request::NetworkRequest;
use crate::network::request_decision::RequestDecision;
use crate::network::request_policy::RequestPolicy;
use crate::profile::unprotected_sites::UnprotectedSites;
use crate::protection::privacy_configuration::PrivacyConfiguration;
use crate::protection::tracking_parameter_remover::TrackingParameterRemover;

pub struct LinkCleaningRequestPolicy {
    configuration: PrivacyConfiguration,
    unprotected_sites: Arc<RwLock<UnprotectedSites>>,
}

impl LinkCleaningRequestPolicy {
    #[must_use]
    /// The list is shared with the code that edits it, so a site switched off takes effect on the next navigation.
    pub fn new(configuration: PrivacyConfiguration, unprotected_sites: Arc<RwLock<UnprotectedSites>>) -> Self {
        Self { configuration, unprotected_sites }
    }
}

impl RequestPolicy for LinkCleaningRequestPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        if !request.is_navigation() || !request.goes_over_the_network() {
            return RequestDecision::Pass;
        }
        let unprotected: Vec<String> = self
            .unprotected_sites
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .domains()
            .into_iter()
            .map(str::to_owned)
            .collect();
        let remover = TrackingParameterRemover::new(&self.configuration, &unprotected);
        let cleaned = remover.cleaned(request.address(), None);
        if cleaned == request.address() { RequestDecision::Pass } else { RequestDecision::Redirect(cleaned) }
    }
}
