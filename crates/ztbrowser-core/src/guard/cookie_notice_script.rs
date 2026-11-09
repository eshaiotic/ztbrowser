//! The page script that rejects cookie notices (`assets/autoconsent/`, DuckDuckGo's autoconsent library).
//!
//! The library presses the *reject* button of the consent dialogs it recognises, so a site never records the
//! person's agreement and the dialog never stays in the way. It needs the page's own world, because some of its
//! rules call the consent manager's JavaScript interface (for example `Didomi.setUserDisagreeToAll()`), but it
//! does nothing until the isolated Guard script, which knows whether the shield is on for the site, dispatches
//! [`CookieNoticeScript::REJECT_EVENT`] on the document.

use crate::engine::injection_point::InjectionPoint;
use crate::engine::page_script::PageScript;
use crate::engine::script_world::ScriptWorld;

pub struct CookieNoticeScript;

impl CookieNoticeScript {
    pub const NAME: &'static str = "ztbrowser-cookie-notices";
    /// What Guard dispatches on a document to start the rejection there; the entry point of the bundle listens for it.
    pub const REJECT_EVENT: &'static str = "ztbrowser-reject-cookie-notices";
    const SOURCE: &'static str = include_str!("../../../../assets/autoconsent/autoconsent.js");

    #[must_use]
    pub fn source() -> &'static str {
        Self::SOURCE
    }

    /// Runs in the page's own world and in every frame (consent managers often live in a frame of their own).
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
