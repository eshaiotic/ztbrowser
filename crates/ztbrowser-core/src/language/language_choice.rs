//! The person's choice of interface language (`interface.json`): follow the system, or keep one language.

use serde_json::Value;
use serde_json::json;

use crate::language::interface_language::InterfaceLanguage;

/// Following the system is the default, so a new installation speaks the language of the desktop it runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LanguageChoice {
    #[default]
    System,
    Fixed(InterfaceLanguage),
}

impl LanguageChoice {
    const SYSTEM: &'static str = "system";

    /// Reads the stored document; anything missing, malformed or naming a language that is no longer offered
    /// means "follow the system".
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        document
            .get("language")
            .and_then(Value::as_str)
            .and_then(InterfaceLanguage::from_tag)
            .map_or(Self::System, Self::Fixed)
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        match self {
            Self::System => json!({"language": Self::SYSTEM}),
            Self::Fixed(language) => json!({"language": language.tag()}),
        }
    }

    /// The language to speak. `system_locales` are the system's languages, most preferred first.
    #[must_use]
    pub fn resolve(&self, system_locales: &[String]) -> InterfaceLanguage {
        match self {
            Self::Fixed(language) => *language,
            Self::System => Self::system_language(system_locales),
        }
    }

    /// The most preferred system language the interface speaks, American English when it speaks none of them.
    #[must_use]
    pub fn system_language(system_locales: &[String]) -> InterfaceLanguage {
        system_locales
            .iter()
            .find_map(|locale| InterfaceLanguage::matching_locale(locale))
            .unwrap_or(InterfaceLanguage::ENGLISH)
    }
}
