//! Translations read from a gettext `.po` file, and filling in a message's named placeholders.

use ztbrowser_core::language::message_catalogue::MessageCatalogue;

struct CatalogueScenario {
    catalogue: MessageCatalogue,
}

impl CatalogueScenario {
    fn given_po(source: &str) -> Self {
        Self { catalogue: MessageCatalogue::parse(source) }
    }

    fn then_translates(&self, message: &str, expected: &str) {
        assert_eq!(self.catalogue.text(message), expected);
    }
}

const POLISH: &str = r#"
# Polish translation
msgid ""
msgstr ""
"Content-Type: text/plain; charset=UTF-8\n"
"Language: pl\n"

msgid "Back"
msgstr "Wstecz"

msgid "Cannot save download: {error}"
msgstr "Nie można zapisać pobierania: {error}"

#, fuzzy
msgid "Reload"
msgstr "Odśwież (do sprawdzenia)"

msgid "Not translated yet"
msgstr ""

msgid "Closes all tabs "
"and windows."
msgstr "Zamyka wszystkie karty "
"i okna."

msgid "Say \"hi\"\n"
msgstr "Powiedz \"cześć\"\n"
"#;

#[test]
fn should_give_the_translation_of_a_message() {
    CatalogueScenario::given_po(POLISH).then_translates("Back", "Wstecz");
}

#[test]
fn should_give_the_message_itself_when_the_catalogue_does_not_know_it() {
    CatalogueScenario::given_po(POLISH).then_translates("Forward", "Forward");
}

#[test]
fn should_give_the_message_itself_when_its_translation_is_empty() {
    CatalogueScenario::given_po(POLISH).then_translates("Not translated yet", "Not translated yet");
}

#[test]
fn should_ignore_a_translation_marked_fuzzy() {
    CatalogueScenario::given_po(POLISH).then_translates("Reload", "Reload");
}

#[test]
fn should_join_a_message_split_over_several_lines() {
    CatalogueScenario::given_po(POLISH)
        .then_translates("Closes all tabs and windows.", "Zamyka wszystkie karty i okna.");
}

#[test]
fn should_read_escaped_quotes_and_newlines() {
    CatalogueScenario::given_po(POLISH).then_translates("Say \"hi\"\n", "Powiedz \"cześć\"\n");
}

#[test]
fn should_not_take_the_header_for_a_message() {
    CatalogueScenario::given_po(POLISH).then_translates("", "");
}

#[test]
fn should_fill_the_named_placeholders_of_the_translation() {
    let scenario = CatalogueScenario::given_po(POLISH);
    let text = scenario.catalogue.text_with("Cannot save download: {error}", &[("error", "disk full")]);
    assert_eq!(text, "Nie można zapisać pobierania: disk full");
}

#[test]
fn should_fill_the_placeholders_of_the_message_itself_when_untranslated() {
    let text = MessageCatalogue::untranslated().text_with("Close {name} first", &[("name", "Work")]);
    assert_eq!(text, "Close Work first");
}

#[test]
fn should_list_the_placeholders_of_a_message_in_order_without_repeats() {
    assert_eq!(MessageCatalogue::placeholders_of("{a} then {b} then {a}"), vec!["a", "b"]);
}

#[test]
fn should_leave_braces_that_are_not_placeholders_alone() {
    assert_eq!(MessageCatalogue::placeholders_of("a {} b {1x} c { d }"), Vec::<&str>::new());
}

#[test]
fn should_list_every_message_the_catalogue_translates() {
    let scenario = CatalogueScenario::given_po(POLISH);
    let mut messages = scenario.catalogue.messages();
    messages.sort_unstable();
    assert_eq!(messages, vec!["Back", "Cannot save download: {error}", "Closes all tabs and windows.", "Say \"hi\"\n"]);
}
