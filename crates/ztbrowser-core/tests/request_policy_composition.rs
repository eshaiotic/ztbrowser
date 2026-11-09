//! Combining policies: first decision wins, unprotected sites are left alone.

use std::sync::Arc;
use std::sync::RwLock;

use ztbrowser_core::network::chained_request_policy::ChainedRequestPolicy;
use ztbrowser_core::network::network_request::NetworkRequest;
use ztbrowser_core::network::request_decision::RequestDecision;
use ztbrowser_core::network::request_policy::RequestPolicy;
use ztbrowser_core::network::resource_type::ResourceType;
use ztbrowser_core::network::unprotected_site_request_policy::UnprotectedSiteRequestPolicy;
use ztbrowser_core::profile::unprotected_sites::UnprotectedSites;

/// A policy that always gives the same answer, standing in for a real protection.
struct FixedDecision(RequestDecision);

impl RequestPolicy for FixedDecision {
    fn decide(&self, _request: &NetworkRequest) -> RequestDecision {
        self.0.clone()
    }
}

fn request(address: &str, page: Option<&str>, request_id: u64) -> NetworkRequest {
    NetworkRequest::new(address, page, ResourceType::Script, request_id)
}

fn navigation(address: &str, from_page: Option<&str>) -> NetworkRequest {
    NetworkRequest::new(address, from_page, ResourceType::MainFrame, 1)
}

#[test]
fn should_use_the_first_policy_that_does_not_pass() {
    let chain = ChainedRequestPolicy::new(vec![
        Box::new(FixedDecision(RequestDecision::Pass)),
        Box::new(FixedDecision(RequestDecision::Block)),
        Box::new(FixedDecision(RequestDecision::Redirect("https://x.test/".to_owned()))),
    ]);
    assert_eq!(chain.decide(&request("https://a.test/", None, 1)), RequestDecision::Block);
}

#[test]
fn should_pass_when_every_policy_passes_and_when_there_are_none() {
    assert!(ChainedRequestPolicy::default().decide(&request("https://a.test/", None, 1)).is_pass());
}

fn unprotected_policy(site: &str) -> UnprotectedSiteRequestPolicy {
    let mut sites = UnprotectedSites::default();
    sites.switch_protection_off(site);
    UnprotectedSiteRequestPolicy::new(Arc::new(FixedDecision(RequestDecision::Block)), Arc::new(RwLock::new(sites)))
}

#[test]
fn should_pass_every_request_made_from_an_unprotected_site() {
    let policy = unprotected_policy("shop.example.org");
    assert!(policy.decide(&request("https://tracker.com/a.js", Some("https://www.shop.example.org/"), 1)).is_pass());
}

#[test]
fn should_pass_a_navigation_to_an_unprotected_site() {
    let policy = unprotected_policy("shop.example.org");
    assert!(policy.decide(&navigation("https://shop.example.org/", None)).is_pass());
}

#[test]
fn should_keep_protecting_other_sites() {
    let policy = unprotected_policy("shop.example.org");
    assert_eq!(
        policy.decide(&request("https://tracker.com/a.js", Some("https://news.example.org/"), 1)),
        RequestDecision::Block
    );
}

#[test]
fn should_protect_a_navigation_from_an_unprotected_page_to_another_site() {
    let policy = unprotected_policy("shop.example.org");
    assert_eq!(
        policy.decide(&navigation("https://tracker.com/", Some("https://shop.example.org/"))),
        RequestDecision::Block
    );
}

#[test]
fn should_protect_a_sub_resource_of_an_unknown_page() {
    let policy = unprotected_policy("shop.example.org");
    assert_eq!(policy.decide(&request("https://shop.example.org/a.js", None, 1)), RequestDecision::Block);
}
