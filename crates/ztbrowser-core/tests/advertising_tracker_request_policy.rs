//! ZTBrowser blocks advertising and analytics trackers where DuckDuckGo's own exceptions would let them through.
//!
//! DuckDuckGo exempts some tracker URLs on some sites because blocking them broke something there. Those
//! exceptions are right for an extension that must never break a page, and too lenient for a browser whose
//! reason to exist is privacy: a person who finds a page broken can switch protection off for that site.

use std::sync::Arc;

use serde_json::json;
use ztbrowser_core::network::advertising_tracker_request_policy::AdvertisingTrackerRequestPolicy;
use ztbrowser_core::network::network_request::NetworkRequest;
use ztbrowser_core::network::request_decision::RequestDecision;
use ztbrowser_core::network::request_policy::RequestPolicy;
use ztbrowser_core::network::resource_type::ResourceType;
use ztbrowser_core::protection::tracker_data_set::TrackerDataSet;
use ztbrowser_core::web_app::registrable_domain_lookup::RegistrableDomainLookup;

struct KnownSuffixes;

impl RegistrableDomainLookup for KnownSuffixes {
    fn is_public_suffix(&self, domain: &str) -> bool {
        matches!(domain, "com" | "net" | "org")
    }
}

/// Given: `adtech.com` (Adtech Inc, also owning `adtech-sites.net`) and `metrics.com` are blocked by default and
/// are advertising or analytics; `payments.com` is blocked by default but is an online payment;
/// `softads.com` is advertising whose default is `ignore`.
struct AdvertisingScenario {
    policy: AdvertisingTrackerRequestPolicy,
}

impl AdvertisingScenario {
    fn given_published_data() -> Self {
        let data_set = TrackerDataSet::from_json(&json!({
            "trackers": {
                "adtech.com": {"domain": "adtech.com", "owner": {"name": "Adtech Inc"}, "default": "block",
                    "categories": ["Advertising", "Ad Motivated Tracking"], "rules": []},
                "metrics.com": {"domain": "metrics.com", "owner": {"name": "Metrics"}, "default": "block",
                    "categories": ["Analytics", "Audience Measurement"], "rules": []},
                "payments.com": {"domain": "payments.com", "owner": {"name": "Payments"}, "default": "block",
                    "categories": ["Advertising", "Online Payment"], "rules": []},
                "softads.com": {"domain": "softads.com", "owner": {"name": "Softads"}, "default": "ignore",
                    "categories": ["Advertising"], "rules": []},
                "uncategorised.com": {"domain": "uncategorised.com", "owner": {"name": "Unknown"}, "default": "block",
                    "categories": [], "rules": []}
            },
            "entities": {"Adtech Inc": {"domains": ["adtech.com", "adtech-sites.net"]}},
            "cnames": {}
        }));
        Self { policy: AdvertisingTrackerRequestPolicy::new(&data_set, Arc::new(KnownSuffixes)) }
    }

    fn when_page_requests(&self, page: Option<&str>, address: &str) -> RequestDecision {
        self.policy.decide(&NetworkRequest::new(address, page, ResourceType::Script, 1))
    }
}

const NEWS: &str = "https://news.example.org/story";

#[test]
fn should_block_a_third_party_advertising_tracker() {
    let decision =
        AdvertisingScenario::given_published_data().when_page_requests(Some(NEWS), "https://adtech.com/ad.js");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_block_a_third_party_analytics_tracker_and_its_subdomains() {
    let decision =
        AdvertisingScenario::given_published_data().when_page_requests(Some(NEWS), "https://cdn.eu.metrics.com/m.js");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_pass_a_tracker_that_is_also_an_online_payment() {
    let decision =
        AdvertisingScenario::given_published_data().when_page_requests(Some(NEWS), "https://payments.com/pay.js");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_pass_a_domain_whose_default_is_ignore() {
    let decision =
        AdvertisingScenario::given_published_data().when_page_requests(Some(NEWS), "https://softads.com/a.js");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_pass_a_tracker_with_no_advertising_or_analytics_category() {
    let decision =
        AdvertisingScenario::given_published_data().when_page_requests(Some(NEWS), "https://uncategorised.com/x.js");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_pass_the_company_on_its_own_sites() {
    let decision = AdvertisingScenario::given_published_data()
        .when_page_requests(Some("https://www.adtech-sites.net/"), "https://adtech.com/ad.js");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_pass_a_tracker_on_its_own_site() {
    let decision = AdvertisingScenario::given_published_data()
        .when_page_requests(Some("https://www.metrics.com/"), "https://cdn.metrics.com/m.js");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_block_a_tracker_asked_for_by_a_page_it_does_not_know() {
    let decision = AdvertisingScenario::given_published_data().when_page_requests(None, "https://adtech.com/ad.js");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_leave_a_navigation_to_a_tracker_alone() {
    let policy = AdvertisingScenario::given_published_data().policy;
    let decision = policy.decide(&NetworkRequest::new("https://adtech.com/", None, ResourceType::MainFrame, 1));
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_not_block_a_look_alike_domain() {
    let decision =
        AdvertisingScenario::given_published_data().when_page_requests(Some(NEWS), "https://notadtech.com/ad.js");
    assert_eq!(decision, RequestDecision::Pass);
}
