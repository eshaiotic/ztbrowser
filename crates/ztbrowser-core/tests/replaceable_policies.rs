//! Protection data that arrives while the browser runs replaces the rules in force, with no restart.

use std::sync::Arc;

use ztbrowser_core::guard::guard_engine::GuardEngine;
use ztbrowser_core::guard::shared_guard_engine::SharedGuardEngine;
use ztbrowser_core::network::network_request::NetworkRequest;
use ztbrowser_core::network::replaceable_request_policy::ReplaceableRequestPolicy;
use ztbrowser_core::network::request_decision::RequestDecision;
use ztbrowser_core::network::request_policy::RequestPolicy;
use ztbrowser_core::network::resource_type::ResourceType;
use ztbrowser_core::storage::cookie_access::CookieAccess;
use ztbrowser_core::storage::cookie_policy::CookiePolicy;
use ztbrowser_core::storage::replaceable_cookie_policy::ReplaceableCookiePolicy;

struct Always(RequestDecision);

impl RequestPolicy for Always {
    fn decide(&self, _request: &NetworkRequest) -> RequestDecision {
        self.0.clone()
    }
}

struct Refuses(bool);

impl CookiePolicy for Refuses {
    fn allows(&self, _access: &CookieAccess) -> bool {
        !self.0
    }
}

fn request() -> NetworkRequest {
    NetworkRequest::new("https://tracker.example/p.gif", Some("https://news.example/"), ResourceType::Image, 1)
}

#[test]
fn should_decide_with_the_policy_it_was_given_first() {
    let policy = ReplaceableRequestPolicy::new(Arc::new(Always(RequestDecision::Pass)));
    assert_eq!(policy.decide(&request()), RequestDecision::Pass);
}

#[test]
fn should_decide_with_the_policy_that_replaced_it() {
    let policy = ReplaceableRequestPolicy::new(Arc::new(Always(RequestDecision::Pass)));
    policy.replace(Arc::new(Always(RequestDecision::Block)));
    assert_eq!(policy.decide(&request()), RequestDecision::Block);
}

#[test]
fn should_answer_cookie_questions_with_the_policy_that_replaced_the_first() {
    let policy = ReplaceableCookiePolicy::new(Arc::new(Refuses(false)));
    let access = CookieAccess::new("tracker.example", Some("news.example"), true);
    assert!(policy.allows(&access));
    policy.replace(Arc::new(Refuses(true)));
    assert!(!policy.allows(&access));
}

#[test]
fn should_filter_with_the_guard_engine_installed_last() {
    let shared = SharedGuardEngine::default();
    assert!(shared.get().is_none());
    shared.install(GuardEngine::from_lists(&["[Adblock Plus 2.0]\n||first.example^"]));
    shared.install(GuardEngine::from_lists(&["[Adblock Plus 2.0]\n||second.example^"]));
    let engine = shared.get().unwrap();
    assert!(!engine.blocks("https://first.example/a.js", "https://news.example/", ResourceType::Script));
    assert!(engine.blocks("https://second.example/a.js", "https://news.example/", ResourceType::Script));
}
