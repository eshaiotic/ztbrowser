//! An AppImage puts itself into the application menu, so the desktop can show its icon.
//!
//! A packaged ZTBrowser already has its entry in `/usr/share/applications`; the AppImage must not shadow it.
//! The entry makes the desktop run the AppImage file later, so it is only written for a file nobody else can
//! replace, it is withdrawn when the file stops being such a file, and it never touches an entry the person
//! wrote themselves.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;

use tempfile::TempDir;
use ztbrowser_qt::appimage_integration::AppImageIntegration;
use ztbrowser_qt::installation_directories::InstallationDirectories;

use ztbrowser_qt as _;

struct IntegrationScenario {
    roots: TempDir,
    appimage: PathBuf,
    /// The folders from here upwards are the person's own business and are not examined.
    home: PathBuf,
}

impl IntegrationScenario {
    const ENTRY: &'static str = "applications/io.github.ztbrowser.ZTBrowser.desktop";
    const ICON: &'static str = "icons/hicolor/scalable/apps/io.github.ztbrowser.ZTBrowser.svg";

    fn given_an_appimage_on_a_machine_without_an_installed_package() -> Self {
        let roots = TempDir::new().expect("roots");
        let appimage = Self::appimage_in(&roots.path().join("Apps"), "ZTBrowser.AppImage");
        let home = roots.path().to_path_buf();
        Self { roots, appimage, home }
    }

    /// An executable file in a directory only its owner can write to.
    fn appimage_in(directory: &Path, name: &str) -> PathBuf {
        fs::create_dir_all(directory).expect("directory");
        fs::set_permissions(directory, fs::Permissions::from_mode(0o755)).expect("directory mode");
        let appimage = directory.join(name);
        fs::write(&appimage, b"AppImage").expect("appimage");
        fs::set_permissions(&appimage, fs::Permissions::from_mode(0o755)).expect("appimage mode");
        appimage
    }

    fn with_the_package_installed_system_wide(self) -> Self {
        let entry = self.system_data().join(Self::ENTRY);
        fs::create_dir_all(entry.parent().expect("parent")).expect("system applications");
        fs::write(entry, "[Desktop Entry]\n").expect("system entry");
        self
    }

    fn with_the_appimage_moved_to_another_folder(mut self) -> Self {
        self.integrate();
        self.appimage = Self::appimage_in(&self.roots.path().join("Downloads"), "ZTBrowser.AppImage");
        self
    }

    fn with_the_appimage_file_writable_by_others(self) -> Self {
        fs::set_permissions(&self.appimage, fs::Permissions::from_mode(0o757)).expect("appimage mode");
        self
    }

    fn with_the_appimage_folder_writable_by_the_group(self) -> Self {
        let folder = self.appimage.parent().expect("folder");
        fs::set_permissions(folder, fs::Permissions::from_mode(0o775)).expect("folder mode");
        self
    }

    fn with_the_appimage_folder_open_to_everyone_like_a_shared_temporary_folder(self) -> Self {
        let folder = self.appimage.parent().expect("folder");
        fs::set_permissions(folder, fs::Permissions::from_mode(0o1777)).expect("folder mode");
        self
    }

    fn with_a_folder_above_the_appimage_folder_writable_by_the_group(self) -> Self {
        let deeper = Self::appimage_in(&self.roots.path().join("Apps/Versions"), "ZTBrowser.AppImage");
        fs::set_permissions(self.roots.path().join("Apps"), fs::Permissions::from_mode(0o775)).expect("folder mode");
        Self { appimage: deeper, ..self }
    }

    /// The path the person started is a link; the file it leads to lies in `target_folder_mode`.
    fn with_the_appimage_reached_through_a_link_to_a_folder_with_mode(self, target_folder_mode: u32) -> Self {
        let real = Self::appimage_in(&self.roots.path().join("Shared"), "ZTBrowser-1.2.3.AppImage");
        fs::set_permissions(real.parent().expect("folder"), fs::Permissions::from_mode(target_folder_mode))
            .expect("folder mode");
        let link = self.roots.path().join("Apps/ZTBrowser.AppImage-link");
        symlink(&real, &link).expect("link");
        Self { appimage: link, ..self }
    }

