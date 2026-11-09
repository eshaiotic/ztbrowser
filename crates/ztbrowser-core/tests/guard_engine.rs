//! ZTBrowser Guard decides from the community filter lists what a page may load and what it must hide.

use ztbrowser_core::guard::guard_engine::GuardEngine;
use ztbrowser_core::network::resource_type::ResourceType;

const LIST: &str = "\
||ads.example^
||metrics.example^$third-party
@@||ads.example/allowed.js^
example.org##.cookie-banner
##.sponsored
##div[data-ad-slot]
";

struct GuardScenario {
    engine: GuardEngine,
}

impl GuardScenario {
    fn given_the_list() -> Self {
        Self { engine: GuardEngine::from_lists(&[LIST]) }
    }

    fn blocks(&self, address: &str, page: &str, resource: ResourceType) -> bool {
        self.engine.blocks(address, page, resource)
    }
}

#[test]
fn should_block_a_request_to_a_listed_ad_server() {
    let scenario = GuardScenario::given_the_list();
    assert!(scenario.blocks("https://ads.example/banner.png", "https://news.test/", ResourceType::Image));
}

#[test]
fn should_let_an_exception_of_the_list_through() {
    let scenario = GuardScenario::given_the_list();
    assert!(!scenario.blocks("https://ads.example/allowed.js", "https://news.test/", ResourceType::Script));
}

#[test]
fn should_apply_a_third_party_filter_only_to_third_parties() {
    let scenario = GuardScenario::given_the_list();
    assert!(!scenario.blocks("https://metrics.example/a.js", "https://metrics.example/", ResourceType::Script));
    assert!(scenario.blocks("https://metrics.example/a.js", "https://news.test/", ResourceType::Script));
}

#[test]
fn should_never_block_a_navigation() {
    let scenario = GuardScenario::given_the_list();
    assert!(!scenario.blocks("https://ads.example/", "", ResourceType::MainFrame));
}

#[test]
fn should_not_block_anything_without_lists() {
    let engine = GuardEngine::from_lists(&[]);
    assert!(!engine.blocks("https://ads.example/x.png", "https://news.test/", ResourceType::Image));
}

#[test]
fn should_hide_the_selectors_a_site_names_and_those_naming_no_class_or_id() {
    let cosmetics = GuardScenario::given_the_list().engine.page_cosmetics("https://example.org/story");
    assert!(cosmetics.hide_selectors.contains(&".cookie-banner".to_owned()));
    assert!(cosmetics.hide_selectors.contains(&"div[data-ad-slot]".to_owned()));
}

#[test]
fn should_not_hide_a_sites_selectors_on_other_sites() {
    let cosmetics = GuardScenario::given_the_list().engine.page_cosmetics("https://other.test/");
    assert!(!cosmetics.hide_selectors.contains(&".cookie-banner".to_owned()));
}

#[test]
fn should_name_the_generic_selectors_the_pages_classes_need() {
    let scenario = GuardScenario::given_the_list();
    let cosmetics = scenario.engine.page_cosmetics("https://other.test/");
    let selectors = scenario.engine.generic_selectors(&["sponsored", "article"], &[], &cosmetics.exceptions);
    assert_eq!(selectors, vec![".sponsored".to_owned()]);
}
