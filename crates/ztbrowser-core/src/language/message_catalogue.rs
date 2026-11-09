//! The translated messages of one interface language, read from a gettext `.po` file.
//!
//! The messages in the source code are written in English and act as their own keys, as in gettext. A message
//! the catalogue lacks (or holds with an empty or fuzzy translation) is shown in English, never half-translated:
//! a test makes sure no catalogue lacks one. Placeholders are named (`{error}`) so a translator can move them
//! around the sentence.

use std::collections::HashMap;

/// Messages and their translations in one language.
#[derive(Debug, Clone, Default)]
pub struct MessageCatalogue {
    translations: HashMap<String, String>,
}

/// Which part of a `.po` entry the continuation lines (`"..."`) belong to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum EntryPart {
    None,
    Message,
    Translation,
    Unsupported,
}

#[derive(Default)]
struct PoEntry {
    message: String,
    translation: String,
    is_fuzzy: bool,
    is_supported: bool,
}

impl MessageCatalogue {
    /// A catalogue without translations: every message is shown as written (English).
    #[must_use]
    pub fn untranslated() -> Self {
        Self::default()
    }

    /// Reads a `.po` file. Plural forms and contexts are not used by ZTBrowser and are skipped.
    #[must_use]
    pub fn parse(source: &str) -> Self {
        let mut translations = HashMap::new();
        let mut entry = PoEntry { is_supported: true, ..PoEntry::default() };
        let mut part = EntryPart::None;
        let mut pending_fuzzy = false;
        for line in source.lines().map(str::trim) {
            if line.is_empty() {
                Self::finish(&mut translations, &mut entry);
                part = EntryPart::None;
                pending_fuzzy = false;
            } else if let Some(flags) = line.strip_prefix("#,") {
                pending_fuzzy |= flags.contains("fuzzy");
            } else if line.starts_with('#') {
            } else if let Some(text) = line.strip_prefix("msgid ") {
                if part != EntryPart::None {
                    Self::finish(&mut translations, &mut entry);
                }
                entry = PoEntry { is_fuzzy: pending_fuzzy, is_supported: true, ..PoEntry::default() };
                pending_fuzzy = false;
                entry.message.push_str(&Self::unquote(text));
                part = EntryPart::Message;
            } else if let Some(text) = line.strip_prefix("msgstr ") {
                entry.translation.push_str(&Self::unquote(text));
                part = EntryPart::Translation;
            } else if line.starts_with('"') {
                match part {
                    EntryPart::Message => entry.message.push_str(&Self::unquote(line)),
                    EntryPart::Translation => entry.translation.push_str(&Self::unquote(line)),
                    EntryPart::None | EntryPart::Unsupported => {}
                }
            } else {
                entry.is_supported = false;
                part = EntryPart::Unsupported;
            }
        }
        Self::finish(&mut translations, &mut entry);
        Self { translations }
    }

    /// The message in this language, or as written when the catalogue has no translation of it.
    #[must_use]
    pub fn text<'a>(&'a self, message: &'a str) -> &'a str {
        self.translations.get(message).map_or(message, String::as_str)
    }

    /// The message in this language with each `{name}` replaced by its value; a `{name}` without a value stays.
    #[must_use]
    pub fn text_with(&self, message: &str, values: &[(&str, &str)]) -> String {
        let text = self.text(message);
        let mut filled = String::with_capacity(text.len());
        let mut copied_up_to = 0;
        for (start, end, name) in Self::placeholder_spans(text) {
            if let Some((_, value)) = values.iter().find(|(candidate, _)| *candidate == name) {
                filled.push_str(&text[copied_up_to..start]);
                filled.push_str(value);
                copied_up_to = end;
            }
        }
        filled.push_str(&text[copied_up_to..]);
        filled
    }

    /// Names of the placeholders in `message`, in order of first appearance.
    #[must_use]
    pub fn placeholders_of(message: &str) -> Vec<&str> {
        let mut names: Vec<&str> = Vec::new();
        for (_, _, name) in Self::placeholder_spans(message) {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        names
    }

    /// Every message this catalogue translates.
    #[must_use]
    pub fn messages(&self) -> Vec<&str> {
        self.translations.keys().map(String::as_str).collect()
    }

    fn finish(translations: &mut HashMap<String, String>, entry: &mut PoEntry) {
        let finished = std::mem::take(entry);
        entry.is_supported = true;
        if finished.is_supported
            && !finished.is_fuzzy
            && !finished.message.is_empty()
            && !finished.translation.is_empty()
        {
            translations.insert(finished.message, finished.translation);
        }
    }

    /// `"text"` without its quotes, with `\n`, `\t`, `\"` and `\\` resolved.
    fn unquote(quoted: &str) -> String {
        let inner = quoted.trim().strip_prefix('"').and_then(|rest| rest.strip_suffix('"')).unwrap_or_default();
        let mut text = String::with_capacity(inner.len());
        let mut letters = inner.chars();
        while let Some(letter) = letters.next() {
            if letter != '\\' {
                text.push(letter);
                continue;
            }
            match letters.next() {
                Some('n') => text.push('\n'),
                Some('t') => text.push('\t'),
                Some(other) => text.push(other),
                None => text.push('\\'),
            }
        }
        text
    }

    /// `(start, end, name)` of every `{name}` in `text`; a name is a letter or underscore followed by letters,
    /// digits and underscores, so `{}` and `{ d }` are not placeholders.
    fn placeholder_spans(text: &str) -> Vec<(usize, usize, &str)> {
        let mut spans = Vec::new();
        let mut search_from = 0;
        while let Some(open) = text[search_from..].find('{').map(|offset| search_from + offset) {
            let Some(close) = text[open..].find('}').map(|offset| open + offset) else { break };
            let name = &text[open + 1..close];
            let mut characters = name.chars();
            let is_name = characters.next().is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
                && characters.all(|letter| letter.is_ascii_alphanumeric() || letter == '_');
            if is_name {
                spans.push((open, close + 1, name));
                search_from = close + 1;
            } else {
                search_from = open + 1;
            }
        }
        spans
    }
}
