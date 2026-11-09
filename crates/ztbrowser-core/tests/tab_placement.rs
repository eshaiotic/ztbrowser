//! Which side of the page a profile's tabs sit on, kept per profile.

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::profile::tab_placement::TabPlacement;
use ztbrowser_core::profile::tab_placement::TabPlacementPreferences;

struct PlacementScenario {
    preferences: TabPlacementPreferences,
}

impl PlacementScenario {
    fn given_nothing_stored() -> Self {
        Self { preferences: TabPlacementPreferences::from_json(&Value::Null) }
    }

    fn given_stored(document: &Value) -> Self {
        Self { preferences: TabPlacementPreferences::from_json(document) }
    }

    fn when_tabs_are_moved_to(mut self, placement: TabPlacement) -> Self {
        self.preferences.set_placement(placement);
        self
    }

    fn after_saving_and_loading(self) -> Self {
        Self { preferences: TabPlacementPreferences::from_json(&self.preferences.to_json()) }
    }

    fn then_tabs_sit(&self, expected: TabPlacement) {
        assert_eq!(self.preferences.placement(), expected);
    }
}

#[test]
fn should_put_the_tabs_on_top_by_default() {
    PlacementScenario::given_nothing_stored().then_tabs_sit(TabPlacement::Top);
}

#[test]
fn should_remember_every_placement_after_saving() {
    for placement in TabPlacement::ALL {
        PlacementScenario::given_nothing_stored()
            .when_tabs_are_moved_to(placement)
            .after_saving_and_loading()
            .then_tabs_sit(placement);
    }
}

#[test]
fn should_keep_the_top_when_the_stored_name_is_unknown() {
    PlacementScenario::given_stored(&json!({"placement": "diagonal"})).then_tabs_sit(TabPlacement::Top);
}

#[test]
fn should_keep_the_top_when_the_stored_value_is_not_text() {
    PlacementScenario::given_stored(&json!({"placement": 3})).then_tabs_sit(TabPlacement::Top);
}

#[test]
fn should_lay_tabs_side_by_side_only_along_the_top_and_bottom_edges() {
    let horizontal: Vec<_> = TabPlacement::ALL.into_iter().filter(|placement| placement.is_horizontal()).collect();
    assert_eq!(horizontal, vec![TabPlacement::Top, TabPlacement::Bottom]);
}

#[test]
fn should_give_every_placement_its_own_stored_name() {
    let names: std::collections::BTreeSet<_> = TabPlacement::ALL.into_iter().map(TabPlacement::id).collect();
    assert_eq!(names.len(), 4);
}

#[test]
fn should_name_the_choices_in_the_language_of_the_catalogue() {
    let catalogue = MessageCatalogue::untranslated();
    assert_eq!(TabPlacementPreferences::title(&catalogue), "Tab Position");
    assert_eq!(TabPlacement::Left.label(&catalogue), "Left");
}
