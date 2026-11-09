//! Guard's lists block requests only once the engine exists, and never a navigation.

use std::sync::Arc;

use ztbrowser_core::guard::guard_engine::GuardEngine;
use ztbrowser_core::guard::shared_guard_engine::SharedGuardEngine;
use ztbrowser_core::network::guard_request_policy::GuardRequestPolicy;
use ztbrowser_core::network::network_request::NetworkRequest;
use ztbrowser_core::network::request_decision::RequestDecision;
use ztbrowser_core::network::request_policy::RequestPolicy;
use ztbrowser_core::network::resource_type::ResourceType;

struct GuardPolicyScenario {
    engine: Arc<SharedGuardEngine>,
    policy: GuardRequestPolicy,
}

impl GuardPolicyScenario {
    fn given_an_engine_not_yet_built() -> Self {
        let engine = Arc::new(SharedGuardEngine::default());
        Self { policy: GuardRequestPolicy::new(Arc::clone(&engine)), engine }
    }

    fn when_the_engine_is_built_from(self, list: &str) -> Self {
        self.engine.install(GuardEngine::from_lists(&[list]));
        self
    }

    fn decision_for(&self, address: &str, resource: ResourceType) -> RequestDecision {
        self.policy.decide(&NetworkRequest::new(address, Some("https://news.test/"), resource, 1))
    }
}

#[test]
fn should_let_everything_through_until_the_engine_is_built() {
    let scenario = GuardPolicyScenario::given_an_engine_not_yet_built();
    assert!(scenario.decision_for("https://ads.example/a.png", ResourceType::Image).is_pass());
}

#[test]
fn should_block_a_listed_request_once_the_engine_is_built() {
    let scenario = GuardPolicyScenario::given_an_engine_not_yet_built().when_the_engine_is_built_from("||ads.example^");
    assert_eq!(scenario.decision_for("https://ads.example/a.png", ResourceType::Image), RequestDecision::Block);
}

#[test]
fn should_never_block_a_navigation() {
    let scenario = GuardPolicyScenario::given_an_engine_not_yet_built().when_the_engine_is_built_from("||ads.example^");
    assert!(scenario.decision_for("https://ads.example/", ResourceType::MainFrame).is_pass());
}

#[test]
fn should_ignore_requests_that_do_not_leave_the_device() {
    let scenario = GuardPolicyScenario::given_an_engine_not_yet_built().when_the_engine_is_built_from("||ads.example^");
    assert!(scenario.decision_for("data:text/plain,ads.example", ResourceType::Image).is_pass());
}
