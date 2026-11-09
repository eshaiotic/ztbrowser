//! Which toolbar buttons a profile's browser windows show, kept per profile.
//!
//! The stored document lists the *hidden* buttons (`{"hiddenButtons": ["reload"]}`), so a button added by a
//! later ZTBrowser version appears on its own instead of staying hidden until the user finds it. The address
//! bar and the main menu are not buttons here: without the menu nobody could reach the preferences again.

use std::collections::BTreeSet;

use serde_json::Value;
use serde_json::json;

use crate::document::lenient_json::LenientJson;

/// A toolbar button the user can hide, in the order the preferences list them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ToolbarButton {
    BackAndForward,
    Reload,
    TorWindow,
    Protections,
    BurnData,
    Downloads,
    NewTab,
    InstallApp,
}

impl ToolbarButton {
    pub const ALL: [Self; 8] = [
        Self::BackAndForward,
        Self::Reload,
        Self::TorWindow,
        Self::Protections,
        Self::BurnData,
        Self::Downloads,
        Self::NewTab,
        Self::InstallApp,
    ];

    /// A stable identifier (`torWindow`), for storage.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::BackAndForward => "backAndForward",
            Self::Reload => "reload",
            Self::TorWindow => "torWindow",
            Self::Protections => "protections",
            Self::BurnData => "burnData",
            Self::Downloads => "downloads",
            Self::NewTab => "newTab",
            Self::InstallApp => "installApp",
        }
    }

    fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|button| button.id() == id)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ToolbarLayout {
    hidden: BTreeSet<ToolbarButton>,
}

impl ToolbarLayout {
    const STORED_KEY: &'static str = "hiddenButtons";

    /// The stored layout; buttons it does not offer are ignored, so every one it offers stays shown.
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let stored = LenientJson::new(document).field(Self::STORED_KEY).strings().unwrap_or_default();
        Self { hidden: stored.iter().filter_map(|id| ToolbarButton::from_id(id)).collect() }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({ Self::STORED_KEY: self.hidden.iter().map(|button| button.id()).collect::<Vec<_>>() })
    }

    #[must_use]
    pub fn shows(&self, button: ToolbarButton) -> bool {
        !self.hidden.contains(&button)
    }

    pub fn set_shown(&mut self, button: ToolbarButton, is_shown: bool) {
        if is_shown {
            self.hidden.remove(&button);
        } else {
            self.hidden.insert(button);
        }
    }
}
