//! The zoom the user chose for each site, kept per profile.

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::page_zoom::site_zoom_levels::SiteZoomLevels;
use ztbrowser_core::page_zoom::zoom_level::ZoomLevel;
use ztbrowser_core::web_app::registrable_domain_lookup::RegistrableDomainLookup;

/// Test double for the public suffix list.
struct KnownSuffixes;

impl RegistrableDomainLookup for KnownSuffixes {
    fn is_public_suffix(&self, domain: &str) -> bool {
        matches!(domain, "com" | "pl" | "com.pl")
    }
}

struct SiteZoomScenario {
    levels: SiteZoomLevels,
}

impl SiteZoomScenario {
    fn given_no_site_zoomed() -> Self {
        Self { levels: SiteZoomLevels::default() }
    }

    fn given_stored(document: &Value) -> Self {
        Self { levels: SiteZoomLevels::from_json(document) }
    }

    fn after_zooming(mut self, address: &str, percent: u64) -> Self {
        let level = ZoomLevel::from_percent(percent).expect("the scenario zooms to a zoom step");
        self.levels.remember(address, level, &KnownSuffixes);
        self
    }

    fn after_saving_and_loading(self) -> Self {
        Self { levels: SiteZoomLevels::from_json(&self.levels.to_json()) }
    }

    fn after_burning_browsing_data(mut self) -> Self {
        self.levels.forget_all();
        self
    }

    fn then_page_opens_at(&self, address: &str, expected: &str) {
        assert_eq!(self.levels.level_for(address, &KnownSuffixes).to_string(), expected);
    }

    fn then_nothing_is_stored(&self) {
        assert_eq!(self.levels.to_json(), json!({"sites": {}}));
    }
}

#[test]
fn should_open_every_page_of_a_site_at_the_zoom_given_to_one_of_them() {
    SiteZoomScenario::given_no_site_zoomed()
        .after_zooming("https://online.mbank.pl/pl/Login", 125)
        .then_page_opens_at("https://www.mbank.pl/", "125%");
}

#[test]
fn should_keep_other_sites_at_actual_size() {
    SiteZoomScenario::given_no_site_zoomed()
        .after_zooming("https://online.mbank.pl/", 125)
        .then_page_opens_at("https://mail.google.com/", "100%");
}

#[test]
fn should_tell_apart_sites_under_a_shared_suffix() {
    SiteZoomScenario::given_no_site_zoomed()
        .after_zooming("https://shop.com.pl/", 150)
        .then_page_opens_at("https://other.com.pl/", "100%");
}

#[test]
fn should_treat_a_device_on_the_local_network_as_its_own_site() {
    SiteZoomScenario::given_no_site_zoomed()
        .after_zooming("http://192.168.1.1/admin", 150)
        .then_page_opens_at("http://192.168.1.1/status", "150%");
}

#[test]
fn should_store_nothing_for_a_site_set_back_to_actual_size() {
    SiteZoomScenario::given_no_site_zoomed()
        .after_zooming("https://mbank.pl/", 125)
        .after_zooming("https://mbank.pl/", 100)
        .then_nothing_is_stored();
}

#[test]
fn should_not_remember_pages_that_are_not_on_the_web() {
    SiteZoomScenario::given_no_site_zoomed()
        .after_zooming("file:///home/user/notes.html", 150)
        .then_nothing_is_stored();
}

#[test]
fn should_survive_a_save_and_load_round_trip() {
    SiteZoomScenario::given_no_site_zoomed()
        .after_zooming("https://mbank.pl/", 175)
        .after_saving_and_loading()
        .then_page_opens_at("https://mbank.pl/", "175%");
}

#[test]
fn should_ignore_stored_sizes_it_never_offers() {
    SiteZoomScenario::given_stored(&json!({"sites": {"odd.pl": 110, "text.pl": "big", "ok.pl": 125}}))
        .after_saving_and_loading()
        .then_page_opens_at("https://odd.pl/", "100%");
}

#[test]
fn should_forget_every_site_when_browsing_data_is_burned() {
    SiteZoomScenario::given_no_site_zoomed()
        .after_zooming("https://mbank.pl/", 125)
        .after_burning_browsing_data()
        .then_nothing_is_stored();
}
