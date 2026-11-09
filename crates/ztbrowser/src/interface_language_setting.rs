//! The person's choice of interface language, kept in `interface.json` and shared by every ZTBrowser process.
//!
//! The choice takes effect when a process starts: [`InterfaceLanguageSetting::activate`] runs before GTK draws
//! anything, makes GTK, libadwaita and WebKit speak the chosen language (through `LANGUAGE`, which they read as
//! the system's language list) and installs ZTBrowser's own catalogue, so a window never mixes two languages.
//! A change made in the preferences is saved at once and applies the next time ZTBrowser starts.

use std::cell::Cell;
use std::os::unix::process::CommandExt;
use std::process::Command;

use ztbrowser_core::language::interface_language::InterfaceLanguage;
use ztbrowser_core::language::language_choice::LanguageChoice;

use crate::interface_messages::InterfaceMessages;
use crate::stored_file::StoredFile;

pub struct InterfaceLanguageSetting {
    file: StoredFile,
    choice: Cell<LanguageChoice>,
}

impl InterfaceLanguageSetting {
    const LOG_DOMAIN: &'static str = "ztbrowser";
    const GETTEXT_VARIABLE: &'static str = "LANGUAGE";

    pub fn load(file: StoredFile) -> Self {
        let choice = file.read_json().map(|document| LanguageChoice::from_json(&document)).unwrap_or_default();
        Self { file, choice: Cell::new(choice) }
    }

    pub fn choice(&self) -> LanguageChoice {
        self.choice.get()
    }

    /// The language the system itself asks for, whatever is chosen: what "use the system language" would give.
    pub fn system_language() -> InterfaceLanguage {
        LanguageChoice::system_language(&Self::system_locales())
    }

    /// Chooses the language this process speaks; call once, before any widget or message exists.
    ///
    /// GTK, libadwaita and WebKit read the language list from `LANGUAGE` at their first lookup. The program may
    /// not change its own environment (`unsafe` is forbidden), so when a fixed language needs another `LANGUAGE`
    /// the process starts itself again with the right one, once, before it has created anything.
    pub fn activate(&self) {
        if let LanguageChoice::Fixed(chosen) = self.choice.get() {
            Self::restart_speaking(&chosen.gettext_languages());
        }
        InterfaceMessages::install(self.choice.get().resolve(&Self::system_locales()));
    }

    /// Replaces this process by a copy of itself whose `LANGUAGE` is `languages`; returns when it already is, or
    /// when the copy cannot start (then GTK's own texts stay in the system's language, which is logged).
    fn restart_speaking(languages: &str) {
        if std::env::var_os(Self::GETTEXT_VARIABLE).is_some_and(|current| current == languages) {
            return;
        }
        let error = std::env::current_exe().map(|program| {
            Command::new(program).args(std::env::args_os().skip(1)).env(Self::GETTEXT_VARIABLE, languages).exec()
        });
        glib::g_warning!(Self::LOG_DOMAIN, "cannot make GTK speak {languages}: {error:?}");
    }

    /// Saves the choice; it applies when ZTBrowser starts next.
    pub fn choose(&self, choice: LanguageChoice) {
        self.choice.set(choice);
        if let Err(error) = self.file.write_json(&choice.to_json()) {
            glib::g_warning!(Self::LOG_DOMAIN, "cannot save the interface language: {error}");
        }
    }

    fn system_locales() -> Vec<String> {
        glib::language_names().iter().map(ToString::to_string).collect()
    }
}
