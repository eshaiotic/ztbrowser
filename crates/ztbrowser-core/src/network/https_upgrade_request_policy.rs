//! Sends plain-HTTP requests over HTTPS where the site is known to support it.

use std::sync::Arc;
use std::sync::RwLock;

use crate::navigation::https_upgrade_policy::HttpsUpgradePolicy;
use crate::network::network_request::NetworkRequest;
use crate::network::request_decision::RequestDecision;
use crate::network::request_policy::RequestPolicy;

pub struct HttpsUpgradeRequestPolicy {
    upgrades: Arc<RwLock<HttpsUpgradePolicy>>,
}

impl HttpsUpgradeRequestPolicy {
    /// The upgrade policy is shared with the code that records sites whose HTTPS attempt failed.
    #[must_use]
    pub fn new(upgrades: Arc<RwLock<HttpsUpgradePolicy>>) -> Self {
        Self { upgrades }
    }
}

impl RequestPolicy for HttpsUpgradeRequestPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        if !request.is_plain_http() {
            return RequestDecision::Pass;
        }
        let upgraded =
            self.upgrades.read().unwrap_or_else(std::sync::PoisonError::into_inner).upgraded(request.address());
        upgraded.map_or(RequestDecision::Pass, RequestDecision::Redirect)
    }
}
