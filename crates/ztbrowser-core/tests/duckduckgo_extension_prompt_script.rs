//! DuckDuckGo's offer of an extension is hidden on its own pages and nowhere else.

use ztbrowser_core::engine::duckduckgo_extension_prompt_script::DuckDuckGoExtensionPromptScript;
use ztbrowser_core::engine::injection_point::InjectionPoint;
use ztbrowser_core::engine::script_world::ScriptWorld;

struct PromptScenario {
    source: &'static str,
}

impl PromptScenario {
    fn given_the_script() -> Self {
        Self { source: DuckDuckGoExtensionPromptScript::source() }
    }
}

#[test]
fn should_act_only_on_duckduckgo_hosts() {
    let scenario = PromptScenario::given_the_script();
    assert!(scenario.source.contains("host !== 'duckduckgo.com' && !host.endsWith('.duckduckgo.com')"));
}

#[test]
fn should_hide_the_offer_with_a_style_sheet_instead_of_claiming_an_extension_is_installed() {
    let scenario = PromptScenario::given_the_script();
    assert!(scenario.source.contains("display: none !important"));
    assert!(!scenario.source.contains("platform"));
}

#[test]
fn should_run_before_the_page_in_an_isolated_world_of_every_frame() {
    let script = DuckDuckGoExtensionPromptScript::page_script();
    assert_eq!(script.injection_point, InjectionPoint::DocumentCreation);
    assert_eq!(script.world, ScriptWorld::Isolated);
    assert!(script.runs_in_all_frames);
}
