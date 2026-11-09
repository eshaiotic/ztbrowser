//! A message the window shows once when it opens (for example that the web engine is out of date).

use std::sync::OnceLock;

static NOTICE: OnceLock<String> = OnceLock::new();

pub struct StartupNotice;

impl StartupNotice {
    /// Only the first call has an effect.
    pub fn set(text: String) {
        let _ = NOTICE.set(text);
    }

    /// Empty when there is nothing to tell.
    #[must_use]
    pub fn text() -> &'static str {
        NOTICE.get().map_or("", String::as_str)
    }
}
