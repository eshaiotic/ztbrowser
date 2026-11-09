//! Pages cannot make the engine contact a host before the request filters have seen a request to it.

use ztbrowser_core::engine::injection_point::InjectionPoint;
use ztbrowser_core::engine::resource_hint_blocking_script::ResourceHintBlockingScript;
use ztbrowser_core::engine::script_world::ScriptWorld;

struct HintScenario {
    source: &'static str,
}

impl HintScenario {
    fn given_the_script() -> Self {
        Self { source: ResourceHintBlockingScript::source() }
    }
}

#[test]
fn should_remove_the_hints_that_open_connections_without_a_request() {
    let scenario = HintScenario::given_the_script();
    for hint in ["preconnect", "dns-prefetch", "prefetch", "prerender"] {
        assert!(scenario.source.contains(&format!("'{hint}'")), "{hint} is not neutralised");
    }
}

#[test]
fn should_leave_preload_alone_because_its_requests_pass_through_the_filters() {
    let scenario = HintScenario::given_the_script();
    assert!(!scenario.source.contains("'preload'"));
    assert!(!scenario.source.contains("'modulepreload'"));
}

#[test]
fn should_cover_every_way_a_page_can_set_the_relationship_of_a_link() {
    let scenario = HintScenario::given_the_script();
    for entry_point in ["HTMLLinkElement.prototype", "setAttribute", "relList", "insertAdjacentHTML", "appendChild"] {
        assert!(scenario.source.contains(entry_point), "{entry_point} is not covered");
    }
}

#[test]
fn should_not_touch_class_lists_that_happen_to_hold_the_word_prefetch() {
    let scenario = HintScenario::given_the_script();
    assert!(scenario.source.contains("WeakSet"));
}

#[test]
fn should_run_before_the_page_in_the_pages_own_world_of_every_frame() {
    let script = ResourceHintBlockingScript::page_script();
    assert_eq!(script.injection_point, InjectionPoint::DocumentCreation);
    assert_eq!(script.world, ScriptWorld::Page);
    assert!(script.runs_in_all_frames);
}
