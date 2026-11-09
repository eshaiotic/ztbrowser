//! The person's choice about smooth mouse-wheel scrolling, kept per profile.

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::scrolling::scrolling_preferences::ScrollingPreferences;

struct PreferencesScenario {
    preferences: ScrollingPreferences,
}

impl PreferencesScenario {
    fn given_nothing_stored() -> Self {
        Self { preferences: ScrollingPreferences::from_json(&Value::Null) }
    }

    fn given_stored(document: &Value) -> Self {
        Self { preferences: ScrollingPreferences::from_json(document) }
    }

    fn when_smooth_wheel_is_turned(mut self, is_enabled: bool) -> Self {
        self.preferences.set_smooth_wheel(is_enabled);
        self
    }

    fn after_saving_and_loading(self) -> Self {
        Self { preferences: ScrollingPreferences::from_json(&self.preferences.to_json()) }
    }

    fn then_wheel_is_smooth(&self, expected: bool) {
        assert_eq!(self.preferences.smooth_wheel(), expected);
    }
}

#[test]
fn should_smooth_the_wheel_by_default() {
    PreferencesScenario::given_nothing_stored().then_wheel_is_smooth(true);
}

#[test]
fn should_remember_that_smoothing_was_turned_off() {
    PreferencesScenario::given_nothing_stored()
        .when_smooth_wheel_is_turned(false)
        .after_saving_and_loading()
        .then_wheel_is_smooth(false);
}

#[test]
fn should_remember_that_smoothing_was_turned_back_on() {
    PreferencesScenario::given_nothing_stored()
        .when_smooth_wheel_is_turned(false)
        .when_smooth_wheel_is_turned(true)
        .after_saving_and_loading()
        .then_wheel_is_smooth(true);
}

#[test]
fn should_fall_back_to_smooth_when_the_stored_value_is_not_a_boolean() {
    PreferencesScenario::given_stored(&json!({"smoothWheel": "no"})).then_wheel_is_smooth(true);
}
