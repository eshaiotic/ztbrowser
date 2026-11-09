//! A short identifier that is safe everywhere ZTBrowser uses identifiers.
//!
//! Profile and web-app identifiers become directory names on disk, `.desktop` file names and parts of GTK
//! application ids (D-Bus names). Restricting them to `[a-z0-9_]` starting with a letter makes all three
//! safe at once: no path separators or `..` (no escaping the profile directory), no characters that need
//! quoting, and a valid D-Bus name element.

use std::fmt;

/// Why a stored identifier was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidSlug;

impl fmt::Display for InvalidSlug {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("identifier must be lower-case letters, digits and underscores, starting with a letter")
    }
}

impl std::error::Error for InvalidSlug {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Slug {
    text: String,
}

impl Slug {
    /// Longest identifier derived from free text; prefixes and numbering may add a few characters.
    pub const MAX_LENGTH: usize = 40;
    /// Longest identifier accepted from storage (derived identifier plus prefix and number).
    const MAX_STORED_LENGTH: usize = 64;
    const SEPARATOR: char = '_';
    const FALLBACK: &'static str = "unnamed";
    const DIGIT_PREFIX: &'static str = "x_";

    /// Derives an identifier from a name or host: `Praca – Żółć` becomes `praca_zolc`.
    #[must_use]
    pub fn from_text(text: &str) -> Self {
        let mut derived = String::with_capacity(text.len());
        for character in text.chars().flat_map(char::to_lowercase).map(Self::transliterated) {
            if character.is_ascii_alphanumeric() {
                derived.push(character);
            } else if !derived.is_empty() && !derived.ends_with(Self::SEPARATOR) {
                derived.push(Self::SEPARATOR);
            }
        }
        let mut derived = derived.trim_end_matches(Self::SEPARATOR).to_owned();
        if derived.is_empty() {
            Self::FALLBACK.clone_into(&mut derived);
        }
        if derived.starts_with(|first: char| first.is_ascii_digit()) {
            derived.insert_str(0, Self::DIGIT_PREFIX);
        }
        derived.truncate(Self::MAX_LENGTH);
        Self { text: derived.trim_end_matches(Self::SEPARATOR).to_owned() }
    }

    /// Validates an identifier read back from storage.
    ///
    /// # Errors
    /// [`InvalidSlug`] when the text is empty, too long, starts with something other than a lower-case
    /// letter, or contains anything but `[a-z0-9_]`.
    pub fn parse(text: &str) -> Result<Self, InvalidSlug> {
        let starts_with_letter = text.starts_with(|first: char| first.is_ascii_lowercase());
        let allowed = text.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == Self::SEPARATOR
        });
        if starts_with_letter && allowed && text.len() <= Self::MAX_STORED_LENGTH {
            Ok(Self { text: text.to_owned() })
        } else {
            Err(InvalidSlug)
        }
    }

    /// The same identifier with a distinguishing number: `mbank` becomes `mbank_2`.
    #[must_use]
    pub fn with_number(&self, number: u32) -> Self {
        Self { text: format!("{}{}{number}", self.text, Self::SEPARATOR) }
    }

    /// This identifier, or the first numbered variant (`_2`, `_3`, ...) for which `is_taken` is false.
    #[must_use]
    pub fn first_unused(&self, is_taken: impl Fn(&Self) -> bool) -> Self {
        if !is_taken(self) {
            return self.clone();
        }
        let mut numbered = (2..=u32::MAX).map(|number| self.with_number(number));
        numbered.find(|candidate| !is_taken(candidate)).unwrap_or_else(|| self.with_number(u32::MAX))
    }

    /// The same identifier behind a namespace prefix: `mbank` becomes `app_mbank`.
    #[must_use]
    pub fn with_prefix(&self, prefix: &str) -> Self {
        Self { text: format!("{prefix}{}{}", Self::SEPARATOR, self.text) }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Polish letters become their Latin base letter; everything else is left for the caller to filter.
    fn transliterated(character: char) -> char {
        match character {
            'ą' => 'a',
            'ć' => 'c',
            'ę' => 'e',
            'ł' => 'l',
            'ń' => 'n',
            'ó' => 'o',
            'ś' => 's',
            'ź' | 'ż' => 'z',
            other => other,
        }
    }
}

impl fmt::Display for Slug {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.text)
    }
}
