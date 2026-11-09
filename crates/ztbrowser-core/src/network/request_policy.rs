//! A rule about requests that the engine adapter asks for every request.

use crate::network::network_request::NetworkRequest;
use crate::network::request_decision::RequestDecision;

/// `Send + Sync` because engines call policies from a network thread while the interface thread changes
/// settings.
pub trait RequestPolicy: Send + Sync {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision;
}
