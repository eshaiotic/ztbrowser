//! The page shown instead of one that could not be loaded: what it says, what it offers, and that nothing
//! taken from the failed address can run or load inside it.

use std::path::Path;

use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::navigation::error_page::ErrorPage;
use ztbrowser_core::navigation::error_page::PageAppearance;
use ztbrowser_core::navigation::load_failure::CertificateProblem;
use ztbrowser_core::navigation::load_failure::FailureCause;
use ztbrowser_core::navigation::load_failure::LoadFailure;
use ztbrowser_core::navigation::search_engine::SearchEngine;

struct ErrorPageScenario {
    failure: LoadFailure,
    appearance: PageAppearance,
    search_engine: SearchEngine,
    catalogue: MessageCatalogue,
}

impl ErrorPageScenario {
    fn given_failure(address: &str, cause: FailureCause) -> Self {
        Self::given_failure_reported_as(address, cause, "")
    }

    fn given_failure_reported_as(address: &str, cause: FailureCause, detail: &str) -> Self {
        Self {
            failure: LoadFailure::new(address, cause, detail),
            appearance: PageAppearance {
                is_dark: false,
                accent: [0x35, 0x84, 0xe4],
                language_tag: "en",
                is_right_to_left: false,
            },
            search_engine: SearchEngine::DUCKDUCKGO,
            catalogue: MessageCatalogue::untranslated(),
        }
    }

    fn given_dark_browser_with_accent(mut self, accent: [u8; 3]) -> Self {
        self.appearance = PageAppearance { is_dark: true, accent, ..self.appearance };
        self
    }

    fn given_interface_language(mut self, tag: &'static str, is_right_to_left: bool) -> Self {
        self.appearance = PageAppearance { language_tag: tag, is_right_to_left, ..self.appearance };
        self
    }

    fn given_translations(mut self, po_source: &str) -> Self {
        self.catalogue = MessageCatalogue::parse(po_source);
        self
    }

    fn with_search_engine(mut self, engine: SearchEngine) -> Self {
        self.search_engine = engine;
        self
    }

    fn when_rendered(&self) -> RenderedErrorPage {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/error-page/error-page.html");
        let template = std::fs::read_to_string(path).expect("the vendored error page template");
        RenderedErrorPage {
            html: ErrorPage::new(&template).render(
                &self.failure,
                &self.appearance,
                self.search_engine,
                &self.catalogue,
            ),
        }
    }
}

struct RenderedErrorPage {
    html: String,
}

impl RenderedErrorPage {
    fn then_says(&self, text: &str) -> &Self {
        assert!(self.html.contains(text), "missing {text:?} in:\n{}", self.html);
        self
    }

    fn then_never_says(&self, text: &str) -> &Self {
        assert!(!self.html.contains(text), "unexpected {text:?} in:\n{}", self.html);
        self
    }

    fn then_heading_is(&self, heading: &str) -> &Self {
        self.then_says(&format!("<h1>{heading}</h1>"))
    }

    fn then_tab_is_titled(&self, title: &str) -> &Self {
        self.then_says(&format!("<title>{title}</title>"))
    }

    fn then_primary_action_opens(&self, label: &str, address: &str) -> &Self {
        self.then_says(&format!("<a class=\"button primary\" href=\"{address}\">{label}</a>"))
    }

    fn then_secondary_action_opens(&self, label: &str, address: &str) -> &Self {
        self.then_says(&format!("<a class=\"button\" href=\"{address}\">{label}</a>"))
    }

    fn then_offers_no_action(&self) -> &Self {
        self.then_never_says("class=\"button")
    }

    fn then_badge_shows(&self, glyph: &str, tone: &str) -> &Self {
        self.then_says(&format!("class=\"light tone-{tone} glyph-{glyph}\""))
    }

    fn then_every_placeholder_is_filled(&self) -> &Self {
        self.then_never_says("{{")
    }
}

#[test]
fn should_name_the_mistyped_host_and_offer_to_search_for_it() {
    ErrorPageScenario::given_failure_reported_as(
        "https://exmaple.com/login",
        FailureCause::HostNotFound,
        "Error resolving “exmaple.com”: Name or service not known",
    )
    .when_rendered()
    .then_heading_is("This site can’t be found")
    .then_says("ZTBrowser couldn’t find a server at <strong>exmaple.com</strong>.")
    .then_tab_is_titled("exmaple.com")
    .then_primary_action_opens("Try Again", "https://exmaple.com/login")
    .then_secondary_action_opens("Search DuckDuckGo", "https://duckduckgo.com/?q=exmaple.com")
    .then_says("<p class=\"detail\">Error resolving “exmaple.com”: Name or service not known</p>")
    .then_badge_shows("search", "accent")
    .then_every_placeholder_is_filled();
}

