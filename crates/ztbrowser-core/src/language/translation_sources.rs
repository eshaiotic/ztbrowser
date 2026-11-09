//! The `.po` catalogues compiled into the program, one per translated language.

use crate::language::interface_language::InterfaceLanguage;
use crate::language::message_catalogue::MessageCatalogue;

mod embedded {
    include!(concat!(env!("OUT_DIR"), "/catalogues.rs"));
}

pub struct TranslationSources;

impl TranslationSources {
    /// The catalogue of `language`: its translations, or none (every message as written, English) for the two
    /// English variants, whose messages are the source text.
    #[must_use]
    pub fn catalogue_of(language: InterfaceLanguage) -> MessageCatalogue {
        Self::source_of(language.tag()).map_or_else(MessageCatalogue::untranslated, MessageCatalogue::parse)
    }

    /// The `.po` text of the language with this tag, when one is compiled in.
    #[must_use]
    pub fn source_of(tag: &str) -> Option<&'static str> {
        embedded::CATALOGUES.iter().find(|(candidate, _)| *candidate == tag).map(|(_, source)| *source)
    }

    /// Tags of every compiled-in catalogue.
    #[must_use]
    pub fn tags() -> Vec<&'static str> {
        embedded::CATALOGUES.iter().map(|(tag, _)| *tag).collect()
    }
}
