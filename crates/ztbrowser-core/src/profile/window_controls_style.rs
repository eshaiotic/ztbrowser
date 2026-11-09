//! How the window's minimise, maximise and close buttons look and where they sit (`window-controls.json`): the
//! three round buttons on the right as libadwaita draws them, or three coloured dots on the left as macOS does.

use serde_json::Value;
use serde_json::json;

use crate::document::lenient_json::LenientJson;
use crate::language::message_catalogue::MessageCatalogue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowControlsStyle {
    #[default]
    Default,
    MacOs,
}

impl WindowControlsStyle {
    pub const ALL: [Self; 2] = [Self::Default, Self::MacOs];

    /// A stable identifier (`macos`), for storage and for the interface.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::MacOs => "macos",
        }
    }

    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|style| style.id() == id)
    }

    /// Whether the buttons sit at the left end of the header bar instead of the right.
    #[must_use]
    pub fn is_on_the_left(self) -> bool {
        self == Self::MacOs
    }

    /// The name of the choice in the preferences window.
    #[must_use]
    pub fn label(self, catalogue: &MessageCatalogue) -> &str {
        match self {
            Self::Default => catalogue.text("Default"),
            Self::MacOs => catalogue.text("macOS"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowControlsPreferences {
    style: WindowControlsStyle,
}

impl WindowControlsPreferences {
    /// An unknown or missing stored value keeps the default style.
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let style = LenientJson::new(document).field("style").as_str().and_then(WindowControlsStyle::from_id);
        Self { style: style.unwrap_or_default() }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({ "style": self.style.id() })
    }

    #[must_use]
    pub fn style(&self) -> WindowControlsStyle {
        self.style
    }

    pub fn set_style(&mut self, style: WindowControlsStyle) {
        self.style = style;
    }

    /// The name of the choice in the preferences window.
    #[must_use]
    pub fn title(catalogue: &MessageCatalogue) -> &str {
        catalogue.text("Window Buttons")
    }
}