#[test]
fn should_escape_markup_smuggled_in_the_address_and_the_error_message() {
    ErrorPageScenario::given_failure_reported_as(
        "https://evil.test/\"><script>alert(1)</script>",
        FailureCause::ConnectionInterrupted,
        "<img src=x onerror=alert(2)>",
    )
    .when_rendered()
    .then_never_says("<script>alert")
    .then_never_says("<img src=x")
    .then_says("&lt;img src=x onerror=alert(2)&gt;")
    .then_says("href=\"https://evil.test/&quot;&gt;&lt;script&gt;alert(1)&lt;/script&gt;\"");
}

#[test]
fn should_forbid_scripts_forms_and_remote_loads_in_the_page() {
    ErrorPageScenario::given_failure("https://mbank.pl/", FailureCause::TimedOut)
        .when_rendered()
        .then_says("content=\"default-src 'none'; style-src 'unsafe-inline'; form-action 'none'; base-uri 'none'\"")
        .then_never_says("<script");
}

#[test]
fn should_never_turn_a_non_web_address_into_a_retry_link() {
    ErrorPageScenario::given_failure("javascript:alert(document.cookie)", FailureCause::Unknown)
        .when_rendered()
        .then_never_says("href=\"javascript:")
        .then_offers_no_action();
}

#[test]
fn should_not_expand_placeholders_written_into_the_address() {
    ErrorPageScenario::given_failure_reported_as("https://site.test/", FailureCause::Unknown, "{{actions}}")
        .when_rendered()
        .then_says("<p class=\"detail\">{{actions}}</p>");
}

#[test]
fn should_list_every_reason_the_certificate_failed_without_a_way_to_skip_it() {
    let problems = vec![CertificateProblem::Expired, CertificateProblem::WrongSite];
    ErrorPageScenario::given_failure("https://expired.badssl.com/", FailureCause::UntrustedCertificate(problems))
        .when_rendered()
        .then_heading_is("This connection isn’t private")
        .then_says("<li>The certificate has expired. If this computer’s date is wrong, correct it and try again.</li>")
        .then_says("<li>The certificate was issued for a different site.</li>")
        .then_says("this warning can’t be skipped by mistake")
        .then_primary_action_opens("Try Again", "https://expired.badssl.com/")
        .then_never_says("Search DuckDuckGo")
        .then_badge_shows("lock", "danger");
}

#[test]
fn should_still_explain_a_certificate_failure_that_names_no_reason() {
    ErrorPageScenario::given_failure("https://bank.test/", FailureCause::UntrustedCertificate(vec![]))
        .when_rendered()
        .then_says("<li>The certificate could not be verified.</li>");
}

#[test]
fn should_tell_the_user_they_are_offline() {
    ErrorPageScenario::given_failure("https://mail.google.com/", FailureCause::Offline)
        .when_rendered()
        .then_heading_is("You’re offline")
        .then_says("<strong>mail.google.com</strong> because this computer isn’t connected to a network")
        .then_badge_shows("offline", "accent");
}

#[test]
fn should_explain_an_empty_not_found_answer_and_offer_the_site_home_page() {
    ErrorPageScenario::given_failure("https://shop.test/products/42", FailureCause::HttpStatus(404))
        .when_rendered()
        .then_heading_is("This page can’t be found")
        .then_says("<strong>shop.test</strong> has no page at this address.")
        .then_says("<p class=\"detail\">HTTP 404</p>")
        .then_primary_action_opens("Reload Page", "https://shop.test/products/42")
        .then_secondary_action_opens("Go to Home Page", "https://shop.test/");
}

#[test]
fn should_not_offer_the_home_page_when_the_home_page_itself_failed() {
    ErrorPageScenario::given_failure("https://api.test/", FailureCause::HttpStatus(503))
        .when_rendered()
        .then_heading_is("This page isn’t working")
        .then_never_says("Go to Home Page");
}

