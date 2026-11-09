//! Per-session settings handed to DuckDuckGo's Content Scope Scripts (`$USER_PREFERENCES$`).

use serde_json::Value;
use serde_json::json;

/// Mirrors the JSON shape of `ContentScopeProperties` in DuckDuckGo's macOS app.
///
/// The session key seeds the fingerprinting noise (canvas, audio, ...): one key per browsing session keeps a
/// site's view stable while preventing cross-session linking. The message secret authenticates messages the
/// script sends back to the browser. Both must come from a cryptographically secure random source; this type
/// only carries them, which keeps it deterministic and testable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentScopePreferences {
    pub global_privacy_control: bool,
    pub session_key: String,
    pub message_secret: String,
    /// Two-letter UI language, e.g. `pl`.
    pub language: String,
    /// The macOS app version whose Content Scope Scripts are bundled; features gate on it.
    pub compatibility_version: String,
}

impl ContentScopePreferences {
    /// Messages from the page-world script are addressed to this WebKit message handler.
    pub const MESSAGING_CONTEXT_NAME: &'static str = "contentScopeScripts";

    /// The bundled scripts and the downloaded configuration are the macOS ones, so the session is described as
    /// macOS; it never leaves the device.
    const PLATFORM_NAME: &'static str = "macos";

    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "globalPrivacyControlValue": self.global_privacy_control,
            "debug": false,
            "sessionKey": self.session_key,
            "messageSecret": self.message_secret,
            "language": self.language,
            "platform": {"name": Self::PLATFORM_NAME, "internal": false, "version": self.compatibility_version},
            "currentCohorts": [],
            "messagingContextName": Self::MESSAGING_CONTEXT_NAME
        })
    }
}
