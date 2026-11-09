//! What the About dialog shows: the application, its details and every licence it owes, in one language.

use serde_json::Value;
use ztbrowser_core::about::about_content::AboutContent;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;

struct AboutScenario {
    content: Value,
}

impl AboutScenario {
    fn given_the_untranslated_dialog() -> Self {
        Self { content: AboutContent::build(&MessageCatalogue::untranslated(), "1.2.0") }
    }

    fn sections(&self) -> &Vec<Value> {
        self.content["sections"].as_array().expect("sections")
    }

    fn section_titled(&self, title: &str) -> &Value {
        self.sections().iter().find(|section| section["title"] == title).expect("a section with that title")
    }
}

#[test]
fn should_name_the_application_and_its_version() {
    let scenario = AboutScenario::given_the_untranslated_dialog();
    assert_eq!(scenario.content["name"], "ZTBrowser");
    assert_eq!(scenario.content["version"], "1.2.0");
}

#[test]
fn should_describe_the_application_on_the_details_page() {
    let scenario = AboutScenario::given_the_untranslated_dialog();
    assert!(scenario.content["comments"].as_str().expect("comments").starts_with("A privacy-first browser for Linux"));
}

#[test]
fn should_start_the_legal_page_with_the_application_licence_and_no_title() {
    let scenario = AboutScenario::given_the_untranslated_dialog();
    let first = &scenario.sections()[0];
    assert_eq!(first["title"], Value::Null);
    assert!(first["body"].as_str().expect("body").contains("GNU General Public License, version 3 or later"));
}

#[test]
fn should_link_each_known_licence_to_its_text() {
    let scenario = AboutScenario::given_the_untranslated_dialog();
    let body = scenario.section_titled("Tor")["body"].as_str().expect("body");
    assert!(body.contains("<a href=\"https://opensource.org/licenses/BSD-3-Clause\">BSD 3-Clause License</a>"));
}

#[test]
fn should_show_the_text_of_a_section_that_has_no_known_licence() {
    let scenario = AboutScenario::given_the_untranslated_dialog();
    let body = scenario.section_titled("Tracker Data Set")["body"].as_str().expect("body");
    assert_eq!(body, "Downloaded at run time from DuckDuckGo; licensed CC BY-NC-SA 4.0.");
}

#[test]
fn should_list_one_section_per_component_the_application_owes_a_licence() {
    assert_eq!(AboutScenario::given_the_untranslated_dialog().sections().len(), 6);
}

#[test]
fn should_escape_markup_in_a_text_it_shows() {
    let content = AboutContent::escaped("a <b> & c");
    assert_eq!(content, "a &lt;b&gt; &amp; c");
}
