//! The page of installed apps: tiles that open an app or uninstall it, built from text a site chose.

use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::web_app::installed_apps_page::AppTile;
use ztbrowser_core::web_app::installed_apps_page::InstalledAppsLink;
use ztbrowser_core::web_app::installed_apps_page::InstalledAppsPage;
use ztbrowser_core::web_app::web_app_id::WebAppId;

struct PageScenario {
    catalogue: MessageCatalogue,
    is_dark: bool,
}

impl PageScenario {
    fn given_the_english_interface() -> Self {
        Self { catalogue: MessageCatalogue::untranslated(), is_dark: false }
    }

    fn in_the_dark_scheme(mut self) -> Self {
        self.is_dark = true;
        self
    }

    fn when_rendering(&self, tiles: &[AppTile<'_>]) -> String {
        InstalledAppsPage::new(&self.catalogue, self.is_dark).render(tiles)
    }
}

fn id(text: &str) -> WebAppId {
    WebAppId::parse(text).expect("valid id")
}

#[test]
fn should_link_each_tile_to_opening_and_uninstalling_its_app() {
    let mail = id("mail_google_com");
    let tile = AppTile { id: &mail, name: "Gmail", host: "mail.google.com", icon: None };

    let page = PageScenario::given_the_english_interface().when_rendering(&[tile]);

    assert!(page.contains("href=\"ztbrowser-app-open:mail_google_com\""));
    assert!(page.contains("href=\"ztbrowser-app-uninstall:mail_google_com\""));
}

#[test]
fn should_escape_a_name_a_site_chose() {
    let evil = id("evil_example");
    let tile = AppTile { id: &evil, name: "<script>alert(1)</script>\"", host: "a&b", icon: None };

    let page = PageScenario::given_the_english_interface().when_rendering(&[tile]);

    assert!(!page.contains("<script>"));
    assert!(page.contains("&lt;script&gt;alert(1)&lt;/script&gt;&quot;"));
    assert!(page.contains("a&amp;b"));
}

#[test]
fn should_show_the_icon_when_it_is_a_data_address() {
    let app = id("mail_google_com");
    let tile = AppTile { id: &app, name: "Gmail", host: "mail.google.com", icon: Some("data:image/png;base64,AAAA") };

    let page = PageScenario::given_the_english_interface().when_rendering(&[tile]);

    assert!(page.contains("<img src=\"data:image/png;base64,AAAA\""));
}

#[test]
fn should_refuse_an_icon_address_that_could_close_the_attribute() {
    let app = id("mail_google_com");
    let tile = AppTile { id: &app, name: "Gmail", host: "h", icon: Some("data:image/png;base64,AA\" onerror=\"x") };

    let page = PageScenario::given_the_english_interface().when_rendering(&[tile]);

    assert!(!page.contains("onerror"));
    assert!(page.contains("class=\"initial\">G<"));
}

#[test]
fn should_say_so_when_no_app_is_installed() {
    let page = PageScenario::given_the_english_interface().when_rendering(&[]);

    assert!(page.contains("No apps installed yet"));
    assert!(!page.contains("class=\"grid\""));
}

#[test]
fn should_paint_the_page_in_the_scheme_of_the_window() {
    let light = PageScenario::given_the_english_interface().when_rendering(&[]);
    let dark = PageScenario::given_the_english_interface().in_the_dark_scheme().when_rendering(&[]);

    assert!(light.contains("#fafafb"));
    assert!(dark.contains("#222226"));
}

#[test]
fn should_allow_no_script_and_no_network_on_the_page() {
    let page = PageScenario::given_the_english_interface().when_rendering(&[]);

    assert!(page.contains("default-src 'none'; img-src data:"));
}

#[test]
fn should_recognise_the_open_and_uninstall_links() {
    assert_eq!(
        InstalledAppsLink::parse("ztbrowser-app-open:mail_google_com"),
        Some(InstalledAppsLink::Open(id("mail_google_com")))
    );
    assert_eq!(
        InstalledAppsLink::parse("ztbrowser-app-uninstall:mail_google_com"),
        Some(InstalledAppsLink::Uninstall(id("mail_google_com")))
    );
}

#[test]
fn should_not_recognise_other_addresses_or_unsafe_identifiers() {
    assert_eq!(InstalledAppsLink::parse("https://example.com/"), None);
    assert_eq!(InstalledAppsLink::parse("ztbrowser-app-open:../etc"), None);
    assert_eq!(InstalledAppsLink::parse("ztbrowser-app-uninstall:"), None);
}

#[test]
fn should_ask_which_app_is_about_to_be_uninstalled() {
    let catalogue = MessageCatalogue::untranslated();

    assert_eq!(InstalledAppsPage::uninstall_heading(&catalogue, "Gmail"), "Uninstall Gmail?");
}
