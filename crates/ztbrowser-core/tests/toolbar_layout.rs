//! Which toolbar buttons a profile shows.

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::profile::toolbar_layout::ToolbarButton;
use ztbrowser_core::profile::toolbar_layout::ToolbarLayout;

struct ToolbarLayoutScenario {
    layout: ToolbarLayout,
}

impl ToolbarLayoutScenario {
    fn given_stored(document: &Value) -> Self {
        Self { layout: ToolbarLayout::from_json(document) }
    }

    fn when_switched(mut self, button: ToolbarButton, is_shown: bool) -> Self {
        self.layout.set_shown(button, is_shown);
        self
    }

    fn after_saving_and_loading(self) -> Self {
        Self { layout: ToolbarLayout::from_json(&self.layout.to_json()) }
    }

    fn then_shows_every_button(&self) {
        let hidden: Vec<ToolbarButton> =
            ToolbarButton::ALL.into_iter().filter(|button| !self.layout.shows(*button)).collect();
        assert_eq!(hidden, Vec::new());
    }

    fn then_hides_only(&self, expected: ToolbarButton) {
        let hidden: Vec<ToolbarButton> =
            ToolbarButton::ALL.into_iter().filter(|button| !self.layout.shows(*button)).collect();
        assert_eq!(hidden, vec![expected]);
    }
}

#[test]
fn should_show_every_button_until_the_profile_hides_one() {
    ToolbarLayoutScenario::given_stored(&Value::Null).then_shows_every_button();
}

#[test]
fn should_keep_a_hidden_button_hidden_across_restarts() {
    ToolbarLayoutScenario::given_stored(&Value::Null)
        .when_switched(ToolbarButton::Reload, false)
        .after_saving_and_loading()
        .then_hides_only(ToolbarButton::Reload);
}

#[test]
fn should_show_a_button_again_once_it_is_switched_back_on() {
    ToolbarLayoutScenario::given_stored(&json!({"hiddenButtons": ["torWindow"]}))
        .when_switched(ToolbarButton::TorWindow, true)
        .then_shows_every_button();
}

#[test]
fn should_show_buttons_the_stored_layout_does_not_mention() {
    ToolbarLayoutScenario::given_stored(&json!({"hiddenButtons": ["newTab"]})).then_hides_only(ToolbarButton::NewTab);
}

#[test]
fn should_ignore_buttons_it_does_not_offer() {
    ToolbarLayoutScenario::given_stored(&json!({"hiddenButtons": ["bookmarks", 7]})).then_shows_every_button();
}

#[test]
fn should_give_every_button_its_own_stored_name() {
    let mut ids: Vec<&str> = ToolbarButton::ALL.into_iter().map(ToolbarButton::id).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), ToolbarButton::ALL.len());
}
