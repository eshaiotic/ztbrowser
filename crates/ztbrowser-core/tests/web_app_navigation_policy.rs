//! Which navigations stay inside an installed web app and which open in the regular browser.
//!
//! A link to another site clicked inside the mBank app must not load inside the bank's isolated session;
//! a redirect (sign-in through accounts.google.com, card 3-D Secure) must stay, or logging in breaks.

use ztbrowser_core::web_app::registrable_domain_lookup::RegistrableDomainLookup;
use ztbrowser_core::web_app::web_app_navigation_policy::NavigationTrigger;
use ztbrowser_core::web_app::web_app_navigation_policy::WebAppNavigation;
use ztbrowser_core::web_app::web_app_navigation_policy::WebAppNavigationPolicy;

/// Test double for the public suffix list.
struct KnownSuffixes;

impl RegistrableDomainLookup for KnownSuffixes {
    fn is_public_suffix(&self, domain: &str) -> bool {
        matches!(domain, "com" | "pl" | "uk" | "co.uk" | "example")
    }
}

struct WebAppNavigationScenario {
    start_url: String,
}

impl WebAppNavigationScenario {
    fn given_web_app_starting_at(start_url: &str) -> Self {
        Self { start_url: start_url.to_owned() }
    }

    fn when_navigating(&self, target: &str, trigger: NavigationTrigger) -> NavigationVerdict {
        let lookup = KnownSuffixes;
        let policy = WebAppNavigationPolicy::new(&self.start_url, &lookup);
        NavigationVerdict { target: target.to_owned(), decision: policy.decide(target, trigger) }
    }

    fn when_showing(&self, address: &str) -> ShownPage {
        let lookup = KnownSuffixes;
        let policy = WebAppNavigationPolicy::new(&self.start_url, &lookup);
        ShownPage { address: address.to_owned(), is_app_page: policy.is_app_page(address) }
    }
}

struct ShownPage {
    address: String,
    is_app_page: bool,
}

impl ShownPage {
    fn then_belongs_to_the_app(&self) {
        assert!(self.is_app_page, "{}", self.address);
    }

    fn then_is_flagged_as_another_site(&self) {
        assert!(!self.is_app_page, "{}", self.address);
    }
}

struct NavigationVerdict {
    target: String,
    decision: WebAppNavigation,
}

impl NavigationVerdict {
    fn then_stays_in_web_app(&self) {
        assert_eq!(self.decision, WebAppNavigation::StayInWebApp, "{}", self.target);
    }

    fn then_opens_in_browser(&self) {
        assert_eq!(self.decision, WebAppNavigation::OpenInBrowser, "{}", self.target);
    }
}

#[test]
fn should_keep_link_to_another_page_of_the_same_site() {
    WebAppNavigationScenario::given_web_app_starting_at("https://mail.google.com/mail/u/0/")
        .when_navigating("https://accounts.google.com/settings", NavigationTrigger::LinkClicked)
        .then_stays_in_web_app();
}

#[test]
fn should_open_clicked_link_to_another_site_in_browser() {
    WebAppNavigationScenario::given_web_app_starting_at("https://mail.google.com/")
        .when_navigating("https://phishing.example/login", NavigationTrigger::LinkClicked)
        .then_opens_in_browser();
}

#[test]
fn should_keep_redirect_to_another_site_so_sign_in_and_payments_work() {
    WebAppNavigationScenario::given_web_app_starting_at("https://online.mbank.pl/")
        .when_navigating("https://3ds.payment-provider.com/challenge", NavigationTrigger::RedirectOrScript)
        .then_stays_in_web_app();
}

#[test]
fn should_compare_sites_using_public_suffixes() {
    WebAppNavigationScenario::given_web_app_starting_at("https://www.bbc.co.uk/")
        .when_navigating("https://evil.co.uk/", NavigationTrigger::LinkClicked)
        .then_opens_in_browser();
}

#[test]
fn should_keep_blank_and_blob_pages_used_by_popups_and_downloads() {
    let scenario = WebAppNavigationScenario::given_web_app_starting_at("https://mail.google.com/");

    for target in ["about:blank", "blob:https://mail.google.com/1234"] {
        scenario.when_navigating(target, NavigationTrigger::LinkClicked).then_stays_in_web_app();
    }
}

#[test]
fn should_open_clicked_mail_and_phone_links_outside_the_web_app() {
    WebAppNavigationScenario::given_web_app_starting_at("https://mail.google.com/")
        .when_navigating("mailto:someone@example.com", NavigationTrigger::LinkClicked)
        .then_opens_in_browser();
}

#[test]
fn should_present_another_page_of_the_same_site_as_part_of_the_app() {
    WebAppNavigationScenario::given_web_app_starting_at("https://mail.google.com/")
        .when_showing("https://accounts.google.com/signin")
        .then_belongs_to_the_app();
}

#[test]
fn should_flag_a_page_a_redirect_took_to_another_site() {
    WebAppNavigationScenario::given_web_app_starting_at("https://online.mbank.pl/")
        .when_showing("https://3ds.payment-provider.com/challenge")
        .then_is_flagged_as_another_site();
}
