//! Upgrading plain-HTTP requests, once per request.

use std::sync::Arc;
use std::sync::RwLock;

use ztbrowser_core::navigation::https_upgrade_policy::HttpsUpgradePolicy;
use ztbrowser_core::network::https_upgrade_request_policy::HttpsUpgradeRequestPolicy;
use ztbrowser_core::network::network_request::NetworkRequest;
use ztbrowser_core::network::request_decision::RequestDecision;
use ztbrowser_core::network::request_policy::RequestPolicy;
use ztbrowser_core::network::resource_type::ResourceType;

struct UpgradeScenario {
    upgrades: Arc<RwLock<HttpsUpgradePolicy>>,
    policy: HttpsUpgradeRequestPolicy,
}

impl UpgradeScenario {
    fn given_https_first_browsing() -> Self {
        let upgrades = Arc::new(RwLock::new(HttpsUpgradePolicy::new(&["legacy-portal.com".to_owned()])));
        Self { policy: HttpsUpgradeRequestPolicy::new(Arc::clone(&upgrades)), upgrades }
    }

    fn after_https_failed_for(self, address: &str) -> Self {
        self.upgrades.write().expect("lock").record_failed_upgrade(address);
        self
    }

    fn when_requesting(&self, address: &str) -> RequestDecision {
        self.policy.decide(&NetworkRequest::new(address, None, ResourceType::MainFrame, 1))
    }
}

#[test]
fn should_redirect_a_plain_http_request_to_https() {
    let decision = UpgradeScenario::given_https_first_browsing().when_requesting("http://news.example.org/story?a=1");
    assert_eq!(decision, RequestDecision::Redirect("https://news.example.org/story?a=1".to_owned()));
}

#[test]
fn should_pass_a_request_that_is_already_https() {
    assert!(UpgradeScenario::given_https_first_browsing().when_requesting("https://news.example.org/").is_pass());
}

#[test]
fn should_pass_a_site_the_configuration_excludes() {
    assert!(UpgradeScenario::given_https_first_browsing().when_requesting("http://legacy-portal.com/").is_pass());
}

#[test]
fn should_pass_a_site_whose_https_attempt_failed() {
    let scenario = UpgradeScenario::given_https_first_browsing().after_https_failed_for("http://old.example.org/");
    assert!(scenario.when_requesting("http://old.example.org/").is_pass());
}

#[test]
fn should_pass_a_home_network_address() {
    assert!(UpgradeScenario::given_https_first_browsing().when_requesting("http://nas.local/").is_pass());
}
