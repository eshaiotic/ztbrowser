//! Where tracker blocking stands down: DuckDuckGo's own exception lists plus the user's unprotected sites.

use serde_json::json;
use ztbrowser_core::protection::content_blocker_rules_builder::RuleExceptions;
use ztbrowser_core::protection::privacy_configuration::PrivacyConfiguration;

struct RuleExceptionsScenario {
    configuration: PrivacyConfiguration,
}

impl RuleExceptionsScenario {
    fn given_configuration_with_exceptions() -> Self {
        Self {
            configuration: PrivacyConfiguration::new(
                json!({
                    "unprotectedTemporary": [{"domain": "broken-shop.com"}],
                    "features": {
                        "contentBlocking": {"state": "enabled", "exceptions": [{"domain": "bank-login.pl"}]},
                        "trackerAllowlist": {"state": "enabled", "settings": {"allowlistedTrackers": {
                            "maps.example": {"rules": [{"rule": "maps.example/api", "domains": ["<all>"]}]}
                        }}}
                    }
                }),
                "1.214.0",
            ),
        }
    }

    fn when_user_unprotected(&self, domains: &[&str]) -> RuleExceptions {
        let user_domains = domains.iter().map(|domain| (*domain).to_owned()).collect();
        RuleExceptions::from_configuration(&self.configuration, user_domains)
    }
}

#[test]
fn should_merge_temporary_list_with_content_blocking_exceptions() {
    let exceptions = RuleExceptionsScenario::given_configuration_with_exceptions().when_user_unprotected(&[]);

    assert_eq!(exceptions.temporary_unprotected_domains, ["broken-shop.com", "bank-login.pl"]);
}

#[test]
fn should_keep_user_unprotected_sites_and_tracker_allowlist() {
    let exceptions = RuleExceptionsScenario::given_configuration_with_exceptions().when_user_unprotected(&["news.pl"]);

    assert_eq!((exceptions.user_unprotected_domains.len(), exceptions.tracker_allowlist.len()), (1, 1));
}
