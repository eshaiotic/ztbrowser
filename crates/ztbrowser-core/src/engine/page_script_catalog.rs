//! The scripts to inject for one profile, in the order they must run.

use crate::engine::injection_point::InjectionPoint;
use crate::engine::page_script::PageScript;
use crate::engine::script_world::ScriptWorld;
use crate::engine::smooth_wheel_page_script::SmoothWheelPageScript;
use crate::scrolling::scrolling_preferences::ScrollingPreferences;
use crate::scrolling::smooth_wheel_scroll::SmoothWheelScroll;

#[derive(Debug, Clone, Default)]
pub struct PageScriptCatalog {
    scripts: Vec<PageScript>,
}

impl PageScriptCatalog {
    pub const CONTENT_SCOPE_NAME: &'static str = "ztbrowser-content-scope";

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// DuckDuckGo's Content Scope Scripts, already filled in (see `ContentScopeScript`). They change what the
    /// page sees, so they run in the page's own world, in every frame, before any page script.
    #[must_use]
    pub fn with_content_scope(mut self, source: String) -> Self {
        self.scripts.push(PageScript {
            name: Self::CONTENT_SCOPE_NAME.to_owned(),
            source,
            injection_point: InjectionPoint::DocumentCreation,
            runs_in_all_frames: true,
            world: ScriptWorld::Page,
        });
        self
    }

    /// Adds the smooth-wheel script when the person's scrolling preferences ask for it.
    #[must_use]
    pub fn with_smooth_wheel_if_enabled(mut self, preferences: &ScrollingPreferences) -> Self {
        if preferences.smooth_wheel() {
            self.scripts.push(SmoothWheelPageScript::for_scroll(&SmoothWheelScroll::chromium_like()));
        }
        self
    }

    #[must_use]
    pub fn scripts(&self) -> &[PageScript] {
        &self.scripts
    }
}
