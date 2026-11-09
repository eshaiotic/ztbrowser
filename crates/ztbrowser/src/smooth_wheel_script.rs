//! The page script that animates each step of the mouse wheel (`assets/smooth-wheel/smooth-wheel.js`).
//!
//! The animation's numbers come from the domain type [`SmoothWheelScroll`]; this type only fills them into the
//! script and wraps it as a user script for WebKit. It runs in a script world of its own, so a page cannot see
//! it, and in every frame, so a scrollable frame is smoothed as well.

use webkit6::UserContentInjectedFrames;
use webkit6::UserScript;
use webkit6::UserScriptInjectionTime;
use ztbrowser_core::engine::smooth_wheel_page_script::SmoothWheelPageScript;
use ztbrowser_core::scrolling::smooth_wheel_scroll::SmoothWheelScroll;

pub struct SmoothWheelScript {
    source: String,
}

impl SmoothWheelScript {
    const WORLD: &'static str = "ztbrowser-smooth-wheel";

    pub fn new(scroll: &SmoothWheelScroll) -> Self {
        Self { source: SmoothWheelPageScript::source_for(scroll) }
    }

    pub fn user_script(&self) -> UserScript {
        UserScript::for_world(
            &self.source,
            UserContentInjectedFrames::AllFrames,
            UserScriptInjectionTime::Start,
            Self::WORLD,
            &[],
            &[],
        )
    }
}
