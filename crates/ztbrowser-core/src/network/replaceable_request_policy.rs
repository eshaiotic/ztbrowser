//! A request policy that can be swapped for another while requests keep being decided.
//!
//! Protection data is downloaded after the browser starts and refreshed while it runs. The engine holds one
//! policy for its whole life, so the policy it holds is this one, and the rules inside it change.

use std::sync::Arc;
use std::sync::PoisonError;
use std::sync::RwLock;

use crate::network::network_request::NetworkRequest;
use crate::network::request_decision::RequestDecision;
use crate::network::request_policy::RequestPolicy;

pub struct ReplaceableRequestPolicy {
    current: RwLock<Arc<dyn RequestPolicy>>,
}

impl ReplaceableRequestPolicy {
    #[must_use]
    pub fn new(initial: Arc<dyn RequestPolicy>) -> Self {
        Self { current: RwLock::new(initial) }
    }

    /// Requests decided from now on are decided by `policy`.
    pub fn replace(&self, policy: Arc<dyn RequestPolicy>) {
        *self.current.write().unwrap_or_else(PoisonError::into_inner) = policy;
    }
}

impl RequestPolicy for ReplaceableRequestPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        // The lock is held only to copy the pointer, never while the policy decides.
        let policy = Arc::clone(&self.current.read().unwrap_or_else(PoisonError::into_inner));
        policy.decide(request)
    }
}
