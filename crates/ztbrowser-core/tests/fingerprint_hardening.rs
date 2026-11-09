//! ZTBrowser Guard switches on the fingerprinting defences DuckDuckGo leaves off on macOS, where Safari's own
//! defences cover for them, keeping DuckDuckGo's list of sites each defence is known to break.

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::guard::fingerprint_hardening::FingerprintHardening;
use ztbrowser_core::protection::privacy_configuration::PrivacyConfiguration;

struct HardeningScenario {
    document: Value,
}

impl HardeningScenario {
    fn given_duckduckgo_configuration() -> Self {
        Self {
            document: json!({
                "features": {
                    "fingerprintingCanvas": {
                        "state": "disabled",
                        "exceptions": [{"domain": "chase.com"}],
                        "settings": {"webGl": "enabled"}
                    },
                    "fingerprintingAudio": {"state": "disabled", "exceptions": []},
                    "fingerprintingHardware": {"state": "enabled", "exceptions": []},
                    "fingerprintingScreenSize": {
                        "state": "disabled",
                        "exceptions": [],
                        "settings": {"colorDepth": {"type": "number", "value": 24}}
                    },
                    "clickToLoad": {"state": "disabled", "exceptions": []},
                    "referrer": {"state": "disabled", "exceptions": [], "minSupportedVersion": "9.0.0"}
                }
            }),
        }
    }

    fn when_hardened(&self) -> HardenedConfiguration {
        HardenedConfiguration {
            configuration: PrivacyConfiguration::new(
                FingerprintHardening::applied_to(self.document.clone()),
                "1.214.0",
            ),
        }
    }
}

struct HardenedConfiguration {
    configuration: PrivacyConfiguration,
}

impl HardenedConfiguration {
    fn then_protects(&self, feature: &str, host: &str) {
        assert!(self.configuration.is_enabled_for_site(feature, host, &[]), "{feature} off on {host}");
    }

    fn then_leaves_unprotected(&self, feature: &str, host: &str) {
        assert!(!self.configuration.is_enabled_for_site(feature, host, &[]), "{feature} on on {host}");
    }

    fn then_keeps_settings(&self, feature: &str, expected: &Value) {
        assert_eq!(&self.configuration.document()["features"][feature]["settings"], expected);
    }

    fn then_does_not_describe(&self, feature: &str) {
        assert!(self.configuration.document()["features"].get(feature).is_none(), "{feature} added");
    }
}

#[test]
fn should_switch_on_canvas_noise_duckduckgo_switched_off() {
    HardeningScenario::given_duckduckgo_configuration()
        .when_hardened()
        .then_protects("fingerprintingCanvas", "news.pl");
}

#[test]
fn should_switch_on_audio_noise_duckduckgo_switched_off() {
    HardeningScenario::given_duckduckgo_configuration().when_hardened().then_protects("fingerprintingAudio", "news.pl");
}

#[test]
fn should_switch_on_screen_size_normalisation_duckduckgo_switched_off() {
    HardeningScenario::given_duckduckgo_configuration()
        .when_hardened()
        .then_protects("fingerprintingScreenSize", "news.pl");
}

#[test]
fn should_spare_sites_duckduckgo_knows_the_defence_breaks() {
    HardeningScenario::given_duckduckgo_configuration()
        .when_hardened()
        .then_leaves_unprotected("fingerprintingCanvas", "secure.chase.com");
}

#[test]
fn should_keep_the_settings_duckduckgo_tuned_the_defence_with() {
    HardeningScenario::given_duckduckgo_configuration()
        .when_hardened()
        .then_keeps_settings("fingerprintingScreenSize", &json!({"colorDepth": {"type": "number", "value": 24}}));
}

#[test]
fn should_leave_features_that_are_not_fingerprinting_defences_alone() {
    HardeningScenario::given_duckduckgo_configuration()
        .when_hardened()
        .then_leaves_unprotected("clickToLoad", "news.pl");
}

#[test]
fn should_not_run_a_defence_the_bundled_script_is_too_old_for() {
    HardeningScenario::given_duckduckgo_configuration().when_hardened().then_leaves_unprotected("referrer", "news.pl");
}

#[test]
fn should_not_invent_a_defence_the_configuration_does_not_describe() {
    HardeningScenario::given_duckduckgo_configuration().when_hardened().then_does_not_describe("fingerprintingBattery");
}

#[test]
fn should_pass_the_extra_check_canvas_noise_runs_behind() {
    HardeningScenario::given_duckduckgo_configuration()
        .when_hardened()
        .then_keeps_settings("fingerprintingCanvas", &json!({"webGl": "enabled", "additionalEnabledCheck": "enabled"}));
}
