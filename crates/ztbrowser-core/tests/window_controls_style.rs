//! How a profile's window buttons look, kept per profile.

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::profile::window_controls_style::WindowControlsPreferences;
use ztbrowser_core::profile::window_controls_style::WindowControlsStyle;

struct ControlsScenario {
    preferences: WindowControlsPreferences,
}

impl ControlsScenario {
    fn given_nothing_stored() -> Self {
        Self { preferences: WindowControlsPreferences::from_json(&Value::Null) }
    }

    fn given_stored(document: &Value) -> Self {
        Self { preferences: WindowControlsPreferences::from_json(document) }
    }

    fn when_style_is_set_to(mut self, style: WindowControlsStyle) -> Self {
        self.preferences.set_style(style);
        self
    }

    fn after_saving_and_loading(self) -> Self {
        Self { preferences: WindowControlsPreferences::from_json(&self.preferences.to_json()) }
    }

    fn then_style_is(&self, expected: WindowControlsStyle) {
        assert_eq!(self.preferences.style(), expected);
    }
}

#[test]
fn should_draw_the_libadwaita_buttons_on_the_right_by_default() {
    let scenario = ControlsScenario::given_nothing_stored();
    scenario.then_style_is(WindowControlsStyle::Default);
    assert!(!scenario.preferences.style().is_on_the_left());
}

#[test]
fn should_remember_the_macos_style_after_saving() {
    ControlsScenario::given_nothing_stored()
        .when_style_is_set_to(WindowControlsStyle::MacOs)
        .after_saving_and_loading()
        .then_style_is(WindowControlsStyle::MacOs);
}

#[test]
fn should_put_the_macos_buttons_on_the_left() {
    assert!(WindowControlsStyle::MacOs.is_on_the_left());
}

#[test]
fn should_keep_the_default_when_the_stored_style_is_unknown() {
    ControlsScenario::given_stored(&json!({"style": "windows95"})).then_style_is(WindowControlsStyle::Default);
}

#[test]
fn should_name_the_choices_in_the_language_of_the_catalogue() {
    let catalogue = MessageCatalogue::untranslated();
    assert_eq!(WindowControlsPreferences::title(&catalogue), "Window Buttons");
    assert_eq!(WindowControlsStyle::MacOs.label(&catalogue), "macOS");
}
