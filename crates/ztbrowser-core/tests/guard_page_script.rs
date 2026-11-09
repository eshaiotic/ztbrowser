//! Guard's page script runs where the page cannot reach it and hides each selector as a rule of its own.

use ztbrowser_core::engine::injection_point::InjectionPoint;
use ztbrowser_core::engine::script_world::ScriptWorld;
use ztbrowser_core::guard::cookie_notice_script::CookieNoticeScript;
use ztbrowser_core::guard::guard_page_script::GuardPageScript;

#[test]
fn should_run_before_the_page_in_an_isolated_world_of_every_frame() {
    let script = GuardPageScript::page_script("/* client */");
    assert_eq!(script.injection_point, InjectionPoint::DocumentCreation);
    assert_eq!(script.world, ScriptWorld::Isolated);
    assert!(script.runs_in_all_frames);
}

#[test]
fn should_place_the_channel_client_in_front_of_the_script() {
    let script = GuardPageScript::page_script("/* client */");
    assert!(script.source.starts_with("/* client */\n"));
}

#[test]
fn should_check_every_selector_before_inserting_it_as_a_rule_of_its_own() {
    let source = GuardPageScript::source();
    assert!(source.contains("CSS.supports(`selector(${selector})`)"));
    assert!(source.contains("sheet.insertRule("));
}

#[test]
fn should_tell_the_cookie_notice_script_of_the_page_world_when_the_shield_is_on() {
    let source = GuardPageScript::source();
    assert!(source.contains(&format!("\"{}\"", CookieNoticeScript::REJECT_EVENT)));
    assert!(source.contains("guard.rejects_cookie_notices(topAddress()"));
}

#[test]
fn should_ask_about_the_top_page_when_it_runs_in_a_frame() {
    assert!(GuardPageScript::source().contains("location.ancestorOrigins"));
}
