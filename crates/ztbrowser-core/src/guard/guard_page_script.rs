//! The page script that hides what Guard's lists name (`assets/guard/page-hiding.js`).

use crate::engine::injection_point::InjectionPoint;
use crate::engine::page_script::PageScript;
use crate::engine::script_world::ScriptWorld;

pub struct GuardPageScript;

impl GuardPageScript {
    /// Where Qt keeps its web-channel client, which the script needs in front of it.
    pub const CHANNEL_CLIENT_RESOURCE: &'static str = ":/qtwebchannel/qwebchannel.js";
    pub const NAME: &'static str = "ztbrowser-guard-page-hiding";
    const SOURCE: &'static str = include_str!("../../../../assets/guard/page-hiding.js");

    /// The script itself. It needs Qt's `QWebChannel` client in front of it and a channel object named `guard`.
    #[must_use]
    pub fn source() -> &'static str {
        Self::SOURCE
    }

    /// Runs before the page in an isolated world of every frame, so the page can neither see nor call it.
    #[must_use]
    pub fn page_script(channel_client: &str) -> PageScript {
        PageScript {
            name: Self::NAME.to_owned(),
            source: format!("{channel_client}\n{}", Self::SOURCE),
            injection_point: InjectionPoint::DocumentCreation,
            runs_in_all_frames: true,
            world: ScriptWorld::Isolated,
        }
    }
}
