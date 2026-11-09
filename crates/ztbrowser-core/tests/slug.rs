//! Identifiers that are safe in file paths, desktop file names and GTK application ids.

use ztbrowser_core::identity::slug::Slug;

struct SlugScenario {
    text: String,
}

impl SlugScenario {
    fn given_text(text: &str) -> Self {
        Self { text: text.to_owned() }
    }

    fn when_derived_from_text(&self) -> SlugOutcome {
        SlugOutcome { slug: Some(Slug::from_text(&self.text).as_str().to_owned()) }
    }

    fn when_parsed(&self) -> SlugOutcome {
        SlugOutcome { slug: Slug::parse(&self.text).ok().map(|slug| slug.as_str().to_owned()) }
    }
}

struct SlugOutcome {
    slug: Option<String>,
}

impl SlugOutcome {
    fn then_is(&self, expected: &str) {
        assert_eq!(self.slug.as_deref(), Some(expected));
    }

    fn then_is_rejected(&self) {
        assert_eq!(self.slug, None);
    }
}

#[test]
fn should_lowercase_and_join_words_with_underscores() {
    SlugScenario::given_text("Mail Google COM").when_derived_from_text().then_is("mail_google_com");
}

#[test]
fn should_transliterate_polish_letters() {
    SlugScenario::given_text("Praca – Żółć Łąka").when_derived_from_text().then_is("praca_zolc_laka");
}

#[test]
fn should_turn_dots_and_slashes_into_separators_so_paths_cannot_escape() {
    SlugScenario::given_text("../../etc/passwd").when_derived_from_text().then_is("etc_passwd");
}

#[test]
fn should_start_with_a_letter_even_when_text_starts_with_a_digit() {
    SlugScenario::given_text("1password.com").when_derived_from_text().then_is("x_1password_com");
}

#[test]
fn should_fall_back_to_a_name_for_text_without_letters_or_digits() {
    SlugScenario::given_text("!!! ???").when_derived_from_text().then_is("unnamed");
}

#[test]
fn should_limit_length() {
    let derived = Slug::from_text(&"a".repeat(200));

    assert_eq!(derived.as_str().len(), Slug::MAX_LENGTH);
}

#[test]
fn should_accept_valid_stored_identifier() {
    SlugScenario::given_text("app_mbank_pl").when_parsed().then_is("app_mbank_pl");
}

#[test]
fn should_reject_stored_identifier_with_path_characters() {
    for text in ["", "../x", "a/b", "A_upper", "1abc", "a-b", "a.b"] {
        SlugScenario::given_text(text).when_parsed().then_is_rejected();
    }
}

#[test]
fn should_add_numbered_suffix_to_make_identifier_unique() {
    assert_eq!(Slug::from_text("mbank").with_number(2).as_str(), "mbank_2");
}
