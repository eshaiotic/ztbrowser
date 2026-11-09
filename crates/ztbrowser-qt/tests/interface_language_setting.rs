//! The interface language is listed with the system's language first and saved when it is chosen.

use tempfile::TempDir;
use ztbrowser_core::language::interface_language::InterfaceLanguage;
use ztbrowser_qt::interface_language_setting::InterfaceLanguageSetting;

use ztbrowser_qt as _;

struct LanguageScenario {
    directory: TempDir,
}

impl LanguageScenario {
    fn given_a_new_profile() -> Self {
        Self { directory: TempDir::new().expect("directory") }
    }

    fn setting(&self) -> InterfaceLanguageSetting {
        InterfaceLanguageSetting::open(&self.directory.path().join("interface.json"))
    }

    fn polish_position() -> usize {
        InterfaceLanguage::ALL.iter().position(|language| language.tag().starts_with("pl")).expect("Polish") + 1
    }
}

#[test]
fn should_follow_the_system_language_until_one_is_chosen() {
    assert_eq!(LanguageScenario::given_a_new_profile().setting().position(), 0);
}

#[test]
fn should_list_the_system_entry_first_and_every_language_after_it() {
    let names = InterfaceLanguageSetting::names("System default");
    assert_eq!(names.first().map(String::as_str), Some("System default"));
    assert_eq!(names.len(), InterfaceLanguage::ALL.len() + 1);
}

#[test]
fn should_save_the_chosen_language_for_the_next_start() {
    let scenario = LanguageScenario::given_a_new_profile();
    let changed = scenario.setting().choose_position(LanguageScenario::polish_position());
    assert!(changed);
    assert_eq!(scenario.setting().position(), LanguageScenario::polish_position());
}

#[test]
fn should_report_no_change_when_the_same_row_is_chosen_again() {
    let scenario = LanguageScenario::given_a_new_profile();
    assert!(!scenario.setting().choose_position(0));
}

#[test]
fn should_fall_back_to_the_system_language_for_a_row_that_does_not_exist() {
    let scenario = LanguageScenario::given_a_new_profile();
    let mut setting = scenario.setting();
    let _ = setting.choose_position(LanguageScenario::polish_position());
    let _ = setting.choose_position(10_000);
    assert_eq!(setting.position(), 0);
}
