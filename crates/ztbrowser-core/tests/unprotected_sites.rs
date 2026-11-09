//! The per-profile list of sites where the user switched privacy protection off (a broken site, say).

use serde_json::json;
use ztbrowser_core::profile::unprotected_sites::UnprotectedSites;

struct UnprotectedSitesScenario {
    sites: UnprotectedSites,
}

impl UnprotectedSitesScenario {
    fn given_every_site_protected() -> Self {
        Self { sites: UnprotectedSites::default() }
    }

    fn after_switching_protection_off_on(mut self, host: &str) -> Self {
        self.sites.switch_protection_off(host);
        self
    }

    fn after_switching_protection_on_on(mut self, host: &str) -> Self {
        self.sites.switch_protection_on(host);
        self
    }

    fn then_unprotected_sites_are(&self, expected: &[&str]) {
        assert_eq!(self.sites.domains(), expected);
    }
}

#[test]
fn should_remember_site_without_its_www_prefix() {
    UnprotectedSitesScenario::given_every_site_protected()
        .after_switching_protection_off_on("WWW.Example.com")
        .then_unprotected_sites_are(&["example.com"]);
}

#[test]
fn should_keep_list_sorted_and_free_of_duplicates() {
    UnprotectedSitesScenario::given_every_site_protected()
        .after_switching_protection_off_on("zeta.pl")
        .after_switching_protection_off_on("alpha.pl")
        .after_switching_protection_off_on("zeta.pl")
        .then_unprotected_sites_are(&["alpha.pl", "zeta.pl"]);
}

#[test]
fn should_protect_site_again() {
    UnprotectedSitesScenario::given_every_site_protected()
        .after_switching_protection_off_on("example.com")
        .after_switching_protection_on_on("www.example.com")
        .then_unprotected_sites_are(&[]);
}

#[test]
fn should_survive_a_save_and_load_round_trip() {
    let saved = UnprotectedSitesScenario::given_every_site_protected()
        .after_switching_protection_off_on("example.com")
        .sites
        .to_json();

    assert_eq!(UnprotectedSites::from_json(&saved).domains(), ["example.com"]);
}

#[test]
fn should_ignore_stored_entries_that_are_not_strings() {
    assert_eq!(UnprotectedSites::from_json(&json!({"domains": [42, "ok.pl"]})).domains(), ["ok.pl"]);
}

#[test]
fn should_match_site_with_and_without_www_because_webkit_compares_hosts_exactly() {
    let scenario =
        UnprotectedSitesScenario::given_every_site_protected().after_switching_protection_off_on("example.com");

    assert_eq!(scenario.sites.hosts_to_match(), ["example.com", "www.example.com"]);
}
