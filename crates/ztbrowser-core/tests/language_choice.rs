//! The user's choice of interface language: follow the system, or one fixed language.

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::language::interface_language::InterfaceLanguage;
use ztbrowser_core::language::language_choice::LanguageChoice;

struct LanguageChoiceScenario {
    choice: LanguageChoice,
    system_locales: Vec<String>,
}

impl LanguageChoiceScenario {
    fn given_stored(document: &Value) -> Self {
        Self { choice: LanguageChoice::from_json(document), system_locales: Vec::new() }
    }

    fn given_fixed(tag: &str) -> Self {
        let language = InterfaceLanguage::from_tag(tag).expect("a listed language");
        Self { choice: LanguageChoice::Fixed(language), system_locales: Vec::new() }
    }

    fn with_system_locales(mut self, locales: &[&str]) -> Self {
        self.system_locales = locales.iter().map(|locale| (*locale).to_owned()).collect();
        self
    }

    fn after_saving_and_loading(self) -> Self {
        Self { choice: LanguageChoice::from_json(&self.choice.to_json()), ..self }
    }

    fn then_speaks(&self, expected: &str) {
        assert_eq!(self.choice.resolve(&self.system_locales).tag(), expected);
    }
}

#[test]
fn should_follow_the_system_until_the_user_chooses() {
    assert_eq!(LanguageChoice::from_json(&Value::Null), LanguageChoice::System);
}

#[test]
fn should_speak_the_first_system_language_the_interface_knows() {
    LanguageChoiceScenario::given_stored(&Value::Null)
        .with_system_locales(&["xx_XX.UTF-8", "pl_PL.UTF-8", "pl", "C"])
        .then_speaks("pl");
}

#[test]
fn should_fall_back_to_american_english_when_no_system_language_is_known() {
    LanguageChoiceScenario::given_stored(&Value::Null).with_system_locales(&["C"]).then_speaks("en-US");
}

#[test]
fn should_speak_the_chosen_language_whatever_the_system_says() {
    LanguageChoiceScenario::given_fixed("de").with_system_locales(&["pl_PL.UTF-8"]).then_speaks("de");
}

#[test]
fn should_keep_the_chosen_language_across_restarts() {
    LanguageChoiceScenario::given_fixed("ja").after_saving_and_loading().then_speaks("ja");
}

#[test]
fn should_follow_the_system_again_when_the_stored_language_is_no_longer_offered() {
    let stored = json!({"language": "klingon"});
    assert_eq!(LanguageChoice::from_json(&stored), LanguageChoice::System);
}

#[test]
fn should_store_the_system_choice_as_the_word_system() {
    assert_eq!(LanguageChoice::System.to_json(), json!({"language": "system"}));
}
