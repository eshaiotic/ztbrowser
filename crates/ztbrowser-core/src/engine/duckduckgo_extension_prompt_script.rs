//! The page script that hides DuckDuckGo's "add our extension" offer (`assets/site-adjustments/`).
//!
//! DuckDuckGo's pages recognise Chromium by its user agent and offer it the extension. ZTBrowser has the same
//! protections built in and runs no extensions, so the offer is hidden with a style sheet rather than the page
//! being told something untrue (a DuckDuckGo user agent, or an installed extension).

use crate::engine::injection_point::InjectionPoint;
use crate::engine::page_script::PageScript;
use crate::engine::script_world::ScriptWorld;

pub struct DuckDuckGoExtensionPromptScript;

impl DuckDuckGoExtensionPromptScript {
    pub const NAME: &'static str = "ztbrowser-duckduckgo-extension-prompt";
    const SOURCE: &'static str = include_str!("../../../../assets/site-adjustments/duckduckgo-extension-prompt.js");

    #[must_use]
    pub fn source() -> &'static str {
        Self::SOURCE
    }

    /// Runs in an isolated world and in every frame; the script itself acts only on duckduckgo.com.
    #[must_use]
    pub fn page_script() -> PageScript {
        PageScript {
            name: Self::NAME.to_owned(),
            source: Self::SOURCE.to_owned(),
            injection_point: InjectionPoint::DocumentCreation,
            runs_in_all_frames: true,
            world: ScriptWorld::Isolated,
        }
    }
}
