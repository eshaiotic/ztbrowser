//! DuckDuckGo's Content Scope Scripts, filled in and ready to inject into every frame of a page.
//!
//! The vendored bundle (`assets/content-scope-scripts/contentScope.js`, Apache-2.0) implements in-page
//! protections: fingerprinting noise, Global Privacy Control, cookie restrictions, web compatibility fixes.
//! It is a template with placeholders that the browser replaces before injection, as DuckDuckGo's macOS
//! `ContentScopeUserScript.generateSource` does.
//!
//! Replacement is a single pass over the template: text inserted for one placeholder is never scanned again,
//! so a configuration value that happens to contain a placeholder name cannot inject anything.

use serde_json::Value;

use crate::protection::content_scope_preferences::ContentScopePreferences;
use crate::protection::privacy_configuration::PrivacyConfiguration;

pub struct ContentScopeScript<'template> {
    template: &'template str,
}

impl<'template> ContentScopeScript<'template> {
    /// The configuration as a JavaScript string literal, which the bundle passes to `JSON.parse`. A string
    /// literal compiles to one constant instead of code building every object, saving memory in each frame.
    const CONFIGURATION_LITERAL: &'static str = "$CONTENT_SCOPE_JSON$";
    /// Bundles older than `$CONTENT_SCOPE_JSON$` expect an expression that evaluates to the object.
    const CONFIGURATION_OBJECT: &'static str = "$CONTENT_SCOPE$";
    const USER_UNPROTECTED_DOMAINS: &'static str = "$USER_UNPROTECTED_DOMAINS$";
    const USER_PREFERENCES: &'static str = "$USER_PREFERENCES$";
    /// Features the bundled `apple` build runs (`platformSupport.apple` in contentScope.js) and those they read
    /// (`featureConfigDependencies`). The bundle throws every other feature away before using the configuration
    /// (`pruneConfigFeatures`), so they are left out here instead: every frame tokenizes and parses the
    /// configuration again, and the discarded part — mostly `autoconsent` — is three quarters of it.
    /// A test reads this set out of the vendored bundle, so a bundle update cannot silently drop a feature.
    const BUNDLED_FEATURES: &'static [&'static str] = &[
        "webCompat",
        "duckPlayerNative",
        "fingerprintingAudio",
        "fingerprintingBattery",
        "fingerprintingCanvas",
        "googleRejected",
        "gpc",
        "fingerprintingHardware",
        "referrer",
        "fingerprintingScreenSize",
        "fingerprintingTemporaryStorage",
        "navigatorInterface",
        "elementHiding",
        "exceptionHandler",
        "apiManipulation",
        "pageContext",
        "print",
        "trackerProtection",
        "textSelection",
        "contentBlocking",
        "trackerAllowlist",
        "clickToLoad",
    ];

    #[must_use]
    pub fn new(template: &'template str) -> Self {
        Self { template }
    }

    #[must_use]
    pub fn render(
        &self,
        configuration: &PrivacyConfiguration,
        user_unprotected_domains: &[String],
        preferences: &ContentScopePreferences,
    ) -> String {
        let configuration_literal =
            Self::javascript_string_literal(&Self::bundled_part(configuration.document()).to_string());
        let substitutions = [
            (Self::CONFIGURATION_OBJECT, format!("JSON.parse({configuration_literal})")),
            (Self::CONFIGURATION_LITERAL, configuration_literal),
            (Self::USER_UNPROTECTED_DOMAINS, Value::from(user_unprotected_domains.to_vec()).to_string()),
            (Self::USER_PREFERENCES, preferences.to_json().to_string()),
        ];
        self.substituted(&substitutions)
    }

    fn substituted(&self, substitutions: &[(&str, String)]) -> String {
        let mut occurrences: Vec<(usize, &str, &str)> = substitutions
            .iter()
            .flat_map(|(placeholder, value)| {
                self.template
                    .match_indices(placeholder)
                    .map(move |(position, _)| (position, *placeholder, value.as_str()))
            })
            .collect();
        occurrences.sort_unstable_by_key(|&(position, ..)| position);

        let mut rendered = String::with_capacity(
            self.template.len() + substitutions.iter().map(|(_, value)| value.len()).sum::<usize>(),
        );
        let mut copied_up_to = 0;
        for (position, placeholder, value) in occurrences {
            rendered.push_str(&self.template[copied_up_to..position]);
            rendered.push_str(value);
            copied_up_to = position + placeholder.len();
        }
        rendered.push_str(&self.template[copied_up_to..]);
        rendered
    }

    /// The configuration with only [`Self::BUNDLED_FEATURES`]; like the bundle, a document without a
    /// `features` object is passed on as it is.
    fn bundled_part(document: &Value) -> Value {
        let (Some(fields), Some(features)) = (document.as_object(), document["features"].as_object()) else {
            return document.clone();
        };
        let kept: serde_json::Map<String, Value> = features
            .iter()
            .filter(|(name, _)| Self::BUNDLED_FEATURES.contains(&name.as_str()))
            .map(|(name, feature)| (name.clone(), feature.clone()))
            .collect();
        let mut pruned: serde_json::Map<String, Value> = fields
            .iter()
            .filter(|(field, _)| *field != "features")
            .map(|(field, value)| (field.clone(), value.clone()))
            .collect();
        pruned.insert("features".to_owned(), Value::Object(kept));
        Value::Object(pruned)
    }

    /// A JSON string is a valid JavaScript string literal, except that engines before ES2019 reject raw
    /// U+2028 and U+2029, so those two are escaped as well.
    fn javascript_string_literal(text: &str) -> String {
        Value::from(text).to_string().replace('\u{2028}', "\\u2028").replace('\u{2029}', "\\u2029")
    }
}
