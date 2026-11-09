//! Which open pages belong to an installed web app.
//!
//! After a site is installed as an app, the browser hands over to the app and closes the tabs that showed it.
//! A page belongs to the app when it has the same *origin* (scheme, host and port) as the app's start page:
//! installing Gmail takes the `mail.google.com` tabs, not a Google search on `www.google.com`.

use ztbrowser_core::web_app::installed_web_app::InstalledWebApp;
use ztbrowser_core::web_app::web_app_id::WebAppId;

struct InstalledWebAppScenario {
    app: InstalledWebApp,
}

impl InstalledWebAppScenario {
    fn given_app_starting_at(start_url: &str) -> Self {
        Self {
            app: InstalledWebApp {
                id: WebAppId::parse("app").expect("valid id"),
                name: "App".to_owned(),
                start_url: start_url.to_owned(),
            },
        }
    }

    fn then_owns(&self, address: &str) {
        assert!(self.app.is_own_page(address), "{address} should belong to {}", self.app.start_url);
    }

    fn then_does_not_own(&self, address: &str) {
        assert!(!self.app.is_own_page(address), "{address} should not belong to {}", self.app.start_url);
    }
}

#[test]
fn should_own_every_page_of_the_start_page_origin() {
    let gmail = InstalledWebAppScenario::given_app_starting_at("https://mail.google.com/mail/u/0/");

    gmail.then_owns("https://mail.google.com/mail/u/0/#inbox");
    gmail.then_owns("https://mail.google.com/settings?tab=general");
}

#[test]
fn should_compare_hosts_regardless_of_letter_case() {
    InstalledWebAppScenario::given_app_starting_at("https://online.mbank.pl/").then_owns("https://ONLINE.mbank.pl/pl");
}

#[test]
fn should_not_own_other_hosts_of_the_same_site() {
    let gmail = InstalledWebAppScenario::given_app_starting_at("https://mail.google.com/mail/u/0/");

    gmail.then_does_not_own("https://www.google.com/search?q=ztbrowser");
    gmail.then_does_not_own("https://accounts.google.com/");
}

#[test]
fn should_not_own_another_scheme_or_port_of_the_same_host() {
    let app = InstalledWebAppScenario::given_app_starting_at("https://example.com/app");

    app.then_does_not_own("http://example.com/app");
    app.then_does_not_own("https://example.com:8443/app");
}

#[test]
fn should_own_no_address_that_is_not_a_web_page() {
    let app = InstalledWebAppScenario::given_app_starting_at("https://example.com/");

    app.then_does_not_own("about:blank");
    app.then_does_not_own("file:///home/user/example.com/index.html");
    app.then_does_not_own("not an address");
}
