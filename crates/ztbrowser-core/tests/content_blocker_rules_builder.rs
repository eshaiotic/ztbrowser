//! Translating the Tracker Data Set into WebKit content-blocking rules.
//!
//! The expected rules follow DuckDuckGo's `TrackerRadarKit.ContentBlockerRulesBuilder` (Swift), so a site
//! behaves in ZTBrowser exactly as it does in the DuckDuckGo macOS browser.

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::protection::content_blocker_rule::ContentBlockerRule;
use ztbrowser_core::protection::content_blocker_rules_builder::ContentBlockerRulesBuilder;
use ztbrowser_core::protection::content_blocker_rules_builder::RuleExceptions;
use ztbrowser_core::protection::tracker_allowlist_entry::TrackerAllowlistEntry;
use ztbrowser_core::protection::tracker_data_set::TrackerDataSet;

const SUBDOMAIN_PREFIX: &str = "^[^:]+://+([^:/]+\\.)?";

/// Given: trackers owned (or not) by "Ads Corp", which owns `ads.com` and `ads-cdn.net`.
struct RulesBuilderScenario {
    trackers: Value,
    cnames: Value,
    exceptions: RuleExceptions,
}

impl RulesBuilderScenario {
    fn given_trackers(trackers: Value) -> Self {
        Self { trackers, cnames: json!({}), exceptions: RuleExceptions::default() }
    }

    fn given_ads_corp_tracker_blocked_by_default() -> Self {
        Self::given_trackers(
            json!({"ads.com": {"domain": "ads.com", "owner": {"name": "Ads Corp"}, "default": "block"}}),
        )
    }

    fn with_cname(mut self, alias: &str, target: &str) -> Self {
        self.cnames[alias] = json!(target);
        self
    }

    fn with_site_unprotected_by_user(mut self, domain: &str) -> Self {
        self.exceptions.user_unprotected_domains.push(domain.to_owned());
        self
    }

    fn with_site_temporarily_unprotected(mut self, domain: &str) -> Self {
        self.exceptions.temporary_unprotected_domains.push(domain.to_owned());
        self
    }

    fn with_allowlisted_tracker_url(mut self, rule: &str, sites: Option<&[&str]>) -> Self {
        let domains = sites.map(|sites| sites.iter().map(|&site| site.to_owned()).collect());
        self.exceptions.tracker_allowlist.push(TrackerAllowlistEntry { rule: rule.to_owned(), domains });
        self
    }

    fn when_rules_are_built(&self) -> BuiltRules {
        let data_set = TrackerDataSet::from_json(&json!({
            "trackers": self.trackers,
            "entities": {"Ads Corp": {"domains": ["ads.com", "ads-cdn.net"]}},
            "cnames": self.cnames
        }));
        let rules = ContentBlockerRulesBuilder::new(&data_set).build_rules(&self.exceptions);
        BuiltRules { rules: rules.iter().map(ContentBlockerRule::to_json).collect() }
    }
}

/// The WebKit JSON rule list, with intention-revealing accessors for assertions.
struct BuiltRules {
    rules: Vec<Value>,
}

impl BuiltRules {
    fn rule(&self, position: usize) -> &Value {
        &self.rules[position]
    }

    fn last_rule(&self) -> &Value {
        self.rules.last().expect("at least one rule")
    }

    fn all(&self) -> &[Value] {
        &self.rules
    }

    fn first_rule_filtering(&self, url_fragment: &str) -> &Value {
        self.rules
            .iter()
            .find(|rule| rule["trigger"]["url-filter"].as_str().is_some_and(|filter| filter.contains(url_fragment)))
            .expect("a rule for the fragment")
    }

    /// The expected WebKit `url-filter` for a TDS pattern anchored on any subdomain.
    fn subdomain_filter(suffix: &str) -> String {
        format!("{SUBDOMAIN_PREFIX}{suffix}")
    }

    fn url_filters_without_subdomain_prefix(&self) -> Vec<&str> {
        self.rules
            .iter()
            .filter_map(|rule| rule["trigger"]["url-filter"].as_str())
            .map(|filter| filter.trim_start_matches(SUBDOMAIN_PREFIX))
            .collect()
    }
}

#[test]
fn should_block_tracker_domain_on_third_party_sites_except_its_owners_sites() {
    let rules = RulesBuilderScenario::given_ads_corp_tracker_blocked_by_default().when_rules_are_built();

    assert_eq!(
        rules.rule(0),
        &json!({
            "trigger": {"url-filter": BuiltRules::subdomain_filter("ads\\.com[:/]"), "unless-domain": ["*ads.com", "*ads-cdn.net"], "load-type": ["third-party"]},
            "action": {"type": "block"}
        })
    );
}

#[test]
fn should_let_user_open_tracker_domain_as_top_level_popup() {
    let rules = RulesBuilderScenario::given_ads_corp_tracker_blocked_by_default().when_rules_are_built();

    assert_eq!(
        rules.rule(1),
        &json!({
            "trigger": {
                "url-filter": BuiltRules::subdomain_filter("ads\\.com[:/]"),
                "resource-type": ["popup"],
                "load-type": ["third-party"],
                "load-context": ["top-frame"]
            },
            "action": {"type": "ignore-previous-rules"}
        })
    );
}

#[test]
fn should_produce_no_rules_for_ignored_tracker_without_rules() {
    let rules = RulesBuilderScenario::given_trackers(json!({"cdn.com": {"domain": "cdn.com", "default": "ignore"}}))
        .when_rules_are_built();

    assert_eq!(rules.all(), &[] as &[Value]);
}

