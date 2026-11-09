//! Isolated browsing profiles: each one keeps its own cookies, storage, cache and history on disk.

use std::path::Path;
use std::path::PathBuf;

use serde_json::json;
use ztbrowser_core::profile::profile_catalog::ProfileCatalog;
use ztbrowser_core::profile::profile_directories::ProfileDirectories;
use ztbrowser_core::profile::profile_id::ProfileId;

struct ProfileCatalogScenario {
    catalog: ProfileCatalog,
}

impl ProfileCatalogScenario {
    fn given_fresh_installation() -> Self {
        Self { catalog: ProfileCatalog::default() }
    }

    fn given_stored_catalog(document: &serde_json::Value) -> Self {
        Self { catalog: ProfileCatalog::from_json(document) }
    }

    fn after_creating_profile(mut self, name: &str) -> Self {
        self.catalog.create_profile(name);
        self
    }

    fn when_creating_profile(&mut self, name: &str) -> ProfileId {
        self.catalog.create_profile(name)
    }

    fn profile_names(&self) -> Vec<(String, String)> {
        self.catalog.profiles().map(|profile| (profile.id.as_str().to_owned(), profile.name.clone())).collect()
    }
}

#[test]
fn should_always_contain_the_default_profile() {
    let scenario = ProfileCatalogScenario::given_fresh_installation();

    assert_eq!(scenario.profile_names(), [("default".to_owned(), "Default".to_owned())]);
}

#[test]
fn should_create_profile_with_identifier_derived_from_its_name() {
    let mut scenario = ProfileCatalogScenario::given_fresh_installation();

    assert_eq!(scenario.when_creating_profile("Praca").as_str(), "praca");
}

#[test]
fn should_keep_identifiers_unique_when_names_collide() {
    let mut scenario = ProfileCatalogScenario::given_fresh_installation().after_creating_profile("Praca");

    assert_eq!(scenario.when_creating_profile("praca").as_str(), "praca_2");
}

#[test]
fn should_reserve_web_app_identifiers_for_web_apps() {
    let mut scenario = ProfileCatalogScenario::given_fresh_installation();

    assert_eq!(scenario.when_creating_profile("app mbank").as_str(), "profile_app_mbank");
}

#[test]
fn should_remove_profile_but_never_the_default_one() {
    let mut scenario = ProfileCatalogScenario::given_fresh_installation().after_creating_profile("Praca");

    scenario.catalog.remove_profile(&ProfileId::parse("praca").unwrap());
    scenario.catalog.remove_profile(&ProfileId::default_profile());

    assert_eq!(scenario.profile_names(), [("default".to_owned(), "Default".to_owned())]);
}

#[test]
fn should_survive_a_save_and_load_round_trip() {
    let saved = ProfileCatalogScenario::given_fresh_installation().after_creating_profile("Praca").catalog.to_json();

    let restored = ProfileCatalogScenario::given_stored_catalog(&saved);

    assert_eq!(
        restored.profile_names(),
        [("default".to_owned(), "Default".to_owned()), ("praca".to_owned(), "Praca".to_owned())]
    );
}

#[test]
fn should_skip_stored_profiles_with_unsafe_identifiers() {
    let scenario = ProfileCatalogScenario::given_stored_catalog(&json!({"profiles": [
        {"id": "../../home", "name": "Evil"},
        {"id": "praca", "name": "Praca"}
    ]}));

    assert_eq!(
        scenario.profile_names(),
        [("default".to_owned(), "Default".to_owned()), ("praca".to_owned(), "Praca".to_owned())]
    );
}

#[test]
fn should_place_each_profile_in_its_own_data_and_cache_directory() {
    let directories =
        ProfileDirectories::new(Path::new("/home/me/.local/share/ztbrowser"), Path::new("/home/me/.cache/ztbrowser"));

    let praca = directories.for_profile(&ProfileId::parse("praca").unwrap());

    assert_eq!(
        (praca.data, praca.cache),
        (
            PathBuf::from("/home/me/.local/share/ztbrowser/profiles/praca"),
            PathBuf::from("/home/me/.cache/ztbrowser/profiles/praca")
        )
    );
}
