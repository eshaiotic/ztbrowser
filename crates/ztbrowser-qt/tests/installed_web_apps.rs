//! The installed apps as the apps page lists them, and removing one with all it owns.

use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::web_app::web_app_id::WebAppId;
use ztbrowser_qt::installation_directories::InstallationDirectories;
use ztbrowser_qt::installed_web_apps::InstalledWebApps;
use ztbrowser_qt::installed_web_apps::UninstallError;
use ztbrowser_qt::single_instance::Instance;
use ztbrowser_qt::single_instance::SingleInstance;
use ztbrowser_qt::web_app_installer::WebAppInstaller;

use ztbrowser_qt as _;

const FALLBACK_ICON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\"/>";

struct AppsScenario {
    _root: TempDir,
    directories: InstallationDirectories,
    runtime: PathBuf,
    catalogue: MessageCatalogue,
}

impl AppsScenario {
    fn given_an_installation() -> Self {
        let root = TempDir::new().expect("root");
        let directories = InstallationDirectories::under(&root.path().join("data"), &root.path().join("cache"));
        let runtime = root.path().join("run");
        Self { _root: root, directories, runtime, catalogue: MessageCatalogue::untranslated() }
    }

    fn after_installing(self, name: &str, start_url: &str) -> Self {
        let installer = WebAppInstaller::new(&self.directories, PathBuf::from("/opt/ztbrowser"), &self.catalogue);
        let app = installer.install(name, start_url, None, FALLBACK_ICON).expect("installs");
        let profile = self.directories.profile(&app.id.profile_id());
        fs::create_dir_all(&profile.data).expect("profile data");
        fs::write(profile.data.join("cookies"), b"session").expect("cookie");
        self
    }

    fn apps(&self) -> InstalledWebApps<'_> {
        InstalledWebApps::new(&self.directories, self.runtime.clone())
    }

    fn id(text: &str) -> WebAppId {
        WebAppId::parse(text).expect("id")
    }
}

#[test]
fn should_list_every_installed_app_with_its_icon_on_the_page() {
    let scenario = AppsScenario::given_an_installation()
        .after_installing("Gmail", "https://mail.google.com/")
        .after_installing("mBank", "https://online.mbank.pl/");

    let page = scenario.apps().page_html(&scenario.catalogue, false);

    assert!(page.contains("Gmail") && page.contains("mBank"));
    assert!(page.contains("ztbrowser-app-open:online_mbank_pl"));
    assert!(page.contains("<img src=\"data:image/svg+xml;base64,"));
}

#[test]
fn should_remove_the_apps_files_data_and_catalogue_entry_on_uninstall() {
    let scenario = AppsScenario::given_an_installation().after_installing("Gmail", "https://mail.google.com/");
    let id = AppsScenario::id("mail_google_com");

    scenario.apps().uninstall(&id).expect("uninstalls");

    assert!(scenario.apps().catalog().web_apps().next().is_none());
    assert!(!scenario.directories.web_app_icon_directory(&id).exists());
    assert!(!scenario.directories.profile(&id.profile_id()).data.exists());
    assert!(fs::read_dir(scenario.directories.applications_directory()).expect("applications").next().is_none());
}

#[test]
fn should_keep_the_other_apps_when_one_is_uninstalled() {
    let scenario = AppsScenario::given_an_installation()
        .after_installing("Gmail", "https://mail.google.com/")
        .after_installing("mBank", "https://online.mbank.pl/");

    scenario.apps().uninstall(&AppsScenario::id("mail_google_com")).expect("uninstalls");

    let remaining: Vec<String> = scenario.apps().catalog().web_apps().map(|app| app.name.clone()).collect();
    assert_eq!(remaining, ["mBank"]);
}

#[test]
fn should_refuse_to_uninstall_an_app_that_is_open() {
    let scenario = AppsScenario::given_an_installation().after_installing("Gmail", "https://mail.google.com/");
    let id = AppsScenario::id("mail_google_com");
    let Ok(Instance::Primary(_open_app)) =
        SingleInstance::acquire(&scenario.runtime, &id.application_id().to_string(), &[])
    else {
        panic!("the app should start")
    };

    let result = scenario.apps().uninstall(&id);

    assert!(matches!(result, Err(UninstallError::StillRunning(name)) if name == "Gmail"));
    assert!(scenario.directories.profile(&id.profile_id()).data.exists());
}
