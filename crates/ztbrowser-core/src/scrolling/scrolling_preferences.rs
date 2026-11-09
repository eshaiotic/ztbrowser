//! The person's choices about scrolling, kept per profile (`scrolling.json`).

use serde_json::Value;
use serde_json::json;

use crate::document::lenient_json::LenientJson;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollingPreferences {
    smooth_wheel: bool,
}

impl Default for ScrollingPreferences {
    /// Smooth wheel scrolling is on until the person turns it off.
    fn default() -> Self {
        Self { smooth_wheel: true }
    }
}

impl ScrollingPreferences {
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let defaults = Self::default();
        Self {
            smooth_wheel: LenientJson::new(document).field("smoothWheel").as_bool().unwrap_or(defaults.smooth_wheel),
        }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({ "smoothWheel": self.smooth_wheel })
    }

    /// Whether each step of the mouse wheel is animated by ZTBrowser.
    #[must_use]
    pub fn smooth_wheel(&self) -> bool {
        self.smooth_wheel
    }

    pub fn set_smooth_wheel(&mut self, is_enabled: bool) {
        self.smooth_wheel = is_enabled;
    }
}
