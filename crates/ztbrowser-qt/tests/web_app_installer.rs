//! Installing a site as an app: a catalogue entry, the app's icon and a `.desktop` file the desktop lists.

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use tempfile::TempDir;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::web_app::web_app_catalog::WebAppCatalog;
use ztbrowser_qt::installation_directories::InstallationDirectories;
use ztbrowser_qt::web_app_installer::IconPng;
use ztbrowser_qt::web_app_installer::WebAppInstaller;

use ztbrowser_qt as _;

/// A real 1×1 PNG: the signature, an `IHDR` chunk and nothing the decoder is asked about.
const TINY_PNG: &[u8] = b"\x89PNG\r\n\x1a\n\x00\x00\x00\x0dIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00";
const FALLBACK_ICON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\"/>";

struct InstallationScenario {
    _root: TempDir,
    directories: InstallationDirectories,
    catalogue: MessageCatalogue,
}

impl InstallationScenario {
    fn given_an_empty_installation() -> Self {
        let root = TempDir::new().expect("root");
        let directories = InstallationDirectories::under(&root.path().join("data"), &root.path().join("cache"));
        Self { _root: root, directories, catalogue: MessageCatalogue::untranslated() }
    }

    fn installer(&self) -> WebAppInstaller<'_> {
        WebAppInstaller::new(&self.directories, PathBuf::from("/opt/ztbrowser/ZTBrowser.AppImage"), &self.catalogue)
    }

    fn desktop_entries(&self) -> Vec<PathBuf> {
        fs::read_dir(self.directories.applications_directory())
            .map(|entries| entries.filter_map(|entry| entry.ok().map(|entry| entry.path())).collect())
            .unwrap_or_default()
    }

    fn catalog(&self) -> WebAppCatalog {
        let text = fs::read(self.directories.web_app_catalog()).expect("catalog");
        WebAppCatalog::from_json(&serde_json::from_slice::<Value>(&text).expect("json"))
    }
}

#[test]
fn should_accept_a_png_of_a_sane_size() {
    assert!(IconPng::accept(TINY_PNG).is_some());
}

#[test]
fn should_refuse_bytes_that_are_not_a_png() {
    assert!(IconPng::accept(b"<svg/>").is_none());
}

#[test]
fn should_refuse_a_png_that_declares_gigantic_dimensions() {
    let mut huge = TINY_PNG.to_vec();
    huge[16..20].copy_from_slice(&30_000_u32.to_be_bytes());
    assert!(IconPng::accept(&huge).is_none());
}

#[test]
fn should_list_the_installed_app_in_the_catalogue() {
    let scenario = InstallationScenario::given_an_empty_installation();
    let app = scenario.installer().install("Mail", "https://mail.example.com/inbox", None, FALLBACK_ICON).expect("app");
    assert_eq!(scenario.catalog().web_app(&app.id).map(|saved| saved.name.clone()), Some("Mail".to_owned()));
}

#[test]
fn should_name_the_desktop_file_after_the_application_id_so_the_window_finds_its_icon() {
    let scenario = InstallationScenario::given_an_empty_installation();
    let app = scenario.installer().install("Mail", "https://mail.example.com/", None, FALLBACK_ICON).expect("app");
    let expected = scenario.directories.applications_directory().join(format!("{}.desktop", app.id.application_id()));
    assert_eq!(scenario.desktop_entries(), vec![expected]);
}

#[test]
fn should_point_the_desktop_file_at_the_icon_it_wrote_and_the_window_class_it_will_have() {
    let scenario = InstallationScenario::given_an_empty_installation();
    let icon = IconPng::accept(TINY_PNG);
    let app =
        scenario.installer().install("Mail", "https://mail.example.com/", icon.as_ref(), FALLBACK_ICON).expect("app");
    let entry = fs::read_to_string(&scenario.desktop_entries()[0]).expect("entry");
    let icon_path = scenario.directories.web_app_icon_directory(&app.id).join("icon.png");
    assert!(entry.contains(&format!("Icon={}", icon_path.display())), "{entry}");
    assert!(entry.contains(&format!("StartupWMClass={}", app.id.application_id())), "{entry}");
    assert_eq!(fs::read(icon_path).expect("icon"), TINY_PNG);
}

#[test]
fn should_start_the_app_through_the_executable_given() {
    let scenario = InstallationScenario::given_an_empty_installation();
    let app = scenario.installer().install("Mail", "https://mail.example.com/", None, FALLBACK_ICON).expect("app");
    let entry = fs::read_to_string(&scenario.desktop_entries()[0]).expect("entry");
    assert!(entry.contains(&format!("Exec=/opt/ztbrowser/ZTBrowser.AppImage --web-app {}", app.id)), "{entry}");
}

#[test]
fn should_fall_back_to_the_browsers_own_icon_when_the_site_offers_none() {
    let scenario = InstallationScenario::given_an_empty_installation();
    let app = scenario.installer().install("Mail", "https://mail.example.com/", None, FALLBACK_ICON).expect("app");
    let icon = scenario.directories.web_app_icon_directory(&app.id).join("icon.svg");
    assert_eq!(fs::read_to_string(icon).expect("icon"), FALLBACK_ICON);
}

#[test]
fn should_give_a_second_app_of_the_same_site_an_id_of_its_own() {
    let scenario = InstallationScenario::given_an_empty_installation();
    let first = scenario.installer().install("Mail", "https://mail.example.com/", None, FALLBACK_ICON).expect("first");
    let second =
        scenario.installer().install("Mail 2", "https://mail.example.com/", None, FALLBACK_ICON).expect("second");
    assert_ne!(first.id, second.id);
}

#[test]
fn should_write_nothing_for_an_address_that_is_not_a_web_page() {
    let scenario = InstallationScenario::given_an_empty_installation();
    assert!(scenario.installer().install("Files", "file:///etc/passwd", None, FALLBACK_ICON).is_err());
    assert_eq!(scenario.desktop_entries(), Vec::<PathBuf>::new());
}

#[test]
fn should_propose_the_manifest_name_and_the_best_icons_first() {
    let metadata = serde_json::json!({
        "title": "Inbox (3)",
        "manifest": "https://mail.example.com/app.webmanifest",
        "icons": [{"href": "https://mail.example.com/favicon.png", "rel": "icon", "sizes": "32x32"}]
    });
    let manifest = serde_json::json!({
        "name": "Example Mail",
        "icons": [{"src": "/icon-512.png", "sizes": "512x512", "type": "image/png"}]
    });
    let draft = WebAppInstaller::draft("https://mail.example.com/inbox", &metadata, Some(&manifest));
    assert_eq!(draft.name, "Example Mail");
    assert_eq!(
        draft.icon_addresses,
        vec!["https://mail.example.com/icon-512.png".to_owned(), "https://mail.example.com/favicon.png".to_owned()]
    );
}
