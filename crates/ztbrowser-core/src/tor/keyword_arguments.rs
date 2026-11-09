//! The `KEY=value` arguments Tor puts in control-port replies and events.
//!
//! `NOTICE BOOTSTRAP PROGRESS=50 TAG=loading_descriptors SUMMARY="Loading relay descriptors"`: words without
//! `=` come first, then arguments whose values are either bare or quoted with C-style escapes (`\"`, `\\`).

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KeywordArguments {
    words: Vec<String>,
    arguments: Vec<(String, String)>,
}

impl KeywordArguments {
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut parsed = Self::default();
        let mut characters = text.chars().peekable();
        loop {
            while characters.next_if(|character| *character == ' ').is_some() {}
            if characters.peek().is_none() {
                return parsed;
            }
            let mut key = String::new();
            while let Some(character) = characters.next_if(|character| *character != ' ' && *character != '=') {
                key.push(character);
            }
            if characters.next_if_eq(&'=').is_none() {
                parsed.words.push(key);
                continue;
            }
            let mut value = String::new();
            if characters.next_if_eq(&'"').is_some() {
                while let Some(character) = characters.next() {
                    match character {
                        '"' => break,
                        '\\' => value.extend(characters.next()),
                        other => value.push(other),
                    }
                }
            } else {
                while let Some(character) = characters.next_if(|character| *character != ' ') {
                    value.push(character);
                }
            }
            parsed.arguments.push((key, value));
        }
    }

    /// The words before the first argument (`NOTICE`, `BOOTSTRAP`).
    #[must_use]
    pub fn words(&self) -> &[String] {
        &self.words
    }

    /// The value of argument `key`, unquoted.
    #[must_use]
    pub fn value(&self, key: &str) -> Option<&str> {
        self.arguments.iter().find(|(name, _)| name == key).map(|(_, value)| value.as_str())
    }
}
