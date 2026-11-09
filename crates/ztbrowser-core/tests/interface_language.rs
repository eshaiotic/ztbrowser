//! Which languages the interface speaks and how the system's locale picks one.

use ztbrowser_core::language::interface_language::InterfaceLanguage;

struct LocaleScenario {
    locale: &'static str,
}

impl LocaleScenario {
    fn given_system_locale(locale: &'static str) -> Self {
        Self { locale }
    }

    fn when_matched(&self) -> Option<&'static str> {
        InterfaceLanguage::matching_locale(self.locale).map(InterfaceLanguage::tag)
    }

    fn then_speaks(&self, expected: &str) {
        assert_eq!(self.when_matched(), Some(expected));
    }

    fn then_speaks_nothing_known(&self) {
        assert_eq!(self.when_matched(), None);
    }
}

#[test]
fn should_offer_the_sixty_eight_languages_brave_ships() {
    assert_eq!(InterfaceLanguage::ALL.len(), 68);
}

#[test]
fn should_list_every_language_once_with_a_name_in_its_own_language() {
    let tags: std::collections::BTreeSet<&str> = InterfaceLanguage::ALL.iter().map(|language| language.tag()).collect();
    let unnamed = InterfaceLanguage::ALL.iter().filter(|language| language.endonym().is_empty()).count();
    assert_eq!((tags.len(), unnamed), (68, 0));
}

#[test]
fn should_name_polish_in_polish() {
    assert_eq!(InterfaceLanguage::from_tag("pl").map(InterfaceLanguage::endonym), Some("Polski"));
}

#[test]
fn should_pick_polish_for_a_polish_system_with_encoding() {
    LocaleScenario::given_system_locale("pl_PL.UTF-8").then_speaks("pl");
}

#[test]
fn should_pick_brazilian_portuguese_for_pt_br() {
    LocaleScenario::given_system_locale("pt_BR.UTF-8").then_speaks("pt-BR");
}

#[test]
fn should_pick_european_portuguese_for_pt_pt() {
    LocaleScenario::given_system_locale("pt_PT").then_speaks("pt-PT");
}

#[test]
fn should_pick_brazilian_portuguese_when_only_the_language_is_given() {
    LocaleScenario::given_system_locale("pt").then_speaks("pt-BR");
}

#[test]
fn should_pick_latin_american_spanish_for_mexico() {
    LocaleScenario::given_system_locale("es_MX.UTF-8").then_speaks("es-419");
}

#[test]
fn should_pick_spanish_for_spain() {
    LocaleScenario::given_system_locale("es_ES.UTF-8").then_speaks("es");
}

#[test]
fn should_pick_simplified_chinese_for_mainland_china() {
    LocaleScenario::given_system_locale("zh_CN.UTF-8").then_speaks("zh-CN");
}

#[test]
fn should_pick_traditional_chinese_for_taiwan_and_hong_kong() {
    LocaleScenario::given_system_locale("zh_HK.UTF-8").then_speaks("zh-TW");
}

#[test]
fn should_pick_serbian_in_latin_script_for_the_latin_modifier() {
    LocaleScenario::given_system_locale("sr_RS@latin").then_speaks("sr-Latn");
}

#[test]
fn should_pick_serbian_in_cyrillic_script_by_default() {
    LocaleScenario::given_system_locale("sr_RS").then_speaks("sr");
}

#[test]
fn should_pick_british_english_for_the_united_kingdom() {
    LocaleScenario::given_system_locale("en_GB.UTF-8").then_speaks("en-GB");
}

#[test]
fn should_pick_american_english_for_any_other_english() {
    LocaleScenario::given_system_locale("en_CA.UTF-8").then_speaks("en-US");
}

#[test]
fn should_treat_the_norwegian_variants_as_bokmal() {
    LocaleScenario::given_system_locale("nn_NO").then_speaks("nb");
}

#[test]
fn should_follow_the_old_codes_for_hebrew_and_indonesian() {
    let hebrew = LocaleScenario::given_system_locale("iw_IL").when_matched();
    let indonesian = LocaleScenario::given_system_locale("in_ID").when_matched();
    assert_eq!((hebrew, indonesian), (Some("he"), Some("id")));
}

#[test]
fn should_pick_filipino_for_tagalog() {
    LocaleScenario::given_system_locale("tl_PH").then_speaks("fil");
}

#[test]
fn should_know_nothing_of_the_c_locale() {
    LocaleScenario::given_system_locale("C").then_speaks_nothing_known();
}

#[test]
fn should_know_nothing_of_an_unsupported_language() {
    LocaleScenario::given_system_locale("xx_YY.UTF-8").then_speaks_nothing_known();
}

#[test]
fn should_find_a_language_by_its_tag() {
    assert_eq!(InterfaceLanguage::from_tag("sr-Latn").map(InterfaceLanguage::tag), Some("sr-Latn"));
}

#[test]
fn should_not_find_a_language_by_a_made_up_tag() {
    assert_eq!(InterfaceLanguage::from_tag("klingon"), None);
}

#[test]
fn should_name_the_language_for_gettext_with_its_region_fallback() {
    let brazilian = InterfaceLanguage::from_tag("pt-BR").map(InterfaceLanguage::gettext_languages);
    assert_eq!(brazilian.as_deref(), Some("pt_BR:pt"));
}

#[test]
fn should_write_arabic_persian_hebrew_and_urdu_from_right_to_left() {
    let right_to_left: Vec<&str> = InterfaceLanguage::ALL
        .iter()
        .filter(|language| language.is_right_to_left())
        .map(|language| language.tag())
        .collect();
    assert_eq!(right_to_left, ["ar", "fa", "he", "ur"]);
}

#[test]
fn should_write_polish_from_left_to_right() {
    let polish = InterfaceLanguage::from_tag("pl").map(InterfaceLanguage::is_right_to_left);
    assert_eq!(polish, Some(false));
}

#[test]
fn should_name_serbian_in_latin_script_with_the_latin_modifier_for_gettext() {
    let serbian = InterfaceLanguage::from_tag("sr-Latn").map(InterfaceLanguage::gettext_languages);
    assert_eq!(serbian.as_deref(), Some("sr@latin"));
}
