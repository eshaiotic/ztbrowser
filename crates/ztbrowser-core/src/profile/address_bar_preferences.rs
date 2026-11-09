//! Whether the address bar takes all the room the toolbar leaves it, or stays short and centred (`address-bar.json`).

use serde_json::Value;
use serde_json::json;

use crate::document::lenient_json::LenientJson;
use crate::language::message_catalogue::MessageCatalogue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddressBarPreferences {
    is_wide: bool,
}

impl Default for AddressBarPreferences {
    /// The address bar is wide until the person asks for the short one.
    fn default() -> Self {
        Self { is_wide: true }
    }
}

impl AddressBarPreferences {
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let defaults = Self::default();
        Self { is_wide: LenientJson::new(document).field("wide").as_bool().unwrap_or(defaults.is_wide) }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({ "wide": self.is_wide })
    }

    #[must_use]
    pub fn is_wide(&self) -> bool {
        self.is_wide
    }

    pub fn set_wide(&mut self, is_wide: bool) {
        self.is_wide = is_wide;
    }

    /// The name of the choice in the preferences window.
    #[must_use]
    pub fn title(catalogue: &MessageCatalogue) -> &str {
        catalogue.text("Wide Address Bar")
    }

    /// What the choice does, in a sentence.
    #[must_use]
    pub fn description(catalogue: &MessageCatalogue) -> &str {
        catalogue.text("The address bar fills all the room in the toolbar; off keeps it short and centred")
    }
}
