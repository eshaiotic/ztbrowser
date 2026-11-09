//! Installing any site as its own isolated app (PWA), with its own profile and desktop identity.

use serde_json::json;
use ztbrowser_core::web_app::web_app_catalog::WebAppCatalog;
use ztbrowser_core::web_app::web_app_catalog::WebAppInstallError;
use ztbrowser_core::web_app::web_app_id::WebAppId;

struct WebAppCatalogScenario {
    catalog: WebAppCatalog,
}

impl WebAppCatalogScenario {
    fn given_no_installed_web_apps() -> Self {
        Self { catalog: WebAppCatalog::default() }
    }

    fn after_installing(mut self, name: &str, start_url: &str) -> Self {
        self.catalog.install(name, start_url).expect("installable");
        self
    }

    fn when_installing(&mut self, name: &str, start_url: &str) -> Result<WebAppId, WebAppInstallError> {
        self.catalog.install(name, start_url).map(|app| app.id.clone())
    }

    fn installed_ids(&self) -> Vec<&str> {
        self.catalog.web_apps().map(|app| app.id.as_str()).collect()
    }
}

#[test]
fn should_derive_identifier_from_the_site_host() {
    let mut scenario = WebAppCatalogScenario::given_no_installed_web_apps();

    let id = scenario.when_installing("mBank", "https://online.mbank.pl/pl/Login").unwrap();

    assert_eq!(id.as_str(), "online_mbank_pl");
}

#[test]
fn should_give_second_app_on_same_host_its_own_identifier() {
    let mut scenario =
        WebAppCatalogScenario::given_no_installed_web_apps().after_installing("Gmail", "https://mail.google.com/");

    let id = scenario.when_installing("Gmail (praca)", "https://mail.google.com/mail/u/1/").unwrap();

    assert_eq!(id.as_str(), "mail_google_com_2");
}

#[test]
fn should_refuse_to_install_non_web_addresses() {
    let mut scenario = WebAppCatalogScenario::given_no_installed_web_apps();

    assert_eq!(scenario.when_installing("Local file", "file:///etc/passwd"), Err(WebAppInstallError::NotAWebAddress));
}

#[test]
fn should_give_every_web_app_its_own_isolated_profile() {
    let mut scenario = WebAppCatalogScenario::given_no_installed_web_apps();

    let id = scenario.when_installing("mBank", "https://online.mbank.pl/").unwrap();

    assert_eq!(id.profile_id().as_str(), "app_online_mbank_pl");
}

#[test]
fn should_give_every_web_app_its_own_desktop_application_identifier() {
    let mut scenario = WebAppCatalogScenario::given_no_installed_web_apps();

    let id = scenario.when_installing("mBank", "https://online.mbank.pl/").unwrap();

    assert_eq!(id.application_id(), "io.github.ztbrowser.ZTBrowser.WebApp.online_mbank_pl");
}

#[test]
fn should_uninstall_web_app() {
    let mut scenario =
        WebAppCatalogScenario::given_no_installed_web_apps().after_installing("Gmail", "https://mail.google.com/");

    scenario.catalog.uninstall(&WebAppId::parse("mail_google_com").unwrap());

    assert_eq!(scenario.installed_ids(), [""; 0]);
}

#[test]
fn should_survive_a_save_and_load_round_trip() {
    let saved = WebAppCatalogScenario::given_no_installed_web_apps()
        .after_installing("Gmail", "https://mail.google.com/")
        .catalog
        .to_json();

    let restored = WebAppCatalog::from_json(&saved);
    let gmail = restored.web_app(&WebAppId::parse("mail_google_com").unwrap()).unwrap();

    assert_eq!((gmail.name.as_str(), gmail.start_url.as_str()), ("Gmail", "https://mail.google.com/"));
}

#[test]
fn should_skip_stored_web_apps_with_unsafe_identifiers_or_addresses() {
    let restored = WebAppCatalog::from_json(&json!({"webApps": [
        {"id": "../evil", "name": "Evil", "startUrl": "https://evil.com/"},
        {"id": "local", "name": "Local", "startUrl": "file:///etc/passwd"},
        {"id": "gmail", "name": "Gmail", "startUrl": "https://mail.google.com/"}
    ]}));

    assert_eq!(restored.web_apps().map(|app| app.id.as_str()).collect::<Vec<_>>(), ["gmail"]);
}
