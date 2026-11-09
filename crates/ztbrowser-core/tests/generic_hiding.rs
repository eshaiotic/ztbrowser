//! ZTBrowser Guard hides elements generic filters name (`##.ad-banner`) lazily: only the selectors whose class
//! or id the page actually uses go into its style sheet. Thirty thousand generic selectors in every page's
//! style sheet made WebKit recalculate styles six times slower.

use ztbrowser_core::guard::generic_hiding::GenericHiding;

struct HidingScenario {
    hiding: GenericHiding,
}

impl HidingScenario {
    fn given_selector(selector: &str) -> Self {
        Self::given_selectors(&[selector])
    }

    fn given_selectors(selectors: &[&str]) -> Self {
        let mut hiding = GenericHiding::default();
        for selector in selectors {
            hiding.add(selector, &[]);
        }
        Self { hiding }
    }

    fn given_selector_except_on(selector: &str, domains: &[&str]) -> Self {
        let mut hiding = GenericHiding::default();
        hiding.add(selector, &domains.iter().map(|domain| (*domain).to_owned()).collect::<Vec<_>>());
        Self { hiding }
    }

    fn and_selector_except_on(mut self, selector: &str, domains: &[&str]) -> Self {
        self.hiding.add(selector, &domains.iter().map(|domain| (*domain).to_owned()).collect::<Vec<_>>());
        self
    }

    fn and_switched_off_on(mut self, site: &str) -> Self {
        self.hiding.switch_off_on(site);
        self
    }

    fn when_stored_and_read_back(self) -> Self {
        Self { hiding: GenericHiding::from_json(&self.hiding.to_json()).expect("the stored copy reads back") }
    }

    fn then_hides_on(&self, host: &str, page_tokens: &[&str], expected: &[&str]) {
        assert_eq!(self.hiding.selectors_for(host, &Self::tokens(page_tokens)), expected);
    }

    fn then_hands_the_page(&self, host: &str, expected: &[&str]) {
        assert_eq!(self.hiding.unkeyed_selectors_for(host), expected);
    }

    fn tokens(page_tokens: &[&str]) -> Vec<String> {
        page_tokens.iter().map(|token| (*token).to_owned()).collect()
    }
}

#[test]
fn should_hide_a_selector_once_the_page_uses_its_class() {
    HidingScenario::given_selector(".ad-banner").then_hides_on("news.pl", &[".ad-banner"], &[".ad-banner"]);
}

#[test]
fn should_hide_a_selector_once_the_page_uses_its_id() {
    HidingScenario::given_selector("#sponsored").then_hides_on("news.pl", &["#sponsored"], &["#sponsored"]);
}

#[test]
fn should_skip_selectors_whose_class_the_page_does_not_use() {
    HidingScenario::given_selector(".ad-banner").then_hides_on("news.pl", &[".article"], &[]);
}

#[test]
fn should_need_the_first_class_of_a_compound_selector() {
    HidingScenario::given_selector("div.sponsored > a").then_hides_on(
        "news.pl",
        &[".sponsored"],
        &["div.sponsored > a"],
    );
}

#[test]
fn should_not_count_a_class_inside_a_negation_as_needed() {
    HidingScenario::given_selector("a:not(.safe)").then_hands_the_page("news.pl", &["a:not(.safe)"]);
}

#[test]
fn should_not_count_a_dot_inside_an_attribute_value_as_a_class() {
    HidingScenario::given_selector("a[href*=\".doubleclick.net\"]")
        .then_hands_the_page("news.pl", &["a[href*=\".doubleclick.net\"]"]);
}

#[test]
fn should_leave_a_selector_list_to_the_page() {
    HidingScenario::given_selector(".ad, .promo").then_hands_the_page("news.pl", &[".ad, .promo"]);
}

#[test]
fn should_leave_an_escaped_class_to_the_page() {
    HidingScenario::given_selector(".\\31 0-ads").then_hands_the_page("news.pl", &[".\\31 0-ads"]);
}

#[test]
fn should_not_hand_the_page_a_selector_it_can_ask_for_by_class() {
    HidingScenario::given_selectors(&[".ad-banner", "[data-ad-slot]"])
        .then_hands_the_page("news.pl", &["[data-ad-slot]"]);
}

#[test]
fn should_keep_a_selector_off_on_a_domain_its_filter_excludes() {
    HidingScenario::given_selector_except_on(".ad-banner", &["bank.pl"]).then_hides_on("bank.pl", &[".ad-banner"], &[]);
}

#[test]
fn should_apply_an_excluded_domain_to_itself_alone() {
    HidingScenario::given_selector_except_on(".ad-banner", &["bank.pl"]).then_hides_on(
        "www.bank.pl",
        &[".ad-banner"],
        &[".ad-banner"],
    );
}

#[test]
fn should_apply_a_starred_excluded_domain_to_its_subdomains() {
    HidingScenario::given_selector_except_on(".ad-banner", &["*bank.pl"]).then_hides_on(
        "www.bank.pl",
        &[".ad-banner"],
        &[],
    );
}

#[test]
fn should_keep_unkeyed_selectors_off_on_a_domain_their_filter_excludes() {
    HidingScenario::given_selector_except_on("[data-ad-slot]", &["bank.pl"]).then_hands_the_page("bank.pl", &[]);
}

#[test]
fn should_lift_a_selector_on_every_domain_its_exceptions_name() {
    HidingScenario::given_selector(".ad-banner")
        .and_selector_except_on(".ad-banner", &["bank.pl"])
        .and_selector_except_on(".ad-banner", &["shop.pl"])
        .then_hides_on("shop.pl", &[".ad-banner"], &[]);
}

#[test]
fn should_hide_a_selector_named_by_several_filters_once() {
    HidingScenario::given_selector(".ad-banner").and_selector_except_on(".ad-banner", &["bank.pl"]).then_hides_on(
        "news.pl",
        &[".ad-banner"],
        &[".ad-banner"],
    );
}

#[test]
fn should_hide_nothing_generic_on_a_site_generic_hiding_is_switched_off_for() {
    HidingScenario::given_selector(".ad-banner").and_switched_off_on("bank.pl").then_hides_on(
        "online.bank.pl",
        &[".ad-banner"],
        &[],
    );
}

#[test]
fn should_hand_the_page_nothing_on_a_site_generic_hiding_is_switched_off_for() {
    HidingScenario::given_selector("[data-ad-slot]").and_switched_off_on("bank.pl").then_hands_the_page("bank.pl", &[]);
}

#[test]
fn should_keep_generic_hiding_on_beside_a_site_it_is_switched_off_for() {
    HidingScenario::given_selector(".ad-banner").and_switched_off_on("bank.pl").then_hides_on(
        "notbank.pl",
        &[".ad-banner"],
        &[".ad-banner"],
    );
}

#[test]
fn should_remember_where_generic_hiding_is_switched_off_after_being_stored() {
    HidingScenario::given_selector(".ad-banner")
        .and_switched_off_on("bank.pl")
        .when_stored_and_read_back()
        .then_hides_on("bank.pl", &[".ad-banner"], &[]);
}

#[test]
fn should_answer_the_same_after_being_stored() {
    HidingScenario::given_selectors(&[".ad-banner", "[data-ad-slot]"]).when_stored_and_read_back().then_hides_on(
        "news.pl",
        &[".ad-banner"],
        &[".ad-banner"],
    );
}

#[test]
fn should_reject_a_damaged_stored_copy() {
    assert!(GenericHiding::from_json("{\"selectors\": 7}").is_none());
}
