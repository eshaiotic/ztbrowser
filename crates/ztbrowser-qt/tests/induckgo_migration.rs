//! ZTBrowser was called InDuckGo up to version 1.2.4. A person who used InDuckGo keeps profiles, web apps and
//! settings: the first ZTBrowser start moves InDuckGo's folders to ZTBrowser's and points InDuckGo's application-menu
//! entries at ZTBrowser, but never while a process still runs on InDuckGo's folders.

use std::fs;
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use tempfile::TempDir;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::web_app::web_app_catalog::WebAppCatalog;
use ztbrowser_core::web_app::web_app_id::WebAppId;
use ztbrowser_qt::induckgo_migration::InDuckGoMigration;
use ztbrowser_qt::installation_directories::InstallationDirectories;
use ztbrowser_qt::single_instance::Instance;
use ztbrowser_qt::single_instance::PrimaryInstance;
use ztbrowser_qt::single_instance::SingleInstance;

use ztbrowser_qt as _;

struct MigrationScenario {
    roots: TempDir,
    web_app: Option<WebAppId>,
    /// InDuckGo's browser window, still open after the upgrade and listening for later launches.
    running_induckgo: Option<PrimaryInstance>,
}

impl MigrationScenario {
    const EXECUTABLE: &'static str = "/usr/bin/ztbrowser";
    const INDUCKGO_APPIMAGE_ENTRY: &'static str = "applications/io.github.induckgo.InDuckGo.desktop";
    const INDUCKGO_APPIMAGE_ICON: &'static str = "icons/hicolor/scalable/apps/io.github.induckgo.InDuckGo.svg";

    fn given_a_person_who_used_induckgo() -> Self {
        let scenario = Self { roots: TempDir::new().expect("roots"), web_app: None, running_induckgo: None };
        Self::write(&scenario.data_home().join("induckgo/profiles.json"), "{\"profiles\":[]}");
        Self::write(&scenario.cache_home().join("induckgo/protections/tds.json"), "{}");
        Self::write(&scenario.cache_home().join("InDuckGo/qmlcache/main.qmlc"), "compiled");
        scenario
    }

    fn given_a_person_who_never_used_induckgo() -> Self {
        Self { roots: TempDir::new().expect("roots"), web_app: None, running_induckgo: None }
    }

    fn with_induckgo_still_running(mut self) -> Self {
        let runtime = self.runtime_home().join("induckgo");
        match SingleInstance::acquire(&runtime, "io.github.induckgo.InDuckGo", &[]).expect("InDuckGo") {
            Instance::Primary(primary) => self.running_induckgo = Some(primary),
            Instance::Forwarded => panic!("nothing else was running"),
        }
        self
    }

    fn with_a_socket_left_behind_by_a_crashed_induckgo(self) -> Self {
        let socket = self.runtime_home().join("induckgo/0123456789abcdef.sock");
        fs::create_dir_all(socket.parent().expect("parent")).expect("runtime directory");
        drop(UnixListener::bind(&socket).expect("socket"));
        self
    }

    fn with_ztbrowser_folders_of_its_own(self) -> Self {
        Self::write(&self.data_home().join("ztbrowser/profiles.json"), "{\"profiles\":[]}");
        self
    }

    /// The catalogue, icon and `.desktop` file InDuckGo's installer wrote, the entry starting InDuckGo's program.
    fn with_a_web_app_installed_by_induckgo(mut self) -> Self {
        let mut catalog = WebAppCatalog::default();
        let id = catalog.install("Mail", "https://mail.example.com/").expect("web app").id.clone();
        Self::write(&self.data_home().join("induckgo/web-apps.json"), &catalog.to_json().to_string());
        Self::write(&self.data_home().join(format!("induckgo/web-apps/{id}/icon.png")), "png");
        let entry = format!("[Desktop Entry]\nName=Mail\nExec=/usr/bin/induckgo --web-app {id}\n");
        Self::write(&self.induckgo_web_app_entry(&id), &entry);
        self.web_app = Some(id);
        self
    }

