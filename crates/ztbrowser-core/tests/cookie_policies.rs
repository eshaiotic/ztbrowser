//! Which cookies a profile accepts: strict profile and the default profile's tracker rule.

use std::sync::Arc;
use std::sync::RwLock;

use serde_json::json;
use ztbrowser_core::network::site_allow_list::SiteAllowList;
use ztbrowser_core::protection::tracker_data_set::TrackerDataSet;
use ztbrowser_core::storage::cookie_access::CookieAccess;
use ztbrowser_core::storage::cookie_policy::CookiePolicy;
use ztbrowser_core::storage::strict_cookie_policy::StrictCookiePolicy;
use ztbrowser_core::storage::tracker_cookie_policy::TrackerCookiePolicy;

struct CookieScenario {
    allow_list: Arc<RwLock<SiteAllowList>>,
    strict: StrictCookiePolicy,
    tracker_rule: TrackerCookiePolicy,
}

impl CookieScenario {
    /// Given: `tracker.com` is a blocked tracker and `harmless.com` is a tracker with action `ignore`.
    fn given_profiles() -> Self {
        let data_set = TrackerDataSet::from_json(&json!({
            "trackers": {
                "tracker.com": {"domain": "tracker.com", "default": "block", "rules": []},
                "harmless.com": {"domain": "harmless.com", "default": "ignore", "rules": []}
            },
            "entities": {},
            "cnames": {}
        }));
        let allow_list = Arc::new(RwLock::new(SiteAllowList::default()));
        Self {
            strict: StrictCookiePolicy::new(Arc::clone(&allow_list)),
            tracker_rule: TrackerCookiePolicy::new(Arc::new(data_set)),
            allow_list,
        }
    }

    fn with_third_parties_allowed_on(self, site: &str) -> Self {
        self.allow_list.write().expect("lock").allow_third_parties_on(site);
        self
    }

    fn strict_allows(&self, access: &CookieAccess) -> bool {
        self.strict.allows(access)
    }

    fn tracker_rule_allows(&self, access: &CookieAccess) -> bool {
        self.tracker_rule.allows(access)
    }
}

fn third_party_cookie(domain: &str) -> CookieAccess {
    CookieAccess::new(domain, Some("news.example.org"), true)
}

#[test]
fn should_refuse_a_third_party_cookie_in_the_strict_profile() {
    assert!(!CookieScenario::given_profiles().strict_allows(&third_party_cookie(".ads.example.com")));
}

#[test]
fn should_accept_a_first_party_cookie_in_the_strict_profile() {
    let access = CookieAccess::new("news.example.org", Some("news.example.org"), false);
    assert!(CookieScenario::given_profiles().strict_allows(&access));
}

#[test]
fn should_accept_a_third_party_cookie_on_a_site_the_person_allowed() {
    let scenario = CookieScenario::given_profiles().with_third_parties_allowed_on("example.org");
    assert!(scenario.strict_allows(&third_party_cookie("cdn.other.com")));
}

#[test]
fn should_refuse_a_third_party_cookie_when_the_page_is_unknown() {
    let access = CookieAccess::new("ads.example.com", None, true);
    assert!(!CookieScenario::given_profiles().strict_allows(&access));
}

#[test]
fn should_accept_a_cookie_the_engine_calls_first_party_even_without_a_known_page() {
    let access = CookieAccess::new("example.org", None, false);
    assert!(CookieScenario::given_profiles().strict_allows(&access));
}

#[test]
fn should_refuse_a_third_party_cookie_of_a_blocked_tracker_in_the_default_profile() {
    assert!(!CookieScenario::given_profiles().tracker_rule_allows(&third_party_cookie(".sub.tracker.com")));
}

#[test]
fn should_accept_a_third_party_cookie_of_a_tracker_whose_action_is_ignore() {
    assert!(CookieScenario::given_profiles().tracker_rule_allows(&third_party_cookie("harmless.com")));
}

#[test]
fn should_accept_the_trackers_own_cookie_on_its_own_site() {
    let access = CookieAccess::new("tracker.com", Some("tracker.com"), false);
    assert!(CookieScenario::given_profiles().tracker_rule_allows(&access));
}

#[test]
fn should_accept_third_party_cookies_of_unknown_sites_in_the_default_profile() {
    assert!(CookieScenario::given_profiles().tracker_rule_allows(&third_party_cookie("cdn.other.com")));
}
