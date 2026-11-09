//! A request of Guard's page script for generic element hiding, and its answer.
//!
//! The script reports the page's host, the Guard lists WebKit applies on the page (it checks each list's probe
//! element) and the classes and ids it found. The message comes from the page's web process, which renders
//! hostile content, so it is read with limits on everything it carries.

use std::collections::HashMap;

use serde_json::Value;
use serde_json::json;

use crate::guard::generic_hiding::GenericHiding;

pub struct GenericHidingRequest {
    host: String,
    lists: Vec<String>,
    tokens: Vec<String>,
    wants_unkeyed: bool,
}

impl GenericHidingRequest {
    /// Far more distinct classes and ids than the largest pages use; the rest of a longer message is ignored.
    pub const MAX_TOKENS: usize = 20_000;
    const MAX_TOKEN_LENGTH: usize = 256;
    const MAX_HOST_LENGTH: usize = 253;
    const MAX_LISTS: usize = 8;

    /// `{"host": "news.pl", "lists": ["guard-ads"], "tokens": [".ad", "#top"], "unkeyed": true}`; `None` without
    /// a plausible host.
    #[must_use]
    pub fn from_json(message: &Value) -> Option<Self> {
        let host = message.get("host")?.as_str().filter(|host| host.len() <= Self::MAX_HOST_LENGTH)?;
        let mut lists: Vec<String> = Vec::new();
        for list in Self::strings(message, "lists").take(Self::MAX_LISTS) {
            if !lists.iter().any(|known| known == list) {
                lists.push(list.to_owned());
            }
        }
        let tokens = Self::strings(message, "tokens")
            .take(Self::MAX_TOKENS)
            .filter(|token| token.len() <= Self::MAX_TOKEN_LENGTH)
            .map(str::to_owned)
            .collect();
        let wants_unkeyed = message.get("unkeyed").and_then(Value::as_bool).unwrap_or(false);
        Some(Self { host: host.to_owned(), lists, tokens, wants_unkeyed })
    }

    /// `{"selectors": [...to hide...], "unkeyed": [...for the page to test...]}`, from the requested lists only.
    #[must_use]
    pub fn answer(&self, lists: &HashMap<String, GenericHiding>) -> Value {
        let active: Vec<&GenericHiding> = self.lists.iter().filter_map(|name| lists.get(name)).collect();
        let selectors: Vec<&str> =
            active.iter().flat_map(|hiding| hiding.selectors_for(&self.host, &self.tokens)).collect();
        let unkeyed: Vec<&str> = if self.wants_unkeyed {
            active.iter().flat_map(|hiding| hiding.unkeyed_selectors_for(&self.host)).collect()
        } else {
            Vec::new()
        };
        json!({ "selectors": selectors, "unkeyed": unkeyed })
    }

    /// The strings of the array at `key`; anything else in it is skipped.
    fn strings<'a>(message: &'a Value, key: &str) -> impl Iterator<Item = &'a str> {
        message.get(key).and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str)
    }
}