    fn with_the_menu_entry_induckgo_s_appimage_wrote(self) -> Self {
        let entry = "[Desktop Entry]\nX-InDuckGo-AppImage=/home/person/Apps/InDuckGo.AppImage\nName=InDuckGo\n";
        Self::write(&self.data_home().join(Self::INDUCKGO_APPIMAGE_ENTRY), entry);
        Self::write(&self.data_home().join(Self::INDUCKGO_APPIMAGE_ICON), "<svg/>");
        self
    }

    fn with_an_induckgo_menu_entry_the_person_wrote(self) -> Self {
        Self::write(&self.data_home().join(Self::INDUCKGO_APPIMAGE_ENTRY), "[Desktop Entry]\nName=My InDuckGo\n");
        self
    }

    /// What `BrowserApplication::run` does: move the folders, use the folders that hold the data, and once those
    /// are ZTBrowser's, point the menu entries at ZTBrowser.
    fn when_ztbrowser_starts(self) -> MigrationOutcome {
        let migration = InDuckGoMigration::new(&self.data_home(), &self.cache_home(), Some(&self.runtime_home()));
        migration.move_folders().expect("move");
        migration
            .repoint_menu_entries(Path::new(Self::EXECUTABLE), &MessageCatalogue::untranslated())
            .expect("menu entries");
        MigrationOutcome { directories: migration.directories(), scenario: self }
    }

    /// A second launch of the browser asks for `address`: the window InDuckGo already shows gets it.
    fn when_the_browser_is_launched_with(self, address: &str) -> HandOverOutcome {
        let migration = InDuckGoMigration::new(&self.data_home(), &self.cache_home(), Some(&self.runtime_home()));
        let handed_over =
            migration.hand_over_to_running_induckgo("io.github.ztbrowser.ZTBrowser", &[address.to_owned()]);
        HandOverOutcome { scenario: self, handed_over }
    }

    fn data_home(&self) -> PathBuf {
        self.roots.path().join("data")
    }

    fn cache_home(&self) -> PathBuf {
        self.roots.path().join("cache")
    }

    fn runtime_home(&self) -> PathBuf {
        self.roots.path().join("run")
    }

    fn induckgo_web_app_entry(&self, id: &WebAppId) -> PathBuf {
        self.data_home().join(format!("applications/io.github.induckgo.InDuckGo.WebApp.{id}.desktop"))
    }

    fn write(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().expect("parent")).expect("folder");
        fs::write(path, content).expect("file");
    }
}

struct MigrationOutcome {
    scenario: MigrationScenario,
    directories: InstallationDirectories,
}

impl MigrationOutcome {
    fn then_ztbrowser_holds_the_profiles(&self) {
        assert!(self.scenario.data_home().join("ztbrowser/profiles.json").is_file());
    }

    fn then_ztbrowser_holds_the_downloaded_protections(&self) {
        assert!(self.scenario.cache_home().join("ztbrowser/protections/tds.json").is_file());
    }

    fn then_no_induckgo_data_folder_is_left(&self) {
        assert!(!self.scenario.data_home().join("induckgo").exists());
    }

    fn then_qt_s_induckgo_cache_is_gone(&self) {
        assert!(!self.scenario.cache_home().join("InDuckGo").exists());
    }

    fn then_the_induckgo_data_stays_where_it_was(&self) {
        assert!(self.scenario.data_home().join("induckgo/profiles.json").is_file());
    }

    fn then_the_process_uses_the_folders_called(&self, name: &str) {
        assert_eq!(self.directories.profile_catalog(), self.scenario.data_home().join(name).join("profiles.json"));
    }

    fn then_its_sockets_live_in_the_runtime_folder_called(&self, name: &str) {
        assert_eq!(self.directories.runtime_directory(), self.scenario.runtime_home().join(name));
    }

    fn then_nothing_was_created(&self) {
        assert!(!self.scenario.data_home().exists() && !self.scenario.cache_home().exists());
    }

    fn web_app_entry(&self) -> String {
        let id = self.scenario.web_app.as_ref().expect("web app");
        let name = format!("applications/{}.desktop", id.application_id());
        fs::read_to_string(self.scenario.data_home().join(name)).expect("ZTBrowser's entry")
    }

