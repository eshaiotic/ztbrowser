//! Guard's page script asks for the generic selectors matching what the page uses: the classes and ids it
//! found, for the Guard lists WebKit applies on the page. The message comes from the page's process, so it is
//! read as untrusted input.

use std::collections::HashMap;

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::guard::generic_hiding::GenericHiding;
use ztbrowser_core::guard::generic_hiding_request::GenericHidingRequest;

struct RequestScenario {
    lists: HashMap<String, GenericHiding>,
}

impl RequestScenario {
    fn given_list_hiding(name: &str, selectors: &[&str]) -> Self {
        Self { lists: HashMap::new() }.and_list_hiding(name, selectors)
    }

    fn and_list_hiding(mut self, name: &str, selectors: &[&str]) -> Self {
        let mut hiding = GenericHiding::default();
        for selector in selectors {
            hiding.add(selector, &[]);
        }
        self.lists.insert(name.to_owned(), hiding);
        self
    }

    fn when_asked(&self, message: &Value) -> Option<Value> {
        GenericHidingRequest::from_json(message).map(|request| request.answer(&self.lists))
    }
}

fn message(lists: &[&str], tokens: &[&str], unkeyed: bool) -> Value {
    json!({"host": "news.pl", "lists": lists, "tokens": tokens, "unkeyed": unkeyed})
}

#[test]
fn should_answer_with_the_selectors_of_classes_the_page_uses() {
    let answer = RequestScenario::given_list_hiding("guard-ads", &[".ad-banner", ".promo"]).when_asked(&message(
        &["guard-ads"],
        &[".ad-banner"],
        false,
    ));

    assert_eq!(answer, Some(json!({"selectors": [".ad-banner"], "unkeyed": []})));
}

#[test]
fn should_skip_lists_webkit_does_not_apply_on_the_page() {
    let answer = RequestScenario::given_list_hiding("guard-ads", &[".ad-banner"])
        .and_list_hiding("guard-annoyances", &[".cookie-bar"])
        .when_asked(&message(&["guard-annoyances"], &[".ad-banner", ".cookie-bar"], false));

    assert_eq!(answer, Some(json!({"selectors": [".cookie-bar"], "unkeyed": []})));
}

#[test]
fn should_hand_over_unkeyed_selectors_when_the_page_asks_for_them() {
    let answer = RequestScenario::given_list_hiding("guard-ads", &[".ad-banner", "[data-ad-slot]"])
        .when_asked(&message(&["guard-ads"], &[], true));

    assert_eq!(answer, Some(json!({"selectors": [], "unkeyed": ["[data-ad-slot]"]})));
}

#[test]
fn should_ignore_a_list_it_does_not_know() {
    let answer = RequestScenario::given_list_hiding("guard-ads", &[".ad-banner"]).when_asked(&message(
        &["guard-unknown"],
        &[".ad-banner"],
        false,
    ));

    assert_eq!(answer, Some(json!({"selectors": [], "unkeyed": []})));
}

#[test]
fn should_reject_a_message_without_a_host() {
    let answer = RequestScenario::given_list_hiding("guard-ads", &[".ad-banner"])
        .when_asked(&json!({"lists": ["guard-ads"], "tokens": [".ad-banner"]}));

    assert_eq!(answer, None);
}

#[test]
fn should_read_no_more_tokens_than_a_page_can_plausibly_use() {
    let flood: Vec<String> = (0..=GenericHidingRequest::MAX_TOKENS).map(|number| format!(".c{number}")).collect();
    let last = flood.last().cloned().unwrap_or_default();
    let answer = RequestScenario::given_list_hiding("guard-ads", &[last.as_str()])
        .when_asked(&json!({"host": "news.pl", "lists": ["guard-ads"], "tokens": flood, "unkeyed": false}));

    assert_eq!(answer, Some(json!({"selectors": [], "unkeyed": []})));
}
