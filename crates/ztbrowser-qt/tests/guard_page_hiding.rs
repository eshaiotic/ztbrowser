//! A page learns what Guard hides on it, unless protection is off for its site, and cannot overload the answer.

use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;
use ztbrowser_core::guard::guard_engine::GuardEngine;
use ztbrowser_core::guard::shared_guard_engine::SharedGuardEngine;
use ztbrowser_qt::guard_page_hiding::GuardPageHiding;
use ztbrowser_qt::site_protection::SiteProtection;

use ztbrowser_qt as _;

const LIST: &str = "shop.test##.cookie-banner\n##.sponsored\n";

struct PageScenario {
    guard: SharedGuardEngine,
    protection: SiteProtection,
    _profile: TempDir,
}

impl PageScenario {
    fn given_a_built_engine() -> Self {
        let profile = TempDir::new().expect("profile");
        let guard = SharedGuardEngine::default();
        guard.install(GuardEngine::from_lists(&[LIST]));
        Self { guard, protection: SiteProtection::open(profile.path()), _profile: profile }
    }

    fn hiding(&self) -> GuardPageHiding<'_> {
        GuardPageHiding::new(&self.guard, &self.protection)
    }
}

#[test]
fn should_name_what_the_lists_hide_on_a_protected_page() {
    let answer = PageScenario::given_a_built_engine().hiding().for_page("https://shop.test/cart");
    assert_eq!(answer["selectors"], json!([".cookie-banner"]));
}

#[test]
fn should_hide_nothing_on_a_site_whose_protection_is_off() {
    let scenario = PageScenario::given_a_built_engine();
    scenario.protection.set_protected("shop.test", false);
    assert_eq!(scenario.hiding().for_page("https://shop.test/cart"), Value::Null);
}

#[test]
fn should_hide_nothing_before_the_engine_is_built() {
    let profile = TempDir::new().expect("profile");
    let guard = SharedGuardEngine::default();
    let protection = SiteProtection::open(profile.path());
    assert_eq!(GuardPageHiding::new(&guard, &protection).for_page("https://shop.test/"), Value::Null);
}

#[test]
fn should_hide_nothing_on_a_page_that_is_not_on_the_web() {
    assert_eq!(PageScenario::given_a_built_engine().hiding().for_page("file:///etc/hosts"), Value::Null);
}

#[test]
fn should_answer_the_generic_selectors_the_reported_classes_need() {
    let report = json!({"classes": ["sponsored", "article"], "ids": [], "exceptions": []}).to_string();
    let answer = PageScenario::given_a_built_engine().hiding().generic_for_page("https://news.test/", &report);
    assert_eq!(answer, json!([".sponsored"]));
}

#[test]
fn should_answer_nothing_for_a_report_that_is_not_understood() {
    let answer = PageScenario::given_a_built_engine().hiding().generic_for_page("https://news.test/", "not json");
    assert_eq!(answer, json!([]));
}

#[test]
fn should_ignore_a_class_name_longer_than_any_real_one() {
    let long = "a".repeat(300);
    let report = json!({"classes": [long], "ids": [], "exceptions": []}).to_string();
    let answer = PageScenario::given_a_built_engine().hiding().generic_for_page("https://news.test/", &report);
    assert_eq!(answer, json!([]));
}

#[test]
fn should_reject_the_cookie_notices_of_a_protected_site_even_before_the_engine_is_built() {
    let profile = TempDir::new().expect("profile");
    let guard = SharedGuardEngine::default();
    let protection = SiteProtection::open(profile.path());
    assert!(GuardPageHiding::new(&guard, &protection).rejects_cookie_notices("https://shop.test/"));
}

#[test]
fn should_leave_the_cookie_notices_of_a_site_whose_protection_is_off() {
    let scenario = PageScenario::given_a_built_engine();
    scenario.protection.set_protected("shop.test", false);
    assert!(!scenario.hiding().rejects_cookie_notices("https://shop.test"));
}

#[test]
fn should_leave_the_cookie_notices_of_a_page_that_is_not_on_the_web() {
    assert!(!PageScenario::given_a_built_engine().hiding().rejects_cookie_notices("null"));
}
