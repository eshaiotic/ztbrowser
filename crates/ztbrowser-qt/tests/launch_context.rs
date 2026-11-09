//! What this process was asked to be: the browser, or one installed web app with its own profile and identity.

use std::fs;

use tempfile::TempDir;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_qt::installation_directories::InstallationDirectories;
use ztbrowser_qt::launch_context::LaunchContext;
use ztbrowser_qt::launch_context::LaunchError;
use ztbrowser_qt::web_app_installer::WebAppInstaller;

use ztbrowser_qt as _;

struct LaunchScenario {
    _root: TempDir,
    directories: InstallationDirectories,
}

impl LaunchScenario {
    fn given_an_installation_with_a_mail_app() -> (Self, String) {
        let root = TempDir::new().expect("root");
        let directories = InstallationDirectories::under(&root.path().join("data"), &root.path().join("cache"));
        let catalogue = MessageCatalogue::untranslated();
        let installer = WebAppInstaller::new(&directories, "/usr/bin/ztbrowser-qt".into(), &catalogue);
        let app = installer.install("Mail", "https://mail.example.com/", None, "<svg/>").expect("app");
        (Self { _root: root, directories }, app.id.to_string())
    }

    fn resolve(&self, arguments: &[&str]) -> Result<LaunchContext, LaunchError> {
        let arguments: Vec<String> = arguments.iter().map(|argument| (*argument).to_owned()).collect();
        LaunchContext::resolve(&arguments, &self.directories)
    }
}

#[test]
fn should_open_the_browser_when_no_option_is_given() {
    let (scenario, _) = LaunchScenario::given_an_installation_with_a_mail_app();
    let launch = scenario.resolve(&["https://example.com/"]).expect("launch");
    assert!(launch.web_app().is_none());
    assert_eq!(launch.addresses(), ["https://example.com/".to_owned()]);
}

#[test]
fn should_open_an_installed_app_in_its_own_profile() {
    let (scenario, id) = LaunchScenario::given_an_installation_with_a_mail_app();
    let launch = scenario.resolve(&["--web-app", &id]).expect("launch");
    assert_eq!(launch.web_app().map(|app| app.name.clone()), Some("Mail".to_owned()));
    assert_eq!(launch.profile_id().to_string(), format!("app_{id}"));
}

#[test]
fn should_take_the_desktop_identity_from_the_app_so_the_compositor_finds_its_icon() {
    let (scenario, id) = LaunchScenario::given_an_installation_with_a_mail_app();
    let launch = scenario.resolve(&["--web-app", &id]).expect("launch");
    assert_eq!(launch.application_id(), format!("io.github.ztbrowser.ZTBrowser.WebApp.{id}"));
}

#[test]
fn should_use_the_icon_the_installer_wrote_as_the_window_icon() {
    let (scenario, id) = LaunchScenario::given_an_installation_with_a_mail_app();
    let launch = scenario.resolve(&["--web-app", &id]).expect("launch");
    let icon = launch.window_icon().expect("icon");
    assert!(fs::metadata(&icon).is_ok(), "{}", icon.display());
}

#[test]
fn should_refuse_an_app_that_is_not_installed() {
    let (scenario, _) = LaunchScenario::given_an_installation_with_a_mail_app();
    assert!(matches!(scenario.resolve(&["--web-app", "nothing"]), Err(LaunchError::UnknownWebApp(_))));
}

#[test]
fn should_refuse_a_tor_window_until_the_qt_edition_has_one() {
    let (scenario, _) = LaunchScenario::given_an_installation_with_a_mail_app();
    assert!(matches!(scenario.resolve(&["--tor"]), Err(LaunchError::TorWindowUnavailable)));
}

#[test]
fn should_refuse_options_it_does_not_know() {
    let (scenario, _) = LaunchScenario::given_an_installation_with_a_mail_app();
    assert!(matches!(scenario.resolve(&["--frobnicate"]), Err(LaunchError::Request(_))));
}
