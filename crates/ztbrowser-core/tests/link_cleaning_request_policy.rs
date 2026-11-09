//! Cleaning tracking parameters from the address of a navigation.

use std::sync::Arc;

use serde_json::json;
use ztbrowser_core::network::link_cleaning_request_policy::LinkCleaningRequestPolicy;
use ztbrowser_core::network::network_request::NetworkRequest;
use ztbrowser_core::network::request_decision::RequestDecision;
use ztbrowser_core::network::request_policy::RequestPolicy;
use ztbrowser_core::network::resource_type::ResourceType;
use ztbrowser_core::protection::privacy_configuration::PrivacyConfiguration;

/// Given: the configuration strips `fbclid` and `utm_source`, except on `needs-utm.com`.
struct LinkCleaningScenario {
    policy: LinkCleaningRequestPolicy,
}

impl LinkCleaningScenario {
    fn given_tracking_parameters_protection() -> Self {
        let configuration = PrivacyConfiguration::new(
            json!({"features": {"trackingParameters": {
                "state": "enabled",
                "exceptions": [{"domain": "needs-utm.com"}],
                "settings": {"parameters": ["fbclid", "utm_source"]}
            }}}),
            "1.214.0",
        );
        Self { policy: LinkCleaningRequestPolicy::new(configuration, Arc::default()) }
    }

    fn when_navigating_to(&self, address: &str) -> RequestDecision {
        self.policy.decide(&NetworkRequest::new(address, None, ResourceType::MainFrame, 1))
    }

    fn when_a_script_requests(&self, address: &str) -> RequestDecision {
        self.policy.decide(&NetworkRequest::new(address, Some("https://news.example.org/"), ResourceType::Script, 1))
    }
}

#[test]
fn should_redirect_a_navigation_to_the_address_without_tracking_parameters() {
    let decision = LinkCleaningScenario::given_tracking_parameters_protection()
        .when_navigating_to("https://shop.example.org/item?id=4&fbclid=abc");
    assert_eq!(decision, RequestDecision::Redirect("https://shop.example.org/item?id=4".to_owned()));
}

#[test]
fn should_pass_a_navigation_that_carries_no_tracking_parameter() {
    assert!(
        LinkCleaningScenario::given_tracking_parameters_protection()
            .when_navigating_to("https://shop.example.org/item?id=4")
            .is_pass()
    );
}

#[test]
fn should_pass_a_site_the_configuration_excepts() {
    assert!(
        LinkCleaningScenario::given_tracking_parameters_protection()
            .when_navigating_to("https://needs-utm.com/?utm_source=mail")
            .is_pass()
    );
}

#[test]
fn should_leave_sub_resource_requests_alone() {
    assert!(
        LinkCleaningScenario::given_tracking_parameters_protection()
            .when_a_script_requests("https://cdn.example.org/a.js?fbclid=abc")
            .is_pass()
    );
}