    fn real_appimage(&self) -> PathBuf {
        fs::canonicalize(self.roots.path().join("Shared/ZTBrowser-1.2.3.AppImage")).expect("real file")
    }

    /// The AppImage lies in a folder of its own inside a folder everybody can create files in, outside the home.
    fn with_the_appimage_in_a_folder_below_a_shared_temporary_folder(mut self) -> Self {
        let shared = self.roots.path().join("Shared");
        fs::create_dir_all(&shared).expect("shared");
        fs::set_permissions(&shared, fs::Permissions::from_mode(0o1777)).expect("shared mode");
        self.appimage = Self::appimage_in(&shared.join("Apps"), "ZTBrowser.AppImage");
        self.home = self.roots.path().join("home");
        fs::create_dir_all(&self.home).expect("home");
        self
    }

    fn with_a_missing_appimage(mut self) -> Self {
        self.appimage = self.roots.path().join("Apps/Gone.AppImage");
        self
    }

    fn with_an_entry_the_person_wrote(self, text: &str) -> Self {
        let entry = self.user_file(Self::ENTRY);
        fs::create_dir_all(entry.parent().expect("parent")).expect("applications");
        fs::write(entry, text).expect("own entry");
        self
    }

    /// The entry names this copy; it is then made unsafe, and a different copy, itself unsafe, is started.
    fn when_another_unsafe_copy_is_started_after_the_one_the_entry_names_became_unsafe(mut self) -> Self {
        self.integrate();
        fs::set_permissions(&self.appimage, fs::Permissions::from_mode(0o757)).expect("appimage mode");
        let shared = self.roots.path().join("Shared");
        fs::create_dir_all(&shared).expect("shared");
        fs::set_permissions(&shared, fs::Permissions::from_mode(0o1777)).expect("shared mode");
        self.appimage = Self::appimage_in(&shared, "ZTBrowser-copy.AppImage");
        fs::set_permissions(&shared, fs::Permissions::from_mode(0o1777)).expect("shared mode");
        self.integrate();
        self
    }

    fn data_home(&self) -> PathBuf {
        self.roots.path().join("data")
    }

    fn system_data(&self) -> PathBuf {
        self.roots.path().join("usr-share")
    }

    fn integrate(&self) {
        let directories = InstallationDirectories::under(&self.data_home(), &self.roots.path().join("cache"));
        AppImageIntegration::new(&directories, &self.home, vec![self.system_data()])
            .install(&self.appimage)
            .expect("install");
    }

    fn when_started(self) -> Self {
        self.integrate();
        self
    }

    /// The person closed the AppImage's permissions behind the desktop's back, then starts it again.
    fn when_started_again_after_others_were_allowed_to_write_to_the_appimage(self) -> Self {
        self.integrate();
        fs::set_permissions(&self.appimage, fs::Permissions::from_mode(0o757)).expect("appimage mode");
        self.integrate();
        self
    }

    /// The AppImage wrote its entry; a package is installed later, and the AppImage is started again.
    fn when_started_again_after_the_package_was_installed_system_wide(self) -> Self {
        self.integrate();
        self.with_the_package_installed_system_wide().when_started()
    }

    fn user_file(&self, relative: &str) -> PathBuf {
        self.data_home().join(relative)
    }

    fn entry_text(&self) -> String {
        fs::read_to_string(self.user_file(Self::ENTRY)).expect("entry")
    }
}

#[test]
fn should_write_a_menu_entry_that_starts_the_appimage() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package().when_started();

    assert!(scenario.entry_text().contains(&format!("Exec={} %U", scenario.appimage.display())));
}

#[test]
fn should_write_the_icon_the_entry_names() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package().when_started();

    let icon = scenario.user_file(IntegrationScenario::ICON);
    assert!(scenario.entry_text().contains(&format!("Icon={}", icon.display())));
    assert!(fs::read_to_string(icon).expect("icon").contains("<svg"));
}

#[test]
fn should_follow_the_appimage_when_it_is_moved() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_the_appimage_moved_to_another_folder()
        .when_started();

    assert!(scenario.entry_text().contains(&format!("Exec={} %U", scenario.appimage.display())));
}

#[test]
fn should_leave_the_installed_package_alone() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_the_package_installed_system_wide()
        .when_started();

    assert!(!scenario.user_file(IntegrationScenario::ENTRY).exists());
}

