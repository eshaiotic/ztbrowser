//! Blocks the requests ZTBrowser Guard's filter lists block, on a site's own requests as well as third parties'.

use std::sync::Arc;

use crate::guard::shared_guard_engine::SharedGuardEngine;
use crate::network::network_request::NetworkRequest;
use crate::network::request_decision::RequestDecision;
use crate::network::request_policy::RequestPolicy;

pub struct GuardRequestPolicy {
    engine: Arc<SharedGuardEngine>,
}

impl GuardRequestPolicy {
    #[must_use]
    pub fn new(engine: Arc<SharedGuardEngine>) -> Self {
        Self { engine }
    }
}

impl RequestPolicy for GuardRequestPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        let Some(engine) = self.engine.get() else { return RequestDecision::Pass };
        if !request.goes_over_the_network() || request.is_navigation() {
            return RequestDecision::Pass;
        }
        let page = request.first_party_host().map_or_else(String::new, |host| format!("https://{host}/"));
        if engine.blocks(request.address(), &page, request.resource_type()) {
            RequestDecision::Block
        } else {
            RequestDecision::Pass
        }
    }
}
