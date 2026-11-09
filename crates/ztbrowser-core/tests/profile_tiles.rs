//! The tiles of the profiles popup: who is shown, with which initial and colour, and which one is current.

use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::profile::profile_catalog::ProfileCatalog;
use ztbrowser_core::profile::profile_id::ProfileId;
use ztbrowser_core::profile::profile_tile::ProfileTile;
use ztbrowser_core::profile::profile_tile::ProfileTiles;

struct TilesScenario {
    catalog: ProfileCatalog,
    current: ProfileId,
}

impl TilesScenario {
    fn given_only_the_default_profile() -> Self {
        Self { catalog: ProfileCatalog::default(), current: ProfileId::default_profile() }
    }

    fn given_profiles_named(names: &[&str]) -> Self {
        let mut catalog = ProfileCatalog::default();
        for name in names {
            catalog.create_profile(name);
        }
        Self { catalog, current: ProfileId::default_profile() }
    }

    fn when_the_current_profile_is(mut self, name: &str) -> Self {
        self.current = ProfileId::parse(&name.to_lowercase()).expect("a slug");
        self
    }

    fn tiles(&self) -> Vec<ProfileTile> {
        ProfileTiles::of(&self.catalog, &self.current, &MessageCatalogue::untranslated())
    }
}

#[test]
fn should_show_the_default_profile_alone_when_none_was_created() {
    let tiles = TilesScenario::given_only_the_default_profile().tiles();
    assert_eq!(tiles.len(), 1);
    assert!(tiles[0].is_default && tiles[0].is_current);
}

#[test]
fn should_name_the_default_profile_in_the_interface_language() {
    let tiles = TilesScenario::given_only_the_default_profile().tiles();
    assert_eq!((tiles[0].name.as_str(), tiles[0].initial.as_str()), ("Default", "D"));
}

#[test]
fn should_list_created_profiles_after_the_default_one_in_creation_order() {
    let names: Vec<_> =
        TilesScenario::given_profiles_named(&["Work", "Banking"]).tiles().into_iter().map(|tile| tile.name).collect();
    assert_eq!(names, vec!["Default", "Work", "Banking"]);
}

#[test]
fn should_mark_only_the_current_profile() {
    let tiles = TilesScenario::given_profiles_named(&["Work", "Banking"]).when_the_current_profile_is("work").tiles();
    let current: Vec<_> = tiles.iter().filter(|tile| tile.is_current).map(|tile| tile.name.as_str()).collect();
    assert_eq!(current, vec!["Work"]);
}

#[test]
fn should_take_the_initial_from_the_first_letter_in_upper_case() {
    let tiles = TilesScenario::given_profiles_named(&["work"]).tiles();
    assert_eq!(tiles[1].initial, "W");
}

#[test]
fn should_skip_leading_punctuation_when_choosing_the_initial() {
    let tiles = TilesScenario::given_profiles_named(&["- łódź"]).tiles();
    assert_eq!(tiles[1].initial, "Ł");
}

#[test]
fn should_show_a_question_mark_when_the_name_has_no_letter() {
    let tiles = TilesScenario::given_profiles_named(&["***"]).tiles();
    assert_eq!(tiles[1].initial, "?");
}

#[test]
fn should_give_a_profile_the_same_colour_every_time() {
    let first = TilesScenario::given_profiles_named(&["Work"]).tiles();
    let second = TilesScenario::given_profiles_named(&["Work"]).tiles();
    assert_eq!(first[1].colour_index, second[1].colour_index);
}

#[test]
fn should_keep_every_colour_inside_the_palette() {
    let names: Vec<String> = (0..40).map(|number| format!("profile number {number}")).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let tiles = TilesScenario::given_profiles_named(&refs).tiles();
    assert!(tiles.iter().all(|tile| tile.colour_index < ProfileTiles::COLOUR_COUNT));
}

#[test]
fn should_use_more_than_one_colour_across_many_profiles() {
    let names: Vec<String> = (0..40).map(|number| format!("profile number {number}")).collect();
    let refs: Vec<&str> = names.iter().map(String::as_str).collect();
    let colours: std::collections::BTreeSet<_> =
        TilesScenario::given_profiles_named(&refs).tiles().into_iter().map(|tile| tile.colour_index).collect();
    assert!(colours.len() > 3);
}

#[test]
fn should_name_the_profile_in_the_deletion_question() {
    let heading = ProfileTiles::delete_heading(&MessageCatalogue::untranslated(), "Work");
    assert_eq!(heading, "Delete profile Work?");
}

#[test]
fn should_label_the_tile_that_adds_a_profile_in_the_catalogue_language() {
    assert_eq!(ProfileTiles::add_label(&MessageCatalogue::untranslated()), "Add Profile");
}
