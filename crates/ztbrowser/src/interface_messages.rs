//! The messages of the interface in the language this process speaks.
//!
//! One language is chosen when the process starts (see `InterfaceLanguageSetting::activate`) and installed here before
//! any window exists, so every part of the interface (menus, dialogs, toasts, the error page) draws from the
//! same catalogue and no screen mixes two languages. A message is written in English at its place in the code
//! and acts as its own key: `tr!("Back")`, `tr!("Close {name} first", name = profile)`.

use std::sync::OnceLock;

use ztbrowser_core::language::interface_language::InterfaceLanguage;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::language::translation_sources::TranslationSources;

static CATALOGUE: OnceLock<(InterfaceLanguage, MessageCatalogue)> = OnceLock::new();

/// Looks a message up in `tr!`'s catalogue; nothing else should call it.
pub struct InterfaceMessages;

impl InterfaceMessages {
    /// Makes `language` the process's language. Only the first call has an effect.
    pub fn install(language: InterfaceLanguage) {
        let _ = CATALOGUE.set((language, TranslationSources::catalogue_of(language)));
    }

    /// The language in use; American English until one is installed (tests, tools).
    pub fn language() -> InterfaceLanguage {
        Self::installed().0
    }

    /// The catalogue in use; the messages as written until a language is installed.
    pub fn catalogue() -> &'static MessageCatalogue {
        &Self::installed().1
    }

    fn installed() -> &'static (InterfaceLanguage, MessageCatalogue) {
        CATALOGUE.get_or_init(|| (InterfaceLanguage::ENGLISH, MessageCatalogue::untranslated()))
    }

    pub fn text(message: &'static str) -> &'static str {
        Self::catalogue().text(message)
    }

    pub fn text_with(message: &'static str, values: &[(&str, &str)]) -> String {
        Self::catalogue().text_with(message, values)
    }
}

/// A message of the interface in the process's language: `tr!("Back")`, or with named placeholders
/// `tr!("Close {name} first", name = profile)` (every value is shown with `Display`).
macro_rules! tr {
    ($message:literal) => {
        $crate::interface_messages::InterfaceMessages::text($message)
    };
    ($message:literal, $($name:ident = $value:expr),+ $(,)?) => {
        $crate::interface_messages::InterfaceMessages::text_with(
            $message,
            &[$((stringify!($name), ($value).to_string().as_str())),+],
        )
    };
}