#[test]
fn should_withdraw_its_entry_when_a_package_is_installed_afterwards_so_the_menu_starts_the_package() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .when_started_again_after_the_package_was_installed_system_wide();

    assert!(!scenario.user_file(IntegrationScenario::ENTRY).exists());
}

#[test]
fn should_keep_an_entry_the_person_wrote_themselves_when_a_package_is_installed() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_the_package_installed_system_wide()
        .with_an_entry_the_person_wrote("[Desktop Entry]\nName=My browser\n")
        .when_started();

    assert_eq!(scenario.entry_text(), "[Desktop Entry]\nName=My browser\n");
}

#[test]
fn should_not_run_an_appimage_file_that_others_can_replace() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_the_appimage_file_writable_by_others()
        .when_started();

    assert!(!scenario.user_file(IntegrationScenario::ENTRY).exists());
}

#[test]
fn should_not_run_an_appimage_from_a_folder_the_group_can_write_to() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_the_appimage_folder_writable_by_the_group()
        .when_started();

    assert!(!scenario.user_file(IntegrationScenario::ENTRY).exists());
}

#[test]
fn should_not_write_an_entry_for_an_appimage_that_does_not_exist() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_a_missing_appimage()
        .when_started();

    assert!(!scenario.user_file(IntegrationScenario::ENTRY).exists());
}

#[test]
fn should_keep_an_entry_the_person_wrote_themselves() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_an_entry_the_person_wrote("[Desktop Entry]\nName=My browser\n")
        .when_started();

    assert_eq!(scenario.entry_text(), "[Desktop Entry]\nName=My browser\n");
}

#[test]
fn should_not_run_an_appimage_from_a_shared_folder_where_anyone_could_put_a_file_in_its_place() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_the_appimage_folder_open_to_everyone_like_a_shared_temporary_folder()
        .when_started();

    assert!(!scenario.user_file(IntegrationScenario::ENTRY).exists());
}

#[test]
fn should_not_run_an_appimage_when_a_folder_above_its_folder_can_be_renamed_by_the_group() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_a_folder_above_the_appimage_folder_writable_by_the_group()
        .when_started();

    assert!(!scenario.user_file(IntegrationScenario::ENTRY).exists());
}

#[test]
fn should_not_run_an_appimage_whose_link_leads_into_a_folder_the_group_can_write_to() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_the_appimage_reached_through_a_link_to_a_folder_with_mode(0o775)
        .when_started();

    assert!(!scenario.user_file(IntegrationScenario::ENTRY).exists());
}

#[test]
fn should_name_the_file_a_link_leads_to_instead_of_the_link() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_the_appimage_reached_through_a_link_to_a_folder_with_mode(0o755)
        .when_started();

    assert!(scenario.entry_text().contains(&format!("Exec={} %U", scenario.real_appimage().display())));
}

#[test]
fn should_not_run_an_appimage_below_a_shared_folder_even_inside_a_folder_of_its_own() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_the_appimage_in_a_folder_below_a_shared_temporary_folder()
        .when_started();

    assert!(!scenario.user_file(IntegrationScenario::ENTRY).exists());
}

#[test]
fn should_withdraw_its_entry_when_the_appimage_it_names_is_no_longer_safe_to_run() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .when_started_again_after_others_were_allowed_to_write_to_the_appimage();

    assert!(!scenario.user_file(IntegrationScenario::ENTRY).exists());
}

#[test]
fn should_never_withdraw_an_entry_the_person_wrote_themselves() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .with_the_appimage_file_writable_by_others()
        .with_an_entry_the_person_wrote("[Desktop Entry]\nName=My browser\n")
        .when_started();

    assert_eq!(scenario.entry_text(), "[Desktop Entry]\nName=My browser\n");
}

#[test]
fn should_withdraw_the_entry_even_when_a_different_unsafe_appimage_is_the_one_started() {
    let scenario = IntegrationScenario::given_an_appimage_on_a_machine_without_an_installed_package()
        .when_another_unsafe_copy_is_started_after_the_one_the_entry_names_became_unsafe();

    assert!(!scenario.user_file(IntegrationScenario::ENTRY).exists());
}
