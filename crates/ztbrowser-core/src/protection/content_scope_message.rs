//! A message the content scope script posts to `window.webkit.messageHandlers.contentScopeScripts`.
//!
//! The script sends notifications (fire and forget) and requests (it waits for a reply whose `id` matches).
//! ZTBrowser has none of the native features those requests serve (Duck Player, breakage reports), so it
//! acknowledges notifications and answers every request with an error: the script then falls back to its
//! default behaviour instead of waiting forever. The message comes from a web page and is only read, never
//! trusted.

use serde_json::Value;
use serde_json::json;

pub struct ContentScopeMessage<'message> {
    message: &'message Value,
}

impl<'message> ContentScopeMessage<'message> {
    #[must_use]
    pub fn new(message: &'message Value) -> Self {
        Self { message }
    }

    /// The JSON text the script's `JSON.parse` receives as the reply.
    #[must_use]
    pub fn reply(&self) -> String {
        let Some(id) = self.message.get("id") else { return "{}".to_owned() };
        let feature = self.text_field("featureName");
        json!({
            "context": self.text_field("context"),
            "featureName": feature,
            "id": id,
            "error": {"message": format!("{feature}.{} is not supported by ZTBrowser", self.text_field("method"))},
        })
        .to_string()
    }

    fn text_field(&self, name: &str) -> &str {
        self.message.get(name).and_then(Value::as_str).unwrap_or_default()
    }
}
