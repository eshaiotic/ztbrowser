//! A system locale name (`pt_BR.UTF-8`, `sr_RS@latin`) taken apart into language, script and region.

/// The parts of a POSIX locale name that decide which interface language fits: the encoding is dropped and the
/// `@latin` / `@cyrillic` modifiers are read as scripts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocaleName {
    language: String,
    script: Option<String>,
    region: Option<String>,
}

impl LocaleName {
    /// `None` for the `C` and `POSIX` locales, which name no language, and for an empty name.
    #[must_use]
    pub fn parse(locale: &str) -> Option<Self> {
        let without_encoding = locale.split('.').next().unwrap_or_default();
        let (name, modifier) = without_encoding
            .split_once('@')
            .map_or((without_encoding, None), |(name, modifier)| (name, Some(modifier)));
        let mut parts = name.split(['_', '-']);
        let language = parts.next().unwrap_or_default().to_ascii_lowercase();
        if language.is_empty() || language == "c" || language == "posix" {
            return None;
        }
        let mut script = modifier.and_then(Self::script_of_modifier);
        let mut region = None;
        for part in parts {
            if part.len() == 4 && part.chars().all(|letter| letter.is_ascii_alphabetic()) {
                script = Some(Self::title_case(part));
            } else if part.len() == 2 || part.len() == 3 {
                region = Some(part.to_ascii_uppercase());
            }
        }
        Some(Self { language, script, region })
    }

    /// Lower-case language code: `pt`.
    #[must_use]
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Script such as `Latn` or `Hant`, when the name states one.
    #[must_use]
    pub fn script(&self) -> Option<&str> {
        self.script.as_deref()
    }

    /// Upper-case region such as `BR`, or the three digits of a macro-region (`419`).
    #[must_use]
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    fn script_of_modifier(modifier: &str) -> Option<String> {
        match modifier {
            "latin" => Some("Latn".to_owned()),
            "cyrillic" => Some("Cyrl".to_owned()),
            _ => None,
        }
    }

    fn title_case(word: &str) -> String {
        let mut letters = word.chars();
        let first = letters.next().map(|letter| letter.to_ascii_uppercase());
        first.into_iter().chain(letters.map(|letter| letter.to_ascii_lowercase())).collect()
    }
}
