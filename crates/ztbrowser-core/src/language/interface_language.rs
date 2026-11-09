//! A language the interface can speak, taken from the list Brave ships (68 languages).

use crate::language::locale_name::LocaleName;

/// One interface language: a BCP 47 tag (`pt-BR`) and its name in its own language (`Português (Brasil)`), the
/// way language pickers list them so that a person can find their language without reading another one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterfaceLanguage {
    tag: &'static str,
    endonym: &'static str,
}

impl InterfaceLanguage {
    /// What a person gets when nothing known matches their system: the language the messages are written in.
    pub const ENGLISH: Self = Self::new("en-US", "English (United States)");

    /// Every language, in the order of Brave's list.
    pub const ALL: [Self; 68] = [
        Self::new("af", "Afrikaans"),
        Self::new("am", "አማርኛ"),
        Self::new("ar", "العربية"),
        Self::new("az", "Azərbaycanca"),
        Self::new("bg", "Български"),
        Self::new("bn", "বাংলা"),
        Self::new("bs", "Bosanski"),
        Self::new("ca", "Català"),
        Self::new("cs", "Čeština"),
        Self::new("da", "Dansk"),
        Self::new("de", "Deutsch"),
        Self::new("el", "Ελληνικά"),
        Self::new("en-GB", "English (United Kingdom)"),
        Self::new("en-US", "English (United States)"),
        Self::new("es-419", "Español (Latinoamérica)"),
        Self::new("es", "Español (España)"),
        Self::new("et", "Eesti"),
        Self::new("fa", "فارسی"),
        Self::new("fi", "Suomi"),
        Self::new("fil", "Filipino"),
        Self::new("fr", "Français"),
        Self::new("gu", "ગુજરાતી"),
        Self::new("he", "עברית"),
        Self::new("hi", "हिन्दी"),
        Self::new("hr", "Hrvatski"),
        Self::new("hu", "Magyar"),
        Self::new("id", "Bahasa Indonesia"),
        Self::new("it", "Italiano"),
        Self::new("ja", "日本語"),
        Self::new("ka", "ქართული"),
        Self::new("kk", "Қазақша"),
        Self::new("km", "ខ្មែរ"),
        Self::new("kn", "ಕನ್ನಡ"),
        Self::new("ko", "한국어"),
        Self::new("lo", "ລາວ"),
        Self::new("lt", "Lietuvių"),
        Self::new("lv", "Latviešu"),
        Self::new("mk", "Македонски"),
        Self::new("ml", "മലയാളം"),
        Self::new("mn", "Монгол"),
        Self::new("mr", "मराठी"),
        Self::new("ms", "Bahasa Melayu"),
        Self::new("my", "မြန်မာ"),
        Self::new("nb", "Norsk bokmål"),
        Self::new("nl", "Nederlands"),
        Self::new("pl", "Polski"),
        Self::new("pt-BR", "Português (Brasil)"),
        Self::new("pt-PT", "Português (Portugal)"),
        Self::new("ro", "Română"),
        Self::new("ru", "Русский"),
        Self::new("si", "සිංහල"),
        Self::new("sk", "Slovenčina"),
        Self::new("sl", "Slovenščina"),
        Self::new("sq", "Shqip"),
        Self::new("sr-Latn", "Srpski (latinica)"),
        Self::new("sr", "Српски"),
        Self::new("sv", "Svenska"),
        Self::new("sw", "Kiswahili"),
        Self::new("ta", "தமிழ்"),
        Self::new("te", "తెలుగు"),
        Self::new("th", "ไทย"),
        Self::new("tr", "Türkçe"),
        Self::new("uk", "Українська"),
        Self::new("ur", "اردو"),
        Self::new("uz", "Oʻzbekcha"),
        Self::new("vi", "Tiếng Việt"),
        Self::new("zh-CN", "简体中文"),
        Self::new("zh-TW", "繁體中文"),
    ];

    /// Countries whose Spanish is the Latin American one (`es-419`).
    const LATIN_AMERICAN_REGIONS: [&'static str; 22] = [
        "419", "AR", "BO", "CL", "CO", "CR", "CU", "DO", "EC", "GT", "HN", "MX", "NI", "PA", "PE", "PR", "PY", "SV",
        "US", "UY", "VE", "BZ",
    ];

    /// Countries whose English is the British one (`en-GB`) rather than the American one.
    const BRITISH_ENGLISH_REGIONS: [&'static str; 7] = ["GB", "IE", "AU", "NZ", "ZA", "IN", "SG"];

    const fn new(tag: &'static str, endonym: &'static str) -> Self {
        Self { tag, endonym }
    }

    /// The language with this tag, if the interface offers it.
    #[must_use]
    pub fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|language| language.tag == tag)
    }

    /// The language that fits a system locale name such as `pl_PL.UTF-8`, or `None` when the interface does not
    /// speak it (or the name is the `C` locale).
    #[must_use]
    pub fn matching_locale(locale: &str) -> Option<Self> {
        let name = LocaleName::parse(locale)?;
        let region = name.region().unwrap_or_default();
        let tag = match name.language() {
            "en" if Self::BRITISH_ENGLISH_REGIONS.contains(&region) => "en-GB",
            "en" => "en-US",
            "es" if Self::LATIN_AMERICAN_REGIONS.contains(&region) => "es-419",
            "pt" if region == "PT" => "pt-PT",
            "pt" => "pt-BR",
            "zh" if name.script() == Some("Hant") || ["TW", "HK", "MO"].contains(&region) => "zh-TW",
            "zh" => "zh-CN",
            "sr" if name.script() == Some("Latn") => "sr-Latn",
            "no" | "nn" => "nb",
            "tl" => "fil",
            "iw" => "he",
            "in" => "id",
            other => other,
        };
        Self::from_tag(tag)
    }

    /// The BCP 47 tag, also the name of the language's catalogue file (`pt-BR.po`).
    #[must_use]
    pub fn tag(self) -> &'static str {
        self.tag
    }

    /// The language's name in that language.
    #[must_use]
    pub fn endonym(self) -> &'static str {
        self.endonym
    }

    /// Whether the language is written from right to left (Arabic, Persian, Hebrew, Urdu), so pages and layouts
    /// must be mirrored.
    #[must_use]
    pub fn is_right_to_left(self) -> bool {
        matches!(self.tag, "ar" | "fa" | "he" | "ur")
    }

    /// The value of the `LANGUAGE` variable that makes GTK, libadwaita and WebKit speak this language: gettext's
    /// own spelling (`pt_BR`), followed by the plain language it falls back to (`pt`).
    #[must_use]
    pub fn gettext_languages(self) -> String {
        match self.tag {
            "sr-Latn" => "sr@latin".to_owned(),
            "es-419" => "es_MX:es".to_owned(),
            "fil" => "fil:tl".to_owned(),
            "nb" => "nb:no".to_owned(),
            tag => match tag.split_once('-') {
                Some((language, region)) => format!("{language}_{region}:{language}"),
                None => tag.to_owned(),
            },
        }
    }
}
