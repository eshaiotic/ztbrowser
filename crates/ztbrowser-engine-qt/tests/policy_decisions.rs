//! How Chromium's questions are answered from the core's policies, without starting Chromium.

use std::sync::Arc;

use ztbrowser_core::network::connection_clock::SystemConnectionClock;
use ztbrowser_core::network::connection_gate::ConnectionGate;
use ztbrowser_core::network::network_request::NetworkRequest;
use ztbrowser_core::network::request_decision::RequestDecision;
use ztbrowser_core::network::request_policy::RequestPolicy;
use ztbrowser_core::storage::cookie_access::CookieAccess;
use ztbrowser_core::storage::cookie_policy::CookiePolicy;
use ztbrowser_engine_qt::engine_bridge::ffi::HookDecision;
use ztbrowser_engine_qt::engine_policies::EnginePolicies;
use ztbrowser_engine_qt::engine_policies::PolicyDecisions;

/// Blocks scripts, redirects main-frame navigations to `https://safe.test/`, passes the rest.
struct ScriptBlockingPolicy;

impl RequestPolicy for ScriptBlockingPolicy {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        match request.resource_type() {
            ztbrowser_core::network::resource_type::ResourceType::Script => RequestDecision::Block,
            ztbrowser_core::network::resource_type::ResourceType::MainFrame => {
                RequestDecision::Redirect("https://safe.test/".to_owned())
            }
            _ => RequestDecision::Pass,
        }
    }
}

/// Refuses third-party cookies.
struct FirstPartyCookiesOnly;

impl CookiePolicy for FirstPartyCookiesOnly {
    fn allows(&self, access: &CookieAccess) -> bool {
        !access.is_third_party()
    }
}

struct DecisionScenario {
    policies: EnginePolicies,
    gate: Arc<ConnectionGate>,
}

impl DecisionScenario {
    fn given_installed_policies() -> Self {
        let gate = Arc::new(ConnectionGate::new(Arc::new(SystemConnectionClock::started_now())));
        let policies = EnginePolicies::with_gate(
            Arc::new(ScriptBlockingPolicy),
            Arc::new(FirstPartyCookiesOnly),
            Arc::clone(&gate),
        );
        Self { policies, gate }
    }

    fn when_chromium_asks_about_request(&self, resource_type: i32) -> HookDecision {
        PolicyDecisions::for_request(
            Some(&self.policies),
            "https://cdn.example.org/a",
            "https://news.example.org/",
            resource_type,
            1,
        )
    }

    fn when_chromium_asks_about_cookie(&self, is_third_party: bool) -> bool {
        PolicyDecisions::for_cookie(Some(&self.policies), "ads.example.com", "news.example.org", is_third_party)
    }
}

#[test]
fn should_tell_chromium_to_block_a_script() {
    let decision = DecisionScenario::given_installed_policies().when_chromium_asks_about_request(3);
    assert_eq!(decision.action, 1);
}

#[test]
fn should_tell_chromium_to_redirect_a_navigation_to_the_policys_address() {
    let decision = DecisionScenario::given_installed_policies().when_chromium_asks_about_request(0);
    assert_eq!((decision.action, decision.redirect_to.as_str()), (2, "https://safe.test/"));
}

#[test]
fn should_let_an_image_pass() {
    let decision = DecisionScenario::given_installed_policies().when_chromium_asks_about_request(4);
    assert_eq!(decision.action, 0);
}

#[test]
fn should_treat_an_unknown_resource_type_as_other_and_not_as_a_script_or_navigation() {
    let decision = DecisionScenario::given_installed_policies().when_chromium_asks_about_request(255);
    assert_eq!(decision.action, 0);
}

#[test]
fn should_refuse_a_third_party_cookie_and_accept_a_first_party_one() {
    let scenario = DecisionScenario::given_installed_policies();
    assert!(!scenario.when_chromium_asks_about_cookie(true));
    assert!(scenario.when_chromium_asks_about_cookie(false));
}

#[test]
fn should_block_requests_made_before_policies_are_installed() {
    let decision = PolicyDecisions::for_request(None, "https://cdn.example.org/a", "", 4, 1);
    assert_eq!(decision.action, 1);
}

#[test]
fn should_refuse_cookies_before_policies_are_installed() {
    assert!(!PolicyDecisions::for_cookie(None, "example.org", "example.org", false));
}

#[test]
fn should_approve_the_connection_of_a_request_that_passed() {
    let scenario = DecisionScenario::given_installed_policies();
    scenario.when_chromium_asks_about_request(4);
    assert_eq!(scenario.gate.approval_count(), 1);
}

#[test]
fn should_not_approve_the_connection_of_a_request_that_was_blocked() {
    let scenario = DecisionScenario::given_installed_policies();
    scenario.when_chromium_asks_about_request(3);
    assert_eq!(scenario.gate.approval_count(), 0);
}
