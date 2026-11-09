//! The search engine a profile sends its searches to, and its new tabs open.

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::navigation::search_engine::SearchEngine;

struct SearchEngineScenario {
    engine: SearchEngine,
}

impl SearchEngineScenario {
    fn given_stored(document: &Value) -> Self {
        Self { engine: SearchEngine::from_json(document) }
    }

    fn given_chosen(engine: SearchEngine) -> Self {
        Self { engine }
    }

    fn after_saving_and_loading(self) -> Self {
        Self { engine: SearchEngine::from_json(&self.engine.to_json()) }
    }

    fn then_searches_with(&self, expected: &str) {
        assert_eq!(self.engine.name(), expected);
    }

    fn then_search_for_opens(&self, text: &str, expected: &str) {
        assert_eq!(self.engine.search_url(text), expected);
    }

    fn then_new_tabs_open(&self, expected: &str) {
        assert_eq!(self.engine.home_page(), expected);
    }
}

#[test]
fn should_search_duckduckgo_until_another_engine_is_chosen() {
    SearchEngineScenario::given_stored(&Value::Null).then_searches_with("DuckDuckGo");
}

#[test]
fn should_keep_the_chosen_engine_across_restarts() {
    SearchEngineScenario::given_chosen(SearchEngine::BRAVE_SEARCH)
        .after_saving_and_loading()
        .then_searches_with("Brave Search");
}

#[test]
fn should_fall_back_to_duckduckgo_for_an_engine_it_does_not_offer() {
    SearchEngineScenario::given_stored(&json!({"searchEngine": "bing"})).then_searches_with("DuckDuckGo");
}

#[test]
fn should_send_searches_to_brave_search() {
    SearchEngineScenario::given_chosen(SearchEngine::BRAVE_SEARCH)
        .then_search_for_opens("linux browser", "https://search.brave.com/search?q=linux+browser");
}

#[test]
fn should_send_searches_to_google() {
    SearchEngineScenario::given_chosen(SearchEngine::GOOGLE)
        .then_search_for_opens("zażółć & co", "https://www.google.com/search?q=za%C5%BC%C3%B3%C5%82%C4%87+%26+co");
}

#[test]
fn should_open_new_tabs_at_the_engine_home_page() {
    SearchEngineScenario::given_chosen(SearchEngine::GOOGLE).then_new_tabs_open("https://www.google.com/");
}

#[test]
fn should_offer_duckduckgo_brave_search_and_google() {
    let names: Vec<&str> = SearchEngine::ALL.iter().map(|engine| engine.name()).collect();

    assert_eq!(names, ["DuckDuckGo", "Brave Search", "Google"]);
}

#[test]
fn should_search_duckduckgo_onion_service_in_the_tor_window() {
    SearchEngineScenario::given_chosen(SearchEngine::DUCKDUCKGO_ONION)
        .then_search_for_opens("tor", "https://duckduckgogg42xjoc72x3sjasowoarfbgcmvfimaftt6twagswzczad.onion/?q=tor");
}

#[test]
fn should_never_restore_the_onion_engine_into_an_ordinary_profile() {
    SearchEngineScenario::given_chosen(SearchEngine::DUCKDUCKGO_ONION)
        .after_saving_and_loading()
        .then_new_tabs_open("https://duckduckgo.com/");
}
