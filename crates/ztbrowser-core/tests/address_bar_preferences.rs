//! How much of the toolbar the address bar takes, kept per profile.

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::profile::address_bar_preferences::AddressBarPreferences;

struct PreferencesScenario {
    preferences: AddressBarPreferences,
}

impl PreferencesScenario {
    fn given_nothing_stored() -> Self {
        Self { preferences: AddressBarPreferences::from_json(&Value::Null) }
    }

    fn given_stored(document: &Value) -> Self {
        Self { preferences: AddressBarPreferences::from_json(document) }
    }

    fn when_wide_is_turned(mut self, is_wide: bool) -> Self {
        self.preferences.set_wide(is_wide);
        self
    }

    fn after_saving_and_loading(self) -> Self {
        Self { preferences: AddressBarPreferences::from_json(&self.preferences.to_json()) }
    }

    fn then_address_bar_is_wide(&self, expected: bool) {
        assert_eq!(self.preferences.is_wide(), expected);
    }
}

#[test]
fn should_stretch_the_address_bar_over_the_toolbar_by_default() {
    PreferencesScenario::given_nothing_stored().then_address_bar_is_wide(true);
}

#[test]
fn should_remember_that_the_short_address_bar_was_chosen() {
    PreferencesScenario::given_nothing_stored()
        .when_wide_is_turned(false)
        .after_saving_and_loading()
        .then_address_bar_is_wide(false);
}

#[test]
fn should_keep_the_default_when_the_stored_value_is_not_a_switch() {
    PreferencesScenario::given_stored(&json!({"wide": "no"})).then_address_bar_is_wide(true);
}

#[test]
fn should_describe_the_choice_in_the_language_of_the_catalogue() {
    let catalogue = MessageCatalogue::untranslated();
    assert_eq!(AddressBarPreferences::title(&catalogue), "Wide Address Bar");
}
