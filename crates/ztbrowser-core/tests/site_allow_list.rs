//! Sites where the person allowed third-party requests in the strict profile.

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::network::site_allow_list::SiteAllowList;

struct AllowListScenario {
    list: SiteAllowList,
}

impl AllowListScenario {
    fn given_nothing_stored() -> Self {
        Self { list: SiteAllowList::from_json(&Value::Null) }
    }

    fn when_allowed_on(mut self, site: &str) -> Self {
        self.list.allow_third_parties_on(site);
        self
    }

    fn when_forbidden_again_on(mut self, site: &str) -> Self {
        self.list.forbid_third_parties_on(site);
        self
    }

    fn after_saving_and_loading(self) -> Self {
        Self { list: SiteAllowList::from_json(&self.list.to_json()) }
    }

    fn then_allowed_on(&self, host: &str, expected: bool) {
        assert_eq!(self.list.allows_third_parties_on(host), expected, "{host}");
    }
}

#[test]
fn should_allow_nothing_by_default() {
    AllowListScenario::given_nothing_stored().then_allowed_on("shop.example.org", false);
}

#[test]
fn should_allow_the_site_and_its_subdomains() {
    let scenario = AllowListScenario::given_nothing_stored().when_allowed_on("shop.example.org");
    scenario.then_allowed_on("shop.example.org", true);
    scenario.then_allowed_on("www.shop.example.org", true);
}

#[test]
fn should_not_allow_a_site_that_only_ends_with_the_same_letters() {
    AllowListScenario::given_nothing_stored()
        .when_allowed_on("shop.example.org")
        .then_allowed_on("notshop.example.org", false);
}

#[test]
fn should_remember_the_sites_after_saving_and_loading() {
    AllowListScenario::given_nothing_stored()
        .when_allowed_on("shop.example.org")
        .after_saving_and_loading()
        .then_allowed_on("shop.example.org", true);
}

#[test]
fn should_forget_a_site_the_person_forbids_again() {
    AllowListScenario::given_nothing_stored()
        .when_allowed_on("shop.example.org")
        .when_forbidden_again_on("shop.example.org")
        .then_allowed_on("shop.example.org", false);
}

#[test]
fn should_ignore_stored_entries_that_are_not_text() {
    let list = SiteAllowList::from_json(&json!({"allowedSites": ["shop.example.org", 5, ""]}));
    assert_eq!(list.sites(), vec!["shop.example.org"]);
}