    fn then_the_web_app_entry_starts_ztbrowser(&self) {
        let id = self.scenario.web_app.as_ref().expect("web app");
        let exec = format!("Exec={} --web-app {id}", MigrationScenario::EXECUTABLE);
        assert!(self.web_app_entry().lines().any(|line| line == exec), "{}", self.web_app_entry());
    }

    fn then_the_web_app_entry_shows_the_moved_icon(&self) {
        let id = self.scenario.web_app.as_ref().expect("web app");
        let icon =
            format!("Icon={}", self.scenario.data_home().join(format!("ztbrowser/web-apps/{id}/icon.png")).display());
        assert!(self.web_app_entry().lines().any(|line| line == icon), "{}", self.web_app_entry());
    }

    fn then_induckgo_s_web_app_entry_is_gone(&self) {
        let id = self.scenario.web_app.as_ref().expect("web app");
        assert!(!self.scenario.induckgo_web_app_entry(id).exists());
    }

    fn then_induckgo_s_web_app_entry_stays(&self) {
        let id = self.scenario.web_app.as_ref().expect("web app");
        assert!(self.scenario.induckgo_web_app_entry(id).is_file());
    }

    fn then_induckgo_s_appimage_entry_is_gone(&self) {
        assert!(!self.scenario.data_home().join(MigrationScenario::INDUCKGO_APPIMAGE_ENTRY).exists());
    }

    fn then_induckgo_s_appimage_icon_is_gone(&self) {
        assert!(!self.scenario.data_home().join(MigrationScenario::INDUCKGO_APPIMAGE_ICON).exists());
    }

    fn then_the_person_s_entry_stays(&self) {
        assert!(self.scenario.data_home().join(MigrationScenario::INDUCKGO_APPIMAGE_ENTRY).is_file());
    }
}

struct HandOverOutcome {
    scenario: MigrationScenario,
    handed_over: bool,
}

impl HandOverOutcome {
    const PATIENCE: Duration = Duration::from_secs(2);
    const POLL: Duration = Duration::from_millis(10);

    fn then_this_process_is_not_needed(&self) {
        assert!(self.handed_over);
    }

    fn then_this_process_opens_the_window_itself(&self) {
        assert!(!self.handed_over);
    }

    fn then_induckgo_opens(&self, address: &str) {
        let induckgo = self.scenario.running_induckgo.as_ref().expect("InDuckGo");
        let mut received = Vec::new();
        for _ in 0..Self::PATIENCE.as_millis() / Self::POLL.as_millis() {
            received.extend(induckgo.take_forwarded());
            if !received.is_empty() {
                break;
            }
            thread::sleep(Self::POLL);
        }
        assert_eq!(received, vec![vec![address.to_owned()]]);
    }
}

#[test]
fn should_move_the_profiles_to_ztbrowser() {
    MigrationScenario::given_a_person_who_used_induckgo().when_ztbrowser_starts().then_ztbrowser_holds_the_profiles();
}

#[test]
fn should_move_the_downloaded_protections_to_ztbrowser() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .when_ztbrowser_starts()
        .then_ztbrowser_holds_the_downloaded_protections();
}

#[test]
fn should_leave_no_induckgo_data_folder_behind() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .when_ztbrowser_starts()
        .then_no_induckgo_data_folder_is_left();
}

#[test]
fn should_delete_qt_s_cache_named_after_induckgo() {
    MigrationScenario::given_a_person_who_used_induckgo().when_ztbrowser_starts().then_qt_s_induckgo_cache_is_gone();
}

#[test]
fn should_use_ztbrowser_s_folders_once_the_data_moved() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .when_ztbrowser_starts()
        .then_the_process_uses_the_folders_called("ztbrowser");
}

#[test]
fn should_use_ztbrowser_s_folders_on_a_fresh_machine() {
    MigrationScenario::given_a_person_who_never_used_induckgo()
        .when_ztbrowser_starts()
        .then_the_process_uses_the_folders_called("ztbrowser");
}

#[test]
fn should_create_nothing_on_a_fresh_machine() {
    MigrationScenario::given_a_person_who_never_used_induckgo().when_ztbrowser_starts().then_nothing_was_created();
}

