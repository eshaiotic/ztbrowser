//! Preparing DuckDuckGo's Content Scope Scripts (fingerprinting protection, GPC, ...) for injection.
//!
//! The vendored script is a template: the browser replaces placeholders with the privacy configuration,
//! the sites the user switched protection off for, and per-session preferences, exactly like
//! `ContentScopeUserScript.generateSource` in DuckDuckGo's macOS app.

use std::collections::BTreeSet;
use std::path::Path;

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::protection::content_scope_preferences::ContentScopePreferences;
use ztbrowser_core::protection::content_scope_script::ContentScopeScript;
use ztbrowser_core::protection::privacy_configuration::PrivacyConfiguration;

struct ContentScopeScenario {
    template: String,
    configuration: PrivacyConfiguration,
    user_unprotected_domains: Vec<String>,
    preferences: ContentScopePreferences,
}

impl ContentScopeScenario {
    fn given_template(template: &str) -> Self {
        Self {
            template: template.to_owned(),
            configuration: PrivacyConfiguration::new(json!({"version": 7, "features": {}}), "1.214.0"),
            user_unprotected_domains: vec![],
            preferences: ContentScopePreferences {
                global_privacy_control: true,
                session_key: "session-key-1".to_owned(),
                message_secret: "secret-1".to_owned(),
                language: "pl".to_owned(),
                compatibility_version: "1.214.0".to_owned(),
            },
        }
    }

    fn given_vendored_content_scope_script() -> Self {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/content-scope-scripts/contentScope.js");
        Self::given_template(&std::fs::read_to_string(path).expect("vendored contentScope.js"))
    }

    fn with_configuration(mut self, document: Value) -> Self {
        self.configuration = PrivacyConfiguration::new(document, "1.214.0");
        self
    }

    fn with_protection_switched_off_by_user_on(mut self, domain: &str) -> Self {
        self.user_unprotected_domains.push(domain.to_owned());
        self
    }

    fn with_global_privacy_control_switched_off(mut self) -> Self {
        self.preferences.global_privacy_control = false;
        self
    }

    fn when_script_is_rendered(&self) -> RenderedScript {
        let script = ContentScopeScript::new(&self.template);
        RenderedScript { source: script.render(&self.configuration, &self.user_unprotected_domains, &self.preferences) }
    }
}

struct RenderedScript {
    source: String,
}

impl RenderedScript {
    fn then_reads(&self, expected: &str) {
        assert_eq!(self.source, expected);
    }

    fn then_has_no_placeholders_left(&self) {
        for placeholder in
            ["$CONTENT_SCOPE_JSON$", "$CONTENT_SCOPE$", "$USER_UNPROTECTED_DOMAINS$", "$USER_PREFERENCES$"]
        {
            assert!(!self.source.contains(placeholder), "{placeholder} left in the script");
        }
    }

    fn as_json(&self) -> Value {
        serde_json::from_str(&self.source).expect("rendered text is JSON")
    }

    /// The configuration embedded by a `$CONTENT_SCOPE_JSON$` template: a string literal holding JSON.
    fn embedded_configuration(&self) -> Value {
        let text: String = serde_json::from_str(&self.source).expect("rendered text is a string literal");
        serde_json::from_str(&text).expect("the literal holds JSON")
    }

    fn then_sends_exactly_the_features(&self, expected: &BTreeSet<String>) {
        let configuration = self.embedded_configuration();
        let sent: BTreeSet<String> =
            configuration["features"].as_object().expect("features object").keys().cloned().collect();
        assert_eq!(&sent, expected);
    }
}

/// The features DuckDuckGo's `apple` build of Content Scope Scripts keeps from the configuration, read from
/// the vendored bundle the same way its `pruneConfigFeatures` computes them: `platformSupport.apple` (with
/// `baseFeatures` spread in) plus the `featureConfigDependencies` of those features.
struct VendoredBundle {
    source: String,
}

impl VendoredBundle {
    fn read() -> Self {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/content-scope-scripts/contentScope.js");
        Self { source: std::fs::read_to_string(path).expect("vendored contentScope.js") }
    }

    fn kept_features(&self) -> BTreeSet<String> {
        let apple = self.section("    apple: [", "],");
        let spreads: Vec<&str> = apple.split("...").skip(1).collect();
        assert!(
            spreads.iter().all(|spread| spread.starts_with("baseFeatures")),
            "platformSupport.apple spreads a list this test does not read: {apple}"
        );
        let mut bundled: BTreeSet<String> = Self::quoted_names(apple).collect();
        if !spreads.is_empty() {
            bundled.extend(Self::quoted_names(self.section("var baseFeatures", ");")));
        }
        let mut kept = bundled.clone();
        for line in self.section("var featureConfigDependencies = {", "};").lines() {
            if let Some((feature, dependencies)) = line.split_once(':')
                && bundled.contains(feature.trim())
            {
                kept.extend(Self::quoted_names(dependencies));
            }
        }
        kept
    }

