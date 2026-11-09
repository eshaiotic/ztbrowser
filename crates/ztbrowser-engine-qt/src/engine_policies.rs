//! The policies Chromium consults, and how their answers become the bridge's plain values.

use std::sync::Arc;
use std::sync::OnceLock;

use ztbrowser_core::network::connection_clock::SystemConnectionClock;
use ztbrowser_core::network::connection_gate::ConnectionGate;
use ztbrowser_core::network::network_request::NetworkRequest;
use ztbrowser_core::network::request_decision::RequestDecision;
use ztbrowser_core::network::request_policy::RequestPolicy;
use ztbrowser_core::storage::cookie_access::CookieAccess;
use ztbrowser_core::storage::cookie_policy::CookiePolicy;

use crate::engine_bridge::ffi::HookDecision;
use crate::resource_type_mapping::ResourceTypeMapping;

/// Chromium has one request pipeline per process, so there is one set of policies per process.
static INSTALLED: OnceLock<EnginePolicies> = OnceLock::new();
/// The gate the process's connection proxy asks; created before Chromium starts, filled in when policies install.
static GATE: OnceLock<Arc<ConnectionGate>> = OnceLock::new();

pub struct EnginePolicies {
    request: Arc<dyn RequestPolicy>,
    cookies: Arc<dyn CookiePolicy>,
    gate: Arc<ConnectionGate>,
}

impl EnginePolicies {
    /// Policies with a connection gate of their own.
    #[must_use]
    pub fn new(request: Arc<dyn RequestPolicy>, cookies: Arc<dyn CookiePolicy>) -> Self {
        Self::with_gate(request, cookies, Arc::new(ConnectionGate::new(Arc::new(SystemConnectionClock::started_now()))))
    }

    /// Policies whose allowed requests are reported to `gate`, which judges hosts by the same request policy.
    #[must_use]
    pub fn with_gate(
        request: Arc<dyn RequestPolicy>,
        cookies: Arc<dyn CookiePolicy>,
        gate: Arc<ConnectionGate>,
    ) -> Self {
        gate.install(Arc::clone(&request));
        Self { request, cookies, gate }
    }

    /// The gate of this process, shared by the connection proxy and the policies installed later.
    #[must_use]
    pub fn process_gate() -> Arc<ConnectionGate> {
        Arc::clone(GATE.get_or_init(|| Arc::new(ConnectionGate::new(Arc::new(SystemConnectionClock::started_now())))))
    }

    /// Makes these the process's policies. Only the first call has an effect; returns whether it was the first.
    pub fn install(self) -> bool {
        INSTALLED.set(self).is_ok()
    }

    /// Until policies are installed nothing passes: a request must never skip protection because it was
    /// made before start-up finished.
    pub(crate) fn installed() -> Option<&'static Self> {
        INSTALLED.get()
    }
}

/// The two questions Chromium asks, answered from a set of policies.
pub struct PolicyDecisions;

impl PolicyDecisions {
    const ACTION_PASS: u8 = 0;
    const ACTION_BLOCK: u8 = 1;
    const ACTION_REDIRECT: u8 = 2;

    #[must_use]
    pub fn for_request(
        policies: Option<&EnginePolicies>,
        url: &str,
        first_party: &str,
        resource_type: i32,
        request_id: u64,
    ) -> HookDecision {
        let Some(policies) = policies else { return Self::decision(Self::ACTION_BLOCK, String::new()) };
        let page = (!first_party.is_empty()).then_some(first_party);
        let request = NetworkRequest::new(url, page, ResourceTypeMapping::from_engine_code(resource_type), request_id);
        match policies.request.decide(&request) {
            RequestDecision::Pass => {
                // The connections this request needs are allowed to follow it.
                if let Some(host) = request.host() {
                    policies.gate.approve(host);
                }
                Self::decision(Self::ACTION_PASS, String::new())
            }
            RequestDecision::Block => Self::decision(Self::ACTION_BLOCK, String::new()),
            RequestDecision::Redirect(address) => Self::decision(Self::ACTION_REDIRECT, address),
        }
    }

    #[must_use]
    pub fn for_cookie(
        policies: Option<&EnginePolicies>,
        domain: &str,
        first_party: &str,
        is_third_party: bool,
    ) -> bool {
        let Some(policies) = policies else { return false };
        let page = (!first_party.is_empty()).then_some(first_party);
        policies.cookies.allows(&CookieAccess::new(domain, page, is_third_party))
    }

    fn decision(action: u8, redirect_to: String) -> HookDecision {
        HookDecision { action, redirect_to }
    }
}
