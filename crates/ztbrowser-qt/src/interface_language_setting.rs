//! The person's choice of interface language (`interface.json`), shared by every ZTBrowser process.
//!
//! A change is saved at once and applies the next time ZTBrowser starts: every window, menu and dialog is built
//! in one language, so none ever mixes two.

use std::path::Path;

use ztbrowser_core::language::interface_language::InterfaceLanguage;
use ztbrowser_core::language::language_choice::LanguageChoice;
use ztbrowser_core::storage::owned_json_file::OwnedJsonFile;

use crate::system_locales::SystemLocales;

pub struct InterfaceLanguageSetting {
    choice: LanguageChoice,
    file: OwnedJsonFile,
}

impl InterfaceLanguageSetting {
    #[must_use]
    pub fn open(file: &Path) -> Self {
        let file = OwnedJsonFile::new(file.to_path_buf());
        let choice = file.read().map(|document| LanguageChoice::from_json(&document)).unwrap_or_default();
        Self { choice, file }
    }

    /// The language the system itself asks for, whatever is chosen: what "use the system language" would give.
    #[must_use]
    pub fn system_language() -> InterfaceLanguage {
        LanguageChoice::system_language(&SystemLocales::from_environment())
    }

    /// The language names in the order the preferences list them: row 0 is the system's language, row `n` the
    /// `n`-th of `InterfaceLanguage::ALL`; `system_entry` is the text of row 0.
    #[must_use]
    pub fn names(system_entry: &str) -> Vec<String> {
        let mut names = vec![system_entry.to_owned()];
        names.extend(InterfaceLanguage::ALL.iter().map(|language| language.endonym().to_owned()));
        names
    }

    /// The row of the chosen language.
    #[must_use]
    pub fn position(&self) -> usize {
        match self.choice {
            LanguageChoice::System => 0,
            LanguageChoice::Fixed(language) => {
                InterfaceLanguage::ALL.iter().position(|candidate| *candidate == language).map_or(0, |index| index + 1)
            }
        }
    }

    /// Chooses the language of `position`; a row that does not exist means the system's language. Returns whether
    /// the choice changed.
    pub fn choose_position(&mut self, position: usize) -> bool {
        let choice = position
            .checked_sub(1)
            .and_then(|index| InterfaceLanguage::ALL.get(index).copied())
            .map_or(LanguageChoice::System, LanguageChoice::Fixed);
        if choice == self.choice {
            return false;
        }
        self.choice = choice;
        let _ = self.file.write(&choice.to_json());
        true
    }
}
