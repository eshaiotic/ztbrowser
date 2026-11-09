//! Turning what the user typed into the address bar into a URL to open.

use ztbrowser_core::navigation::address_bar_input::AddressBarInput;
use ztbrowser_core::navigation::search_engine::SearchEngine;

struct AddressBarScenario {
    typed: String,
    search_engine: SearchEngine,
}

impl AddressBarScenario {
    fn given_user_typed(text: &str) -> Self {
        Self { typed: text.to_owned(), search_engine: SearchEngine::DUCKDUCKGO }
    }

    fn with_search_engine(mut self, engine: SearchEngine) -> Self {
        self.search_engine = engine;
        self
    }

    fn when_enter_is_pressed(&self) -> AddressBarOutcome {
        AddressBarOutcome {
            typed: self.typed.clone(),
            destination: AddressBarInput::new(&self.typed).destination(self.search_engine),
        }
    }
}

struct AddressBarOutcome {
    typed: String,
    destination: Option<String>,
}

impl AddressBarOutcome {
    fn then_opens(&self, expected: &str) {
        assert_eq!(self.destination.as_deref(), Some(expected), "typed: {:?}", self.typed);
    }

    fn then_searches_duckduckgo(&self) {
        let destination = self.destination.as_deref().unwrap_or_default();
        assert!(destination.starts_with("https://duckduckgo.com/?q="), "typed: {:?} -> {destination}", self.typed);
    }

    fn then_does_nothing(&self) {
        assert_eq!(self.destination, None);
    }
}

#[test]
fn should_open_full_url_as_typed() {
    AddressBarScenario::given_user_typed("https://mbank.pl/login")
        .when_enter_is_pressed()
        .then_opens("https://mbank.pl/login");
}

#[test]
fn should_open_bare_domain_over_https() {
    AddressBarScenario::given_user_typed("mbank.pl").when_enter_is_pressed().then_opens("https://mbank.pl/");
}

#[test]
fn should_open_domain_with_path_ignoring_surrounding_spaces() {
    AddressBarScenario::given_user_typed("  mail.google.com/mail/u/0  ")
        .when_enter_is_pressed()
        .then_opens("https://mail.google.com/mail/u/0");
}

#[test]
fn should_open_localhost_over_plain_http() {
    AddressBarScenario::given_user_typed("localhost:8080/app")
        .when_enter_is_pressed()
        .then_opens("http://localhost:8080/app");
}

#[test]
fn should_open_ip_address_over_plain_http() {
    AddressBarScenario::given_user_typed("192.168.1.1").when_enter_is_pressed().then_opens("http://192.168.1.1/");
}

#[test]
fn should_open_about_and_file_urls() {
    AddressBarScenario::given_user_typed("about:blank").when_enter_is_pressed().then_opens("about:blank");
    AddressBarScenario::given_user_typed("file:///home/me/notes.html")
        .when_enter_is_pressed()
        .then_opens("file:///home/me/notes.html");
}

#[test]
fn should_search_for_words_and_numbers() {
    for typed in ["privacy browser linux", "3.14", "what is example.com"] {
        AddressBarScenario::given_user_typed(typed).when_enter_is_pressed().then_searches_duckduckgo();
    }
}

#[test]
fn should_search_instead_of_running_script_urls() {
    for typed in ["javascript:alert(document.cookie)", "data:text/html,x"] {
        AddressBarScenario::given_user_typed(typed).when_enter_is_pressed().then_searches_duckduckgo();
    }
}

#[test]
fn should_encode_search_terms_safely() {
    AddressBarScenario::given_user_typed("zażółć & gęślą")
        .when_enter_is_pressed()
        .then_opens("https://duckduckgo.com/?q=za%C5%BC%C3%B3%C5%82%C4%87+%26+g%C4%99%C5%9Bl%C4%85");
}

#[test]
fn should_search_with_the_chosen_engine() {
    AddressBarScenario::given_user_typed("privacy browser")
        .with_search_engine(SearchEngine::BRAVE_SEARCH)
        .when_enter_is_pressed()
        .then_opens("https://search.brave.com/search?q=privacy+browser");
}

#[test]
fn should_open_addresses_whatever_engine_is_chosen() {
    AddressBarScenario::given_user_typed("mbank.pl")
        .with_search_engine(SearchEngine::GOOGLE)
        .when_enter_is_pressed()
        .then_opens("https://mbank.pl/");
}

#[test]
fn should_do_nothing_for_blank_input() {
    AddressBarScenario::given_user_typed("   ").when_enter_is_pressed().then_does_nothing();
}