    fn section(&self, start: &str, end: &str) -> &str {
        let from = self.source.find(start).unwrap_or_else(|| panic!("bundle has no `{start}`")) + start.len();
        let length = self.source[from..].find(end).unwrap_or_else(|| panic!("`{start}` never ends with `{end}`"));
        &self.source[from..from + length]
    }

    fn quoted_names(text: &str) -> impl Iterator<Item = String> + '_ {
        text.split('"').skip(1).step_by(2).map(str::to_owned)
    }
}

#[test]
fn should_embed_privacy_configuration_as_json_string_literal() {
    ContentScopeScenario::given_template("JSON.parse($CONTENT_SCOPE_JSON$)")
        .when_script_is_rendered()
        .then_reads(r#"JSON.parse("{\"features\":{},\"version\":7}")"#);
}

#[test]
fn should_embed_privacy_configuration_as_object_for_older_bundles() {
    ContentScopeScenario::given_template("const c = $CONTENT_SCOPE$;")
        .when_script_is_rendered()
        .then_reads(r#"const c = JSON.parse("{\"features\":{},\"version\":7}");"#);
}

#[test]
fn should_list_sites_the_user_unprotected() {
    ContentScopeScenario::given_template("$USER_UNPROTECTED_DOMAINS$")
        .with_protection_switched_off_by_user_on("mbank.pl")
        .when_script_is_rendered()
        .then_reads(r#"["mbank.pl"]"#);
}

#[test]
fn should_describe_session_as_macos_app_of_the_compatibility_version() {
    let preferences = ContentScopeScenario::given_template("$USER_PREFERENCES$").when_script_is_rendered().as_json();

    assert_eq!(
        preferences,
        json!({
            "globalPrivacyControlValue": true,
            "debug": false,
            "sessionKey": "session-key-1",
            "messageSecret": "secret-1",
            "language": "pl",
            "platform": {"name": "macos", "internal": false, "version": "1.214.0"},
            "currentCohorts": [],
            "messagingContextName": "contentScopeScripts"
        })
    );
}

#[test]
fn should_pass_global_privacy_control_choice() {
    let preferences = ContentScopeScenario::given_template("$USER_PREFERENCES$")
        .with_global_privacy_control_switched_off()
        .when_script_is_rendered()
        .as_json();

    assert_eq!(preferences["globalPrivacyControlValue"], json!(false));
}

#[test]
fn should_escape_line_separators_that_older_javascript_engines_reject() {
    ContentScopeScenario::given_template("$CONTENT_SCOPE_JSON$")
        .with_configuration(json!({"note": "a\u{2028}b\u{2029}c"}))
        .when_script_is_rendered()
        .then_reads(r#""{\"note\":\"a\u2028b\u2029c\"}""#);
}

#[test]
fn should_not_expand_placeholders_that_appear_inside_the_configuration() {
    ContentScopeScenario::given_template("$CONTENT_SCOPE_JSON$|$USER_UNPROTECTED_DOMAINS$")
        .with_configuration(json!({"trap": "$USER_UNPROTECTED_DOMAINS$"}))
        .when_script_is_rendered()
        .then_reads(r#""{\"trap\":\"$USER_UNPROTECTED_DOMAINS$\"}"|[]"#);
}

#[test]
fn should_leave_out_features_the_bundle_discards_before_every_frame_parses_them() {
    let rendered = ContentScopeScenario::given_template("$CONTENT_SCOPE_JSON$")
        .with_configuration(json!({
            "version": 7,
            "unprotectedTemporary": [{"domain": "broken.example"}],
            "features": {
                "gpc": {"state": "enabled"},
                "trackerAllowlist": {"state": "enabled"},
                "autoconsent": {"state": "enabled"},
                "eventHub": {"state": "enabled"}
            }
        }))
        .when_script_is_rendered();

    assert_eq!(
        rendered.embedded_configuration(),
        json!({
            "version": 7,
            "unprotectedTemporary": [{"domain": "broken.example"}],
            "features": {"gpc": {"state": "enabled"}, "trackerAllowlist": {"state": "enabled"}}
        })
    );
}

#[test]
fn should_send_every_feature_the_vendored_bundle_keeps_and_nothing_else() {
    let kept = VendoredBundle::read().kept_features();
    let mut features: serde_json::Map<String, Value> =
        kept.iter().map(|name| (name.clone(), json!({"state": "enabled"}))).collect();
    features.insert("autoconsent".to_owned(), json!({"state": "enabled"}));

    ContentScopeScenario::given_template("$CONTENT_SCOPE_JSON$")
        .with_configuration(json!({"version": 7, "features": features}))
        .when_script_is_rendered()
        .then_sends_exactly_the_features(&kept);
}

#[test]
fn should_fill_every_placeholder_of_the_vendored_script() {
    ContentScopeScenario::given_vendored_content_scope_script()
        .when_script_is_rendered()
        .then_has_no_placeholders_left();
}
