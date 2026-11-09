//! Reading DuckDuckGo's remote privacy configuration (which protections are on, and where not).

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::protection::privacy_configuration::PrivacyConfiguration;
use ztbrowser_core::protection::tracker_allowlist_entry::TrackerAllowlistEntry;

/// Given: a remote configuration evaluated as macOS app 1.214.0 would evaluate it.
struct RemoteConfigurationScenario {
    configuration: PrivacyConfiguration,
    user_unprotected_domains: Vec<String>,
}

impl RemoteConfigurationScenario {
    fn given_remote_configuration() -> Self {
        Self::given_configuration_document(json!({
            "version": 1_761_900_000_000_u64,
            "unprotectedTemporary": [{"domain": "broken.com"}, {"domain": "  "}],
            "features": {
                "contentBlocking": {"state": "enabled", "exceptions": [{"domain": "strava.com", "reason": "x"}]},
                "trackingParameters": {"state": "enabled", "settings": {"parameters": ["fbclid", "gclid"]}},
                "gpc": {"state": "disabled"},
                "debugTool": {"state": "internal"},
                "futureFeature": {"state": "enabled", "minSupportedVersion": "1.300.0"},
                "olderFeature": {"state": "enabled", "minSupportedVersion": "1.99.2"},
                "trackerAllowlist": {
                    "state": "enabled",
                    "settings": {"allowlistedTrackers": {"ads.com": {"rules": [
                        {"rule": "ads.com/sdk.js", "domains": ["<all>"]},
                        {"rule": "ads.com/player", "domains": ["Shop.COM", "żółw.pl"]}
                    ]}}}
                }
            }
        }))
    }

    fn given_configuration_document(document: Value) -> Self {
        Self { configuration: PrivacyConfiguration::new(document, "1.214.0"), user_unprotected_domains: vec![] }
    }

    fn with_protection_switched_off_by_user_on(mut self, domain: &str) -> Self {
        self.user_unprotected_domains.push(domain.to_owned());
        self
    }

    fn when_checking_feature(&self, feature: &str) -> FeatureVerdict {
        FeatureVerdict { feature: feature.to_owned(), enabled: self.configuration.is_enabled(feature) }
    }

    fn when_checking_feature_on_site(&self, feature: &str, host: &str) -> FeatureVerdict {
        let enabled = self.configuration.is_enabled_for_site(feature, host, &self.user_unprotected_domains);
        FeatureVerdict { feature: format!("{feature} on {host}"), enabled }
    }

    fn configuration(&self) -> &PrivacyConfiguration {
        &self.configuration
    }
}

struct FeatureVerdict {
    feature: String,
    enabled: bool,
}

impl FeatureVerdict {
    fn then_is_enabled(&self) {
        assert!(self.enabled, "{} should be enabled", self.feature);
    }

    fn then_is_disabled(&self) {
        assert!(!self.enabled, "{} should be disabled", self.feature);
    }
}

#[test]
fn should_expose_configuration_version() {
    assert_eq!(RemoteConfigurationScenario::given_remote_configuration().configuration().version(), 1_761_900_000_000);
}

#[test]
fn should_report_feature_with_enabled_state_as_enabled() {
    RemoteConfigurationScenario::given_remote_configuration()
        .when_checking_feature("contentBlocking")
        .then_is_enabled();
}

#[test]
fn should_report_disabled_feature_as_disabled() {
    RemoteConfigurationScenario::given_remote_configuration().when_checking_feature("gpc").then_is_disabled();
}

#[test]
fn should_treat_internal_only_feature_as_disabled() {
    RemoteConfigurationScenario::given_remote_configuration().when_checking_feature("debugTool").then_is_disabled();
}

#[test]
fn should_treat_missing_feature_as_disabled() {
    RemoteConfigurationScenario::given_remote_configuration().when_checking_feature("noSuchFeature").then_is_disabled();
}

#[test]
fn should_keep_feature_disabled_until_compatibility_version_reaches_minimum() {
    RemoteConfigurationScenario::given_remote_configuration().when_checking_feature("futureFeature").then_is_disabled();
}

#[test]
fn should_compare_versions_numerically_not_alphabetically() {
    RemoteConfigurationScenario::given_remote_configuration().when_checking_feature("olderFeature").then_is_enabled();
}

#[test]
fn should_list_feature_exception_domains() {
    let scenario = RemoteConfigurationScenario::given_remote_configuration();

    assert_eq!(scenario.configuration().exception_domains("contentBlocking"), ["strava.com"]);
}

#[test]
fn should_ignore_blank_temporary_unprotected_domains() {
    let scenario = RemoteConfigurationScenario::given_remote_configuration();

    assert_eq!(scenario.configuration().temporary_unprotected_domains(), ["broken.com"]);
}

#[test]
fn should_disable_feature_on_subdomain_of_exception_site() {
    RemoteConfigurationScenario::given_remote_configuration()
        .when_checking_feature_on_site("contentBlocking", "www.strava.com")
        .then_is_disabled();
}

#[test]
fn should_disable_feature_on_temporarily_unprotected_site() {
    RemoteConfigurationScenario::given_remote_configuration()
        .when_checking_feature_on_site("contentBlocking", "shop.broken.com")
        .then_is_disabled();
}

#[test]
fn should_disable_feature_on_site_the_user_unprotected() {
    RemoteConfigurationScenario::given_remote_configuration()
        .with_protection_switched_off_by_user_on("mybank.com")
        .when_checking_feature_on_site("contentBlocking", "mybank.com")
        .then_is_disabled();
}

#[test]
fn should_keep_feature_enabled_on_unrelated_site() {
    RemoteConfigurationScenario::given_remote_configuration()
        .when_checking_feature_on_site("contentBlocking", "example.com")
        .then_is_enabled();
}

#[test]
fn should_not_match_site_that_merely_ends_with_exception_text() {
    RemoteConfigurationScenario::given_remote_configuration()
        .when_checking_feature_on_site("contentBlocking", "notstrava.com")
        .then_is_enabled();
}

#[test]
fn should_read_tracker_allowlist_with_all_sites_marker() {
    let scenario = RemoteConfigurationScenario::given_remote_configuration();

    assert_eq!(
        scenario.configuration().tracker_allowlist()[0],
        TrackerAllowlistEntry { rule: "ads.com/sdk.js".to_owned(), domains: None }
    );
}

#[test]
fn should_normalize_allowlist_domains_for_webkit() {
    let scenario = RemoteConfigurationScenario::given_remote_configuration();

    assert_eq!(
        scenario.configuration().tracker_allowlist()[1].domains,
        Some(vec!["shop.com".to_owned(), "xn--w-uga1v8h.pl".to_owned()])
    );
}

#[test]
fn should_list_tracking_parameters_to_strip() {
    let scenario = RemoteConfigurationScenario::given_remote_configuration();

    let mut parameters: Vec<String> = scenario.configuration().tracking_parameters().into_iter().collect();
    parameters.sort();

    assert_eq!(parameters, ["fbclid", "gclid"]);
}

#[test]
fn should_list_no_tracking_parameters_when_feature_disabled() {
    let scenario = RemoteConfigurationScenario::given_configuration_document(json!({
        "features": {"trackingParameters": {"state": "disabled", "settings": {"parameters": ["fbclid"]}}}
    }));

    assert!(scenario.configuration().tracking_parameters().is_empty());
}
