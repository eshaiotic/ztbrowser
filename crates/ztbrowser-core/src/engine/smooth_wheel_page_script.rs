//! The page script that animates each step of the mouse wheel (`assets/smooth-wheel/smooth-wheel.js`).

use crate::engine::injection_point::InjectionPoint;
use crate::engine::page_script::PageScript;
use crate::engine::script_world::ScriptWorld;
use crate::scrolling::smooth_wheel_scroll::SmoothWheelScroll;

pub struct SmoothWheelPageScript;

impl SmoothWheelPageScript {
    pub const NAME: &'static str = "ztbrowser-smooth-wheel";
    const TEMPLATE: &'static str = include_str!("../../../../assets/smooth-wheel/smooth-wheel.js");

    /// The script with the animation's numbers filled in.
    #[must_use]
    pub fn source_for(scroll: &SmoothWheelScroll) -> String {
        Self::TEMPLATE.replace("/*SMOOTH_WHEEL_CONFIGURATION*/{}", &scroll.script_configuration().to_string())
    }

    /// Runs in an isolated world and in every frame, so a scrollable frame is smoothed as well.
    #[must_use]
    pub fn for_scroll(scroll: &SmoothWheelScroll) -> PageScript {
        PageScript {
            name: Self::NAME.to_owned(),
            source: Self::source_for(scroll),
            injection_point: InjectionPoint::DocumentCreation,
            runs_in_all_frames: true,
            world: ScriptWorld::Isolated,
        }
    }
}
