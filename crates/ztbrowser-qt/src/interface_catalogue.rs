//! The messages of the interface in the language this process speaks.
//!
//! One language is chosen when the process starts (the person's choice in `interface.json`, else the system's)
//! and used for every text of the window, so no screen mixes two languages.

use std::fs;
use std::sync::OnceLock;

use serde_json::Value;
use ztbrowser_core::language::language_choice::LanguageChoice;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::language::translation_sources::TranslationSources;

use crate::installation_directories::InstallationDirectories;
use crate::system_locales::SystemLocales;

static CATALOGUE: OnceLock<MessageCatalogue> = OnceLock::new();

pub struct InterfaceCatalogue;

impl InterfaceCatalogue {
    /// Chooses the language from the person's saved choice and the system's. Only the first call has an effect.
    pub fn choose(directories: &InstallationDirectories) {
        let choice = fs::read(directories.interface_language_file())
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
            .map(|document| LanguageChoice::from_json(&document))
            .unwrap_or_default();
        let language = choice.resolve(&SystemLocales::from_environment());
        let _ = CATALOGUE.set(TranslationSources::catalogue_of(language));
    }

    /// The catalogue in use; the messages as written until a language is chosen.
    pub fn current() -> &'static MessageCatalogue {
        CATALOGUE.get_or_init(MessageCatalogue::untranslated)
    }
}