#[test]
fn should_not_move_the_folders_while_induckgo_runs_on_them() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_induckgo_still_running()
        .when_ztbrowser_starts()
        .then_the_induckgo_data_stays_where_it_was();
}

#[test]
fn should_keep_using_induckgo_s_folders_while_induckgo_runs_on_them() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_induckgo_still_running()
        .when_ztbrowser_starts()
        .then_the_process_uses_the_folders_called("induckgo");
}

#[test]
fn should_listen_beside_induckgo_while_it_runs_so_the_move_waits_for_this_process_too() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_induckgo_still_running()
        .when_ztbrowser_starts()
        .then_its_sockets_live_in_the_runtime_folder_called("induckgo");
}

#[test]
fn should_move_the_folders_when_the_only_socket_is_one_a_crashed_induckgo_left() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_a_socket_left_behind_by_a_crashed_induckgo()
        .when_ztbrowser_starts()
        .then_ztbrowser_holds_the_profiles();
}

#[test]
fn should_keep_induckgo_s_folders_untouched_when_ztbrowser_has_folders_of_its_own() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_ztbrowser_folders_of_its_own()
        .when_ztbrowser_starts()
        .then_the_induckgo_data_stays_where_it_was();
}

#[test]
fn should_point_a_web_app_s_menu_entry_at_ztbrowser() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_a_web_app_installed_by_induckgo()
        .when_ztbrowser_starts()
        .then_the_web_app_entry_starts_ztbrowser();
}

#[test]
fn should_show_the_web_app_s_icon_from_its_new_folder() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_a_web_app_installed_by_induckgo()
        .when_ztbrowser_starts()
        .then_the_web_app_entry_shows_the_moved_icon();
}

#[test]
fn should_remove_induckgo_s_web_app_entry_so_the_menu_lists_the_app_once() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_a_web_app_installed_by_induckgo()
        .when_ztbrowser_starts()
        .then_induckgo_s_web_app_entry_is_gone();
}

#[test]
fn should_leave_induckgo_s_web_app_entry_while_induckgo_runs() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_a_web_app_installed_by_induckgo()
        .with_induckgo_still_running()
        .when_ztbrowser_starts()
        .then_induckgo_s_web_app_entry_stays();
}

#[test]
fn should_withdraw_the_menu_entry_induckgo_s_appimage_wrote() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_the_menu_entry_induckgo_s_appimage_wrote()
        .when_ztbrowser_starts()
        .then_induckgo_s_appimage_entry_is_gone();
}

#[test]
fn should_withdraw_the_icon_induckgo_s_appimage_wrote() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_the_menu_entry_induckgo_s_appimage_wrote()
        .when_ztbrowser_starts()
        .then_induckgo_s_appimage_icon_is_gone();
}

#[test]
fn should_keep_an_induckgo_menu_entry_the_person_wrote() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_an_induckgo_menu_entry_the_person_wrote()
        .when_ztbrowser_starts()
        .then_the_person_s_entry_stays();
}

#[test]
fn should_name_the_induckgo_process_that_shows_the_same_web_app() {
    assert_eq!(
        InDuckGoMigration::previous_identity("io.github.ztbrowser.ZTBrowser.WebApp.mail"),
        Some("io.github.induckgo.InDuckGo.WebApp.mail".to_owned())
    );
}

#[test]
fn should_name_no_induckgo_process_for_an_identity_ztbrowser_never_announces() {
    assert_eq!(InDuckGoMigration::previous_identity("org.example.Other"), None);
}

#[test]
fn should_hand_a_launch_to_the_induckgo_that_shows_the_same_window() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_induckgo_still_running()
        .when_the_browser_is_launched_with("https://example.com/")
        .then_this_process_is_not_needed();
}

#[test]
fn should_give_the_launch_s_address_to_that_induckgo() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .with_induckgo_still_running()
        .when_the_browser_is_launched_with("https://example.com/")
        .then_induckgo_opens("https://example.com/");
}

#[test]
fn should_open_the_window_itself_when_no_induckgo_runs() {
    MigrationScenario::given_a_person_who_used_induckgo()
        .when_the_browser_is_launched_with("https://example.com/")
        .then_this_process_opens_the_window_itself();
}
