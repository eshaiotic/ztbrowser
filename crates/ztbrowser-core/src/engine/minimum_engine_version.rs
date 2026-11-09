//! The oldest engine version ZTBrowser accepts, raised with every release (spec §7).
//!
//! A normal window only warns, because an out-of-date engine is still usable for most people; the Tor window
//! refuses, because anonymity is its whole purpose and a known engine bug can reveal who the person is.

use crate::engine::engine_verdict::EngineVerdict;
use crate::engine::engine_version::EngineVersion;
use crate::engine::window_kind::WindowKind;

#[derive(Debug, Clone, Copy)]
pub struct MinimumEngineVersion {
    oldest_accepted: EngineVersion,
}

impl MinimumEngineVersion {
    /// `QtWebEngine` 6.12 ships Chromium 140.0.7339; raise this with each release after checking Qt's
    /// security announcements.
    const CURRENT_FLOOR: &'static str = "140.0.7339.0";

    #[must_use]
    pub fn new(oldest_accepted: EngineVersion) -> Self {
        Self { oldest_accepted }
    }

    /// # Panics
    /// Never: the floor is a literal that a test parses.
    #[must_use]
    pub fn current() -> Self {
        Self::new(EngineVersion::parse(Self::CURRENT_FLOOR).expect("the floor is a valid version"))
    }

    #[must_use]
    pub fn verdict(&self, installed: &EngineVersion, window: WindowKind) -> EngineVerdict {
        match (installed >= &self.oldest_accepted, window) {
            (true, _) => EngineVerdict::Run,
            (false, WindowKind::Normal) => EngineVerdict::Warn,
            (false, WindowKind::Tor) => EngineVerdict::Refuse,
        }
    }
}
