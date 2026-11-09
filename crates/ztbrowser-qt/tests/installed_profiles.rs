//! The profiles on disk: creating one, listing it, and deleting it with its data.

use std::fs;
use std::path::PathBuf;

use tempfile::TempDir;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::profile::profile_id::ProfileId;
use ztbrowser_qt::installation_directories::InstallationDirectories;
use ztbrowser_qt::installed_profiles::DeleteError;
use ztbrowser_qt::installed_profiles::InstalledProfiles;
use ztbrowser_qt::single_instance::Instance;
use ztbrowser_qt::single_instance::SingleInstance;

struct ProfilesScenario {
    root: TempDir,
    directories: InstallationDirectories,
}

impl ProfilesScenario {
    fn given_a_fresh_installation() -> Self {
        let root = TempDir::new().expect("temporary directory");
        let directories = InstallationDirectories::under(&root.path().join("data"), &root.path().join("cache"));
        Self { root, directories }
    }

    fn runtime(&self) -> PathBuf {
        self.root.path().join("run")
    }

    fn profiles(&self) -> InstalledProfiles<'_> {
        InstalledProfiles::new(&self.directories, self.runtime())
    }

    fn when_a_profile_is_created(&self, name: &str) -> Option<ProfileId> {
        self.profiles().create(name).expect("catalogue written")
    }

    fn when_its_files_exist(&self, id: &ProfileId) {
        let location = self.directories.profile(id);
        fs::create_dir_all(&location.data).expect("data directory");
        fs::write(location.data.join("cookies"), "x").expect("a file");
        fs::create_dir_all(&location.cache).expect("cache directory");
    }

    fn names(&self) -> Vec<String> {
        self.profiles()
            .tiles(&ProfileId::default_profile(), &MessageCatalogue::untranslated())
            .into_iter()
            .map(|tile| tile.name)
            .collect()
    }
}

#[test]
fn should_list_only_the_default_profile_in_a_fresh_installation() {
    assert_eq!(ProfilesScenario::given_a_fresh_installation().names(), vec!["Default"]);
}

#[test]
fn should_list_a_created_profile_after_the_default_one() {
    let scenario = ProfilesScenario::given_a_fresh_installation();
    scenario.when_a_profile_is_created("Work");
    assert_eq!(scenario.names(), vec!["Default", "Work"]);
}

#[test]
fn should_refuse_a_blank_profile_name() {
    let scenario = ProfilesScenario::given_a_fresh_installation();
    assert!(scenario.when_a_profile_is_created("   ").is_none());
    assert_eq!(scenario.names(), vec!["Default"]);
}

#[test]
fn should_cut_an_over_long_profile_name() {
    let scenario = ProfilesScenario::given_a_fresh_installation();
    scenario.when_a_profile_is_created(&"x".repeat(200));
    assert_eq!(scenario.names()[1].chars().count(), 40);
}

#[test]
fn should_delete_a_profile_with_its_files() {
    let scenario = ProfilesScenario::given_a_fresh_installation();
    let id = scenario.when_a_profile_is_created("Work").expect("created");
    scenario.when_its_files_exist(&id);
    scenario.profiles().delete(&id).expect("deleted");
    assert!(!scenario.directories.profile(&id).data.exists());
    assert_eq!(scenario.names(), vec!["Default"]);
}

#[test]
fn should_never_delete_the_default_profile() {
    let scenario = ProfilesScenario::given_a_fresh_installation();
    let result = scenario.profiles().delete(&ProfileId::default_profile());
    assert!(matches!(result, Err(DeleteError::IsDefault)));
}

#[test]
fn should_keep_a_profile_that_is_open_in_a_window() {
    let scenario = ProfilesScenario::given_a_fresh_installation();
    let id = scenario.when_a_profile_is_created("Work").expect("created");
    scenario.when_its_files_exist(&id);
    let Ok(Instance::Primary(_held)) =
        SingleInstance::acquire(&scenario.runtime(), &id.application_id().to_string(), &[])
    else {
        panic!("the profile should have been free");
    };
    let result = scenario.profiles().delete(&id);
    assert!(matches!(result, Err(DeleteError::StillRunning(name)) if name == "Work"));
    assert!(scenario.directories.profile(&id).data.exists());
}

#[test]
fn should_treat_deleting_an_unknown_profile_as_done() {
    let scenario = ProfilesScenario::given_a_fresh_installation();
    let unknown = ProfileId::parse("ghost").expect("a slug");
    assert!(scenario.profiles().delete(&unknown).is_ok());
}
