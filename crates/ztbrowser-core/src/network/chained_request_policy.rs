//! Asks several policies in order; the first one that wants to block or redirect decides.

use crate::network::network_request::NetworkRequest;
use crate::network::request_decision::RequestDecision;
use crate::network::request_policy::RequestPolicy;

#[derive(Default)]
pub struct ChainedRequestPolicy {
    policies: Vec<Box<dyn RequestPolicy>>,
}

impl ChainedRequestPolicy {
    #[must_use]
    pub fn new(policies: Vec<Box<dyn RequestPolicy>>) -> Self {
        Self { policies }
    }
}

impl RequestPolicy for ChainedRequestPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        self.policies
            .iter()
            .map(|policy| policy.decide(request))
            .find(|decision| !decision.is_pass())
            .unwrap_or(RequestDecision::Pass)
    }
}
