//! Parsing DuckDuckGo's Tracker Data Set (TDS) JSON into domain values.

use serde_json::json;
use ztbrowser_core::protection::tracker_data_set::KnownTracker;
use ztbrowser_core::protection::tracker_data_set::TrackerAction;
use ztbrowser_core::protection::tracker_data_set::TrackerDataSet;

/// Given: a published TDS with one blocked tracker (with one valid and two malformed rules), one tracker
/// with an unknown default action, its owning company, and a CNAME cloaking `shop.tracker.com`.
struct PublishedDataSetScenario {
    data_set: TrackerDataSet,
}

impl PublishedDataSetScenario {
    fn given_published_tracker_data_set() -> Self {
        let data_set = TrackerDataSet::from_json(&json!({
            "trackers": {
                "tracker.com": {
                    "domain": "tracker.com",
                    "owner": {"name": "Tracker Inc", "displayName": "Tracker"},
                    "default": "block",
                    "rules": [
                        {"rule": "tracker\\.com/script\\.js", "exceptions": {"domains": ["news.com"], "types": ["script"]}},
                        "not-a-rule-object",
                        {"surrogate": "rule-without-pattern.js"}
                    ]
                },
                "odd.com": {"domain": "odd.com", "default": "sometimes"}
            },
            "entities": {"Tracker Inc": {"displayName": "Tracker", "domains": ["tracker.com", "tracker-cdn.net"]}},
            "domains": {"tracker.com": "Tracker Inc"},
            "cnames": {"metrics.shop.com": "shop.tracker.com"}
        }));
        Self { data_set }
    }

    fn when_looking_up_tracker(&self, domain: &str) -> &KnownTracker {
        self.data_set.tracker(domain).expect("tracker should be parsed")
    }

    fn when_resolving_cname_target(&self, target: &str) -> Option<&KnownTracker> {
        self.data_set.find_tracker_by_cname(target)
    }

    fn when_listing_domains_of_owner(&self, owner: Option<&str>) -> Option<Vec<String>> {
        self.data_set.related_domains(owner).map(<[String]>::to_vec)
    }
}

#[test]
fn should_read_default_action_of_a_tracker() {
    let scenario = PublishedDataSetScenario::given_published_tracker_data_set();

    assert_eq!(scenario.when_looking_up_tracker("tracker.com").default_action, Some(TrackerAction::Block));
}

#[test]
fn should_treat_unknown_default_action_as_missing() {
    let scenario = PublishedDataSetScenario::given_published_tracker_data_set();

    assert_eq!(scenario.when_looking_up_tracker("odd.com").default_action, None);
}

#[test]
fn should_skip_malformed_rules_instead_of_rejecting_the_whole_tracker() {
    let scenario = PublishedDataSetScenario::given_published_tracker_data_set();

    let tracker = scenario.when_looking_up_tracker("tracker.com");

    assert_eq!(
        tracker.rules.iter().map(|rule| rule.pattern.as_str()).collect::<Vec<_>>(),
        ["tracker\\.com/script\\.js"]
    );
}

#[test]
fn should_read_rule_exceptions_as_domains_and_resource_types() {
    let scenario = PublishedDataSetScenario::given_published_tracker_data_set();

    let exceptions = scenario.when_looking_up_tracker("tracker.com").rules[0].exceptions.clone().expect("exceptions");

    assert_eq!(
        (exceptions.domains, exceptions.types),
        (Some(vec!["news.com".to_owned()]), Some(vec!["script".to_owned()]))
    );
}

#[test]
fn should_list_domains_of_the_entity_owning_a_tracker() {
    let scenario = PublishedDataSetScenario::given_published_tracker_data_set();

    assert_eq!(
        scenario.when_listing_domains_of_owner(Some("Tracker Inc")),
        Some(vec!["tracker.com".to_owned(), "tracker-cdn.net".to_owned()])
    );
}

#[test]
fn should_return_no_related_domains_for_unknown_owner() {
    let scenario = PublishedDataSetScenario::given_published_tracker_data_set();

    assert_eq!(scenario.when_listing_domains_of_owner(None), None);
}

#[test]
fn should_find_tracker_behind_cname_by_walking_up_parent_domains() {
    let scenario = PublishedDataSetScenario::given_published_tracker_data_set();

    let tracker = scenario.when_resolving_cname_target("shop.tracker.com").expect("cloaked tracker");

    assert_eq!(tracker.domain.as_deref(), Some("tracker.com"));
}

#[test]
fn should_not_find_tracker_for_unrelated_cname_target() {
    let scenario = PublishedDataSetScenario::given_published_tracker_data_set();

    assert!(scenario.when_resolving_cname_target("cdn.example.org").is_none());
}