#[test]
fn should_block_matching_url_of_ignored_tracker_and_allow_it_as_popup() {
    let rules = RulesBuilderScenario::given_trackers(json!({"cdn.com": {
        "domain": "cdn.com", "owner": {"name": "Ads Corp"}, "default": "ignore",
        "rules": [{"rule": "cdn\\.com/pixel"}]
    }}))
    .when_rules_are_built();

    assert_eq!(
        rules.all(),
        [
            json!({
                "trigger": {"url-filter": BuiltRules::subdomain_filter("cdn\\.com/pixel"), "unless-domain": ["*ads.com", "*ads-cdn.net"], "load-type": ["third-party"]},
                "action": {"type": "block"}
            }),
            json!({
                "trigger": {
                    "url-filter": BuiltRules::subdomain_filter("cdn\\.com/pixel"),
                    "resource-type": ["popup"],
                    "load-type": ["third-party"],
                    "load-context": ["top-frame"]
                },
                "action": {"type": "ignore-previous-rules"}
            }),
        ]
    );
}

#[test]
fn should_block_ignored_tracker_rule_only_on_listed_sites_and_types() {
    let rules = RulesBuilderScenario::given_trackers(json!({"cdn.com": {
        "domain": "cdn.com", "default": "ignore",
        "rules": [{"rule": "cdn\\.com/ad", "options": {"domains": ["news.com"], "types": ["script", "ping"]}}]
    }}))
    .when_rules_are_built();

    assert_eq!(
        rules.all(),
        [json!({
            "trigger": {
                "url-filter": BuiltRules::subdomain_filter("cdn\\.com/ad"),
                "if-domain": ["*news.com"],
                "resource-type": ["script"],
                "load-type": ["third-party"]
            },
            "action": {"type": "block"}
        })]
    );
}

#[test]
fn should_unblock_rule_of_blocked_tracker_on_exception_sites() {
    let rules = RulesBuilderScenario::given_trackers(json!({"ads.com": {
        "domain": "ads.com", "owner": {"name": "Ads Corp"}, "default": "block",
        "rules": [{"rule": "ads\\.com/player\\.js", "exceptions": {"domains": ["video.com"]}}]
    }}))
    .when_rules_are_built();

    assert_eq!(
        rules.rule(2),
        &json!({
            "trigger": {"url-filter": BuiltRules::subdomain_filter("ads\\.com/player\\.js"), "if-domain": ["*video.com"], "load-type": ["third-party"]},
            "action": {"type": "ignore-previous-rules"}
        })
    );
}

#[test]
fn should_keep_absolute_rule_patterns_unchanged() {
    let rules = RulesBuilderScenario::given_trackers(json!({"ads.com": {
        "domain": "ads.com", "default": "block",
        "rules": [{"rule": "https://ads\\.com/x", "action": "ignore"}]
    }}))
    .when_rules_are_built();

    assert_eq!(rules.rule(2)["trigger"]["url-filter"], "https://ads\\.com/x");
}

#[test]
fn should_put_larger_rule_groups_first_and_drop_duplicate_rules() {
    let rules = RulesBuilderScenario::given_trackers(json!({"ads.com": {
        "domain": "ads.com", "default": "block",
        "rules": [
            {"rule": "ads\\.com/a"},
            {"rule": "ads\\.com/b", "options": {"domains": ["x.com"]}, "exceptions": {"domains": ["y.com"]}},
            {"rule": "ads\\.com/a"}
        ]
    }}))
    .when_rules_are_built();

    assert_eq!(
        rules.url_filters_without_subdomain_prefix(),
        ["ads\\.com[:/]", "ads\\.com[:/]", "ads\\.com/b", "ads\\.com/b", "ads\\.com/b", "ads\\.com/a"]
    );
}

#[test]
fn should_block_cname_cloaked_tracker_also_as_first_party() {
    let rules = RulesBuilderScenario::given_ads_corp_tracker_blocked_by_default()
        .with_cname("metrics.shop.com", "eu.ads.com")
        .when_rules_are_built();

    assert_eq!(
        rules.first_rule_filtering("metrics\\.shop\\.com")["trigger"]["load-type"],
        json!(["first-party", "third-party"])
    );
}

#[test]
fn should_disable_all_tracker_rules_on_unprotected_sites() {
    let rules = RulesBuilderScenario::given_ads_corp_tracker_blocked_by_default()
        .with_site_unprotected_by_user("mybank.com")
        .with_site_temporarily_unprotected("broken.com")
        .when_rules_are_built();

    assert_eq!(
        rules.last_rule(),
        &json!({
            "trigger": {"url-filter": ".*", "if-domain": ["mybank.com", "*broken.com"], "load-type": ["first-party", "third-party"]},
            "action": {"type": "ignore-previous-rules"}
        })
    );
}

#[test]
fn should_allow_allowlisted_tracker_url_on_every_site() {
    let rules = RulesBuilderScenario::given_ads_corp_tracker_blocked_by_default()
        .with_allowlisted_tracker_url("ads.com/sdk.js", None)
        .when_rules_are_built();

    assert_eq!(
        rules.last_rule(),
        &json!({
            "trigger": {"url-filter": BuiltRules::subdomain_filter("ads\\.com(:[0-9]+)?\\/sdk\\.js.*"), "load-type": ["first-party", "third-party"]},
            "action": {"type": "ignore-previous-rules"}
        })
    );
}

#[test]
fn should_allow_allowlisted_tracker_url_only_on_listed_sites() {
    let rules = RulesBuilderScenario::given_ads_corp_tracker_blocked_by_default()
        .with_allowlisted_tracker_url("ads.com/sdk.js", Some(&["shop.com"]))
        .when_rules_are_built();

    assert_eq!(rules.last_rule()["trigger"]["if-domain"], json!(["*shop.com"]));
}
