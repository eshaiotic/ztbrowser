//! Decides whether the engine may open a connection to a host, for connections no request announced.
//!
//! A page can make the engine connect to a host without making a request: `<link rel="preconnect">` and
//! `dns-prefetch` hints in its HTML are read before any script runs, and the connection (with the visitor's
//! address and the host's name in the TLS handshake) happens before any request filter is asked. Measured on
//! bbc.com and edition.cnn.com, this contacted every advertising host the filters block.
//!
//! The gate closes that gap at the one place such a connection has to pass. A host is admitted when the
//! request policies would not block it, or when a request to it was allowed a moment ago: that keeps
//! DuckDuckGo's per-site exceptions and the switch that turns protection off for a site working, because both
//! act on requests, and a connection follows the request they allowed.
//!
//! The common case costs one lookup in a hash table: most hosts are never blocked, so the policies run only
//! for hosts that have no recent approval.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::PoisonError;
use std::sync::RwLock;
use std::time::Duration;

use crate::network::connection_clock::ConnectionClock;
use crate::network::network_request::NetworkRequest;
use crate::network::request_decision::RequestDecision;
use crate::network::request_policy::RequestPolicy;
use crate::network::resource_type::ResourceType;

pub struct ConnectionGate {
    clock: Arc<dyn ConnectionClock>,
    policy: RwLock<Option<Arc<dyn RequestPolicy>>>,
    /// Host (lower case) → the moment a request to it was last allowed.
    approvals: Mutex<HashMap<String, Duration>>,
}

impl ConnectionGate {
    /// How long a connection may follow an allowed request. A page that was allowed to load a host opens its
    /// connections within moments; a minute also covers a slow page.
    pub const APPROVAL_LIFETIME: Duration = Duration::from_secs(60);
    /// Approvals are swept out once there are this many, so a long session does not collect every host it met.
    pub const PRUNE_THRESHOLD: usize = 2048;
    /// A page nobody visits: the filters judge a host by its own name, as a third party's.
    const NEUTRAL_PAGE: &'static str = "https://ztbrowser.invalid/";

    #[must_use]
    pub fn new(clock: Arc<dyn ConnectionClock>) -> Self {
        Self { clock, policy: RwLock::new(None), approvals: Mutex::new(HashMap::new()) }
    }

    /// Makes `policy` the one that judges hosts, replacing the previous one. Until the first call nothing is
    /// admitted: a connection must never skip protection because it was made before start-up finished.
    pub fn install(&self, policy: Arc<dyn RequestPolicy>) {
        *self.policy.write().unwrap_or_else(PoisonError::into_inner) = Some(policy);
    }

    /// Records that a request to `host` was allowed, so the connections it needs are admitted.
    pub fn approve(&self, host: &str) {
        let now = self.clock.elapsed();
        let mut approvals = self.approvals.lock().unwrap_or_else(PoisonError::into_inner);
        if approvals.len() >= Self::PRUNE_THRESHOLD {
            approvals.retain(|_, approved| now.saturating_sub(*approved) <= Self::APPROVAL_LIFETIME);
        }
        match approvals.get_mut(host) {
            Some(approved) => *approved = now,
            None => {
                approvals.insert(host.to_ascii_lowercase(), now);
            }
        }
    }

    /// Whether the engine may connect to `host`.
    #[must_use]
    pub fn admits(&self, host: &str) -> bool {
        let Some(policy) = self.policy.read().unwrap_or_else(PoisonError::into_inner).clone() else { return false };
        let host = host.to_ascii_lowercase();
        self.is_approved(&host) || !Self::blocks(policy.as_ref(), &host)
    }

    /// How many approvals are remembered, for the tests that check the memory stays bounded.
    #[must_use]
    pub fn approval_count(&self) -> usize {
        self.approvals.lock().unwrap_or_else(PoisonError::into_inner).len()
    }

    fn is_approved(&self, host: &str) -> bool {
        let now = self.clock.elapsed();
        self.approvals
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(host)
            .is_some_and(|approved| now.saturating_sub(*approved) <= Self::APPROVAL_LIFETIME)
    }

    fn blocks(policy: &dyn RequestPolicy, host: &str) -> bool {
        let request =
            NetworkRequest::new(&format!("https://{host}/"), Some(Self::NEUTRAL_PAGE), ResourceType::Other, 0);
        matches!(policy.decide(&request), RequestDecision::Block)
    }
}
