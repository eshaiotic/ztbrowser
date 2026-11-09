//! The page script that keeps pages from opening connections with resource hints (`assets/guard/no-resource-hints.js`).
//!
//! `preconnect` and `dns-prefetch` hints make the engine connect to a host before any request exists, so the
//! request filters cannot refuse the connection. Measured on bbc.com, nine advertising and tracking hosts were
//! contacted this way while every request to them was blocked.

use crate::engine::injection_point::InjectionPoint;
use crate::engine::page_script::PageScript;
use crate::engine::script_world::ScriptWorld;

pub struct ResourceHintBlockingScript;

impl ResourceHintBlockingScript {
    pub const NAME: &'static str = "ztbrowser-no-resource-hints";
    const SOURCE: &'static str = include_str!("../../../../assets/guard/no-resource-hints.js");

    #[must_use]
    pub fn source() -> &'static str {
        Self::SOURCE
    }

    /// Runs in the page's own world, because it must change what the page's scripts call, and in every frame.
    #[must_use]
    pub fn page_script() -> PageScript {
        PageScript {
            name: Self::NAME.to_owned(),
            source: Self::SOURCE.to_owned(),
            injection_point: InjectionPoint::DocumentCreation,
            runs_in_all_frames: true,
            world: ScriptWorld::Page,
        }
    }
}