#[test]
fn should_say_a_crashed_page_can_be_reloaded() {
    ErrorPageScenario::given_failure("https://maps.test/", FailureCause::PageCrashed)
        .when_rendered()
        .then_heading_is("This page crashed")
        .then_primary_action_opens("Reload Page", "https://maps.test/")
        .then_badge_shows("crash", "warning");
}

#[test]
fn should_name_the_missing_file_by_its_path() {
    ErrorPageScenario::given_failure("file:///home/ola/Pobrane/raport%202026.html", FailureCause::FileNotFound)
        .when_rendered()
        .then_heading_is("File not found")
        .then_says("There is no file at <strong>/home/ola/Pobrane/raport 2026.html</strong>.")
        .then_tab_is_titled("raport 2026.html");
}

#[test]
fn should_refuse_a_restricted_port_without_offering_a_retry() {
    ErrorPageScenario::given_failure("http://127.0.0.1:25/", FailureCause::RestrictedPort)
        .when_rendered()
        .then_heading_is("This address is blocked")
        .then_says("<strong>127.0.0.1</strong> on port 25")
        .then_offers_no_action();
}

#[test]
fn should_match_the_dark_browser_and_its_accent_colour() {
    ErrorPageScenario::given_failure("https://mbank.pl/", FailureCause::ConnectionRefused)
        .given_dark_browser_with_accent([0x9a, 0x6a, 0xff])
        .when_rendered()
        .then_says("<html lang=\"en\" dir=\"ltr\" class=\"dark tone-accent glyph-alert\">")
        .then_says("<meta name=\"color-scheme\" content=\"dark\">")
        .then_says("--accent: #9a6aff;");
}

const POLISH_TRANSLATIONS: &str = r#"
msgid "This page can’t be loaded"
msgstr "Nie można wczytać tej strony"

msgid "Something went wrong while loading {host}."
msgstr "Podczas wczytywania {host} coś poszło nie tak."

msgid "Try Again"
msgstr "Spróbuj ponownie"

msgid "Search {engine}"
msgstr "Szukaj w {engine}"
"#;

#[test]
fn should_word_the_page_in_the_language_of_the_catalogue() {
    ErrorPageScenario::given_failure("https://site.test/", FailureCause::Unknown)
        .given_translations(POLISH_TRANSLATIONS)
        .when_rendered()
        .then_heading_is("Nie można wczytać tej strony")
        .then_primary_action_opens("Spróbuj ponownie", "https://site.test/")
        .then_says("Podczas wczytywania <strong>site.test</strong> coś poszło nie tak.");
}

#[test]
fn should_keep_the_english_sentence_the_catalogue_does_not_translate() {
    ErrorPageScenario::given_failure("https://site.test/", FailureCause::Unknown)
        .given_translations(POLISH_TRANSLATIONS)
        .when_rendered()
        .then_says("<li>Check your internet connection.</li>");
}

#[test]
fn should_escape_markup_in_the_address_even_when_a_translation_places_it() {
    ErrorPageScenario::given_failure("https://<script>alert(1)</script>.test/", FailureCause::Unknown)
        .given_translations(POLISH_TRANSLATIONS)
        .when_rendered()
        .then_never_says("<script>")
        .then_says("<strong>https://&lt;script&gt;alert(1)&lt;/script&gt;.test/</strong> coś poszło nie tak.");
}

#[test]
fn should_name_the_search_engine_in_a_translated_button() {
    ErrorPageScenario::given_failure("https://exmaple.com/", FailureCause::HostNotFound)
        .with_search_engine(SearchEngine::GOOGLE)
        .given_translations(POLISH_TRANSLATIONS)
        .when_rendered()
        .then_secondary_action_opens("Szukaj w Google", "https://www.google.com/search?q=exmaple.com");
}

#[test]
fn should_mirror_the_page_for_a_right_to_left_language() {
    ErrorPageScenario::given_failure("https://site.test/", FailureCause::Unknown)
        .given_interface_language("he", true)
        .when_rendered()
        .then_says("<html lang=\"he\" dir=\"rtl\" class=")
        .then_every_placeholder_is_filled();
}

#[test]
fn should_offer_to_search_with_the_chosen_engine() {
    ErrorPageScenario::given_failure("https://exmaple.com/", FailureCause::HostNotFound)
        .with_search_engine(SearchEngine::GOOGLE)
        .when_rendered()
        .then_secondary_action_opens("Search Google", "https://www.google.com/search?q=exmaple.com");
}
