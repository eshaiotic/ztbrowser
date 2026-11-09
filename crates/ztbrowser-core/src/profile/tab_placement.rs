//! Which side of the page the tabs sit on (`tab-placement.json`): above it, below it, or along its left or right edge.

use serde_json::Value;
use serde_json::json;

use crate::document::lenient_json::LenientJson;
use crate::language::message_catalogue::MessageCatalogue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TabPlacement {
    #[default]
    Top,
    Bottom,
    Left,
    Right,
}

impl TabPlacement {
    /// The choices in the order the preferences window lists them.
    pub const ALL: [Self; 4] = [Self::Top, Self::Bottom, Self::Left, Self::Right];

    /// A stable identifier (`left`), for storage and for the interface.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Top => "top",
            Self::Bottom => "bottom",
            Self::Left => "left",
            Self::Right => "right",
        }
    }

    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|placement| placement.id() == id)
    }

    /// Whether the tabs run along a horizontal edge, side by side, rather than stacked along a vertical one.
    #[must_use]
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::Top | Self::Bottom)
    }

    /// The name of the choice in the preferences window.
    #[must_use]
    pub fn label(self, catalogue: &MessageCatalogue) -> &str {
        match self {
            Self::Top => catalogue.text("Top"),
            Self::Bottom => catalogue.text("Bottom"),
            Self::Left => catalogue.text("Left"),
            Self::Right => catalogue.text("Right"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TabPlacementPreferences {
    placement: TabPlacement,
}

impl TabPlacementPreferences {
    /// An unknown or missing stored value keeps the default, the top.
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let placement = LenientJson::new(document).field("placement").as_str().and_then(TabPlacement::from_id);
        Self { placement: placement.unwrap_or_default() }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({ "placement": self.placement.id() })
    }

    #[must_use]
    pub fn placement(&self) -> TabPlacement {
        self.placement
    }

    pub fn set_placement(&mut self, placement: TabPlacement) {
        self.placement = placement;
    }

    /// The name of the choice in the preferences window.
    #[must_use]
    pub fn title(catalogue: &MessageCatalogue) -> &str {
        catalogue.text("Tab Position")
    }
}
