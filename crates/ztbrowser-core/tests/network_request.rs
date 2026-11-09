//! What a policy can read from one request.

use ztbrowser_core::network::network_request::NetworkRequest;
use ztbrowser_core::network::resource_type::ResourceType;

struct RequestScenario {
    request: NetworkRequest,
}

impl RequestScenario {
    fn given_request(address: &str, page: Option<&str>) -> Self {
        Self { request: NetworkRequest::new(address, page, ResourceType::Script, 7) }
    }

    fn then_host_is(&self, expected: Option<&str>) {
        assert_eq!(self.request.host(), expected);
    }

    fn then_page_host_is(&self, expected: Option<&str>) {
        assert_eq!(self.request.first_party_host(), expected);
    }

    fn then_goes_over_the_network(&self, expected: bool) {
        assert_eq!(self.request.goes_over_the_network(), expected);
    }
}

#[test]
fn should_read_the_lower_case_host_of_the_request_and_of_the_page() {
    let scenario =
        RequestScenario::given_request("https://CDN.Tracker.com/a.js", Some("https://News.Example.org/story"));
    scenario.then_host_is(Some("cdn.tracker.com"));
    scenario.then_page_host_is(Some("news.example.org"));
}

#[test]
fn should_have_no_page_host_for_an_address_bar_navigation() {
    RequestScenario::given_request("https://example.org/", None).then_page_host_is(None);
}

#[test]
fn should_have_no_host_for_an_address_that_does_not_parse() {
    RequestScenario::given_request("not a url", None).then_host_is(None);
}

#[test]
fn should_treat_web_socket_requests_as_network_requests() {
    RequestScenario::given_request("wss://live.example.org/socket", None).then_goes_over_the_network(true);
}

#[test]
fn should_not_treat_data_and_blob_requests_as_network_requests() {
    RequestScenario::given_request("data:text/plain,hi", None).then_goes_over_the_network(false);
    RequestScenario::given_request("blob:https://example.org/uuid", None).then_goes_over_the_network(false);
}

#[test]
fn should_read_the_creating_site_of_a_blob_page() {
    RequestScenario::given_request("https://cdn.example.org/a.js", Some("blob:https://News.Example.org/6f1c"))
        .then_page_host_is(Some("news.example.org"));
}

#[test]
fn should_have_no_page_host_for_a_page_without_a_host() {
    RequestScenario::given_request("https://cdn.example.org/a.js", Some("data:text/html,x")).then_page_host_is(None);
}
