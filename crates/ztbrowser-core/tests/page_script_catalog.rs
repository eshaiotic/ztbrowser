//! The scripts injected into every page of a profile.

use serde_json::json;
use ztbrowser_core::engine::injection_point::InjectionPoint;
use ztbrowser_core::engine::page_script::PageScript;
use ztbrowser_core::engine::page_script_catalog::PageScriptCatalog;
use ztbrowser_core::engine::script_world::ScriptWorld;
use ztbrowser_core::engine::smooth_wheel_page_script::SmoothWheelPageScript;
use ztbrowser_core::scrolling::scrolling_preferences::ScrollingPreferences;

struct ScriptScenario {
    catalog: PageScriptCatalog,
}

impl ScriptScenario {
    fn given_content_scope_script() -> Self {
        Self { catalog: PageScriptCatalog::new().with_content_scope("/* content scope */".to_owned()) }
    }

    fn with_smooth_wheel_turned(self, is_enabled: bool) -> Self {
        let mut preferences = ScrollingPreferences::from_json(&json!({}));
        preferences.set_smooth_wheel(is_enabled);
        Self { catalog: self.catalog.with_smooth_wheel_if_enabled(&preferences) }
    }

    fn script_named(&self, name: &str) -> Option<&PageScript> {
        self.catalog.scripts().iter().find(|script| script.name == name)
    }
}

#[test]
fn should_inject_the_content_scope_script_into_all_frames_of_the_page_world_before_page_scripts() {
    let scenario = ScriptScenario::given_content_scope_script();
    let script = scenario.script_named(PageScriptCatalog::CONTENT_SCOPE_NAME).expect("listed");
    assert!(script.runs_in_all_frames);
    assert_eq!(script.world, ScriptWorld::Page);
    assert_eq!(script.injection_point, InjectionPoint::DocumentCreation);
}

#[test]
fn should_list_the_smooth_wheel_script_when_it_is_enabled() {
    let scenario = ScriptScenario::given_content_scope_script().with_smooth_wheel_turned(true);
    assert!(scenario.script_named(SmoothWheelPageScript::NAME).is_some());
}

#[test]
fn should_not_list_the_smooth_wheel_script_when_it_is_turned_off() {
    let scenario = ScriptScenario::given_content_scope_script().with_smooth_wheel_turned(false);
    assert!(scenario.script_named(SmoothWheelPageScript::NAME).is_none());
}

#[test]
fn should_keep_the_smooth_wheel_script_in_an_isolated_world_the_page_cannot_reach() {
    let scenario = ScriptScenario::given_content_scope_script().with_smooth_wheel_turned(true);
    assert_eq!(scenario.script_named(SmoothWheelPageScript::NAME).expect("listed").world, ScriptWorld::Isolated);
}

#[test]
fn should_fill_the_animation_numbers_into_the_smooth_wheel_script() {
    let scenario = ScriptScenario::given_content_scope_script().with_smooth_wheel_turned(true);
    let source = &scenario.script_named(SmoothWheelPageScript::NAME).expect("listed").source;
    assert!(!source.contains("/*SMOOTH_WHEEL_CONFIGURATION*/{}"));
    assert!(source.contains("pixelsPerNotch"));
}

#[test]
fn should_list_scripts_in_the_order_they_were_added() {
    let scenario = ScriptScenario::given_content_scope_script().with_smooth_wheel_turned(true);
    let names: Vec<&str> = scenario.catalog.scripts().iter().map(|script| script.name.as_str()).collect();
    assert_eq!(names, vec![PageScriptCatalog::CONTENT_SCOPE_NAME, SmoothWheelPageScript::NAME]);
}
