//! Cookie notices are rejected by DuckDuckGo's autoconsent, only ever rejected, and only where Guard asks for it.

use ztbrowser_core::engine::injection_point::InjectionPoint;
use ztbrowser_core::engine::script_world::ScriptWorld;
use ztbrowser_core::guard::cookie_notice_script::CookieNoticeScript;

struct NoticeScenario {
    source: &'static str,
}

impl NoticeScenario {
    fn given_the_script() -> Self {
        Self { source: CookieNoticeScript::source() }
    }
}

#[test]
fn should_run_before_the_page_in_the_pages_world_of_every_frame() {
    let script = CookieNoticeScript::page_script();
    assert_eq!(script.injection_point, InjectionPoint::DocumentCreation);
    assert_eq!(script.world, ScriptWorld::Page);
    assert!(script.runs_in_all_frames);
}

#[test]
fn should_start_only_when_guard_dispatches_the_event_for_a_protected_site() {
    let source = NoticeScenario::given_the_script().source;
    assert!(source.contains(&format!("\"{}\"", CookieNoticeScript::REJECT_EVENT)));
    assert!(source.contains("{once:!0}"), "the rejection should start once per document");
}

#[test]
fn should_reject_and_never_accept_a_notice() {
    let source = NoticeScenario::given_the_script().source;
    assert!(source.contains("autoAction:\"optOut\""));
    assert!(source.contains("heuristicMode:\"reject\""));
    assert!(!source.contains("autoAction:\"optIn\""));
    assert!(!source.contains("heuristicMode:\"tier"));
}

#[test]
fn should_know_the_consent_managers_of_the_big_sites() {
    let source = NoticeScenario::given_the_script().source;
    for manager in ["Didomi", "Cookiebot", "onetrust-banner-sdk", "facebook", "meta-cookie-dialog"] {
        assert!(source.contains(manager), "no rule for {manager}");
    }
}

#[test]
fn should_name_nothing_on_the_page_in_the_global_scope() {
    let source = NoticeScenario::given_the_script().source;
    assert!(source.starts_with("\"use strict\";(()=>{"), "the bundle should be one closure");
    assert!(source.ends_with("})();") || source.trim_end().ends_with("})();"));
}
