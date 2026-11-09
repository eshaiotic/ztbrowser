//! DuckDuckGo's tracker rules, applied to each request exactly as the rule list they build means them.
//!
//! The same rule list is compiled by WebKit in the GTK edition; here it is evaluated directly, with the order
//! of the rules and the effect of `ignore-previous-rules` kept, so a site DuckDuckGo fixed on macOS is fixed
//! here too.

use std::sync::Arc;

use serde_json::json;
use ztbrowser_core::network::content_blocker_request_policy::ContentBlockerRequestPolicy;
use ztbrowser_core::network::network_request::NetworkRequest;
use ztbrowser_core::network::request_decision::RequestDecision;
use ztbrowser_core::network::request_policy::RequestPolicy;
use ztbrowser_core::network::resource_type::ResourceType;
use ztbrowser_core::protection::privacy_configuration::PrivacyConfiguration;
use ztbrowser_core::protection::tracker_data_set::TrackerDataSet;
use ztbrowser_core::web_app::registrable_domain_lookup::RegistrableDomainLookup;

struct KnownSuffixes;

impl RegistrableDomainLookup for KnownSuffixes {
    fn is_public_suffix(&self, domain: &str) -> bool {
        matches!(domain, "com" | "net" | "org")
    }
}

/// Given the published data of a small web:
/// - `tracker.com` (Tracker Inc, also owning `tracker-sites.net`) is blocked by default, except that its
///   `/captcha/` pages are allowlisted for every site and its `/embed` pages on `video.org`;
/// - `mixed.com` is ignored by default, but its `/collect` URL is blocked, `/collect/ok` excepted;
/// - `scoped.com` is ignored by default, but blocked for scripts and only on `shop.org`;
/// - `metrics.example.org` is a first-party alias that points at `collector.tracker.com`;
/// - `broken.org` is on DuckDuckGo's temporary list of sites where blocking is off.
struct RulesScenario {
    policy: ContentBlockerRequestPolicy,
}

impl RulesScenario {
    fn given_published_data() -> Self {
        let data_set = TrackerDataSet::from_json(&json!({
            "trackers": {
                "tracker.com": {"domain": "tracker.com", "owner": {"name": "Tracker Inc"}, "default": "block", "rules": []},
                "mixed.com": {"domain": "mixed.com", "owner": {"name": "Mixed"}, "default": "ignore", "rules": [
                    {"rule": "mixed\\.com\\/collect"},
                    {"rule": "mixed\\.com\\/collect\\/ok", "action": "ignore"}
                ]},
                "scoped.com": {"domain": "scoped.com", "owner": {"name": "Scoped"}, "default": "ignore", "rules": [
                    {"rule": "scoped\\.com\\/tag", "options": {"domains": ["shop.org"], "types": ["script"]}}
                ]}
            },
            "entities": {"Tracker Inc": {"domains": ["tracker.com", "tracker-sites.net"]}},
            "cnames": {"metrics.example.org": "collector.tracker.com"}
        }));
        let configuration = PrivacyConfiguration::new(
            json!({
                "unprotectedTemporary": [{"domain": "broken.org"}],
                "features": {
                    "contentBlocking": {"state": "enabled", "exceptions": []},
                    "trackerAllowlist": {"state": "enabled", "settings": {"allowlistedTrackers": {
                        "tracker.com": {"rules": [
                            {"rule": "tracker.com/captcha/", "domains": ["<all>"]},
                            {"rule": "tracker.com/embed", "domains": ["video.org"]}
                        ]}
                    }}}
                }
            }),
            "1.214.0",
        );
        Self { policy: ContentBlockerRequestPolicy::from_documents(&data_set, &configuration, Arc::new(KnownSuffixes)) }
    }

    fn when_page_requests(&self, page: &str, address: &str, kind: ResourceType) -> RequestDecision {
        self.policy.decide(&NetworkRequest::new(address, Some(page), kind, 1))
    }

    fn when_page_loads_image(&self, page: &str, address: &str) -> RequestDecision {
        self.when_page_requests(page, address, ResourceType::Image)
    }
}

const NEWS: &str = "https://news.example.org/story";

#[test]
fn should_block_a_third_party_request_to_a_domain_blocked_by_default() {
    let decision = RulesScenario::given_published_data().when_page_loads_image(NEWS, "https://tracker.com/pixel.gif");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_block_a_subdomain_of_a_blocked_domain() {
    let decision =
        RulesScenario::given_published_data().when_page_loads_image(NEWS, "https://cdn.eu.tracker.com/p.gif");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_not_block_a_look_alike_domain() {
    let decision = RulesScenario::given_published_data().when_page_loads_image(NEWS, "https://nottracker.com/p.gif");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_not_block_the_company_on_its_own_sites() {
    let decision = RulesScenario::given_published_data()
        .when_page_loads_image("https://www.tracker-sites.net/", "https://tracker.com/pixel.gif");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_pass_what_duckduckgo_allowlists_for_every_site() {
    let decision = RulesScenario::given_published_data().when_page_requests(
        NEWS,
        "https://tracker.com/captcha/i.js",
        ResourceType::Script,
    );
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_pass_what_duckduckgo_allowlists_only_on_the_listed_site() {
    let scenario = RulesScenario::given_published_data();
    let on_listed = scenario.when_page_loads_image("https://www.video.org/watch", "https://tracker.com/embed/1");
    let elsewhere = scenario.when_page_loads_image(NEWS, "https://tracker.com/embed/1");
    assert_eq!((on_listed, elsewhere), (RequestDecision::Pass, RequestDecision::Block));
}

#[test]
fn should_still_block_the_rest_of_a_tracker_that_has_an_allowlisted_path() {
    let decision = RulesScenario::given_published_data().when_page_loads_image(NEWS, "https://tracker.com/other.gif");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_pass_a_domain_whose_default_is_ignore() {
    let decision = RulesScenario::given_published_data().when_page_loads_image(NEWS, "https://mixed.com/logo.png");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_block_the_url_a_rule_names_on_a_domain_that_is_otherwise_ignored() {
    let decision = RulesScenario::given_published_data().when_page_loads_image(NEWS, "https://mixed.com/collect?id=1");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_let_a_later_ignore_rule_cancel_an_earlier_block() {
    let decision = RulesScenario::given_published_data().when_page_loads_image(NEWS, "https://mixed.com/collect/ok");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_apply_a_scoped_rule_only_to_its_site_and_request_kind() {
    let scenario = RulesScenario::given_published_data();
    let script_on_site =
        scenario.when_page_requests("https://shop.org/", "https://scoped.com/tag.js", ResourceType::Script);
    let image_on_site = scenario.when_page_loads_image("https://shop.org/", "https://scoped.com/tag.gif");
    let script_elsewhere = scenario.when_page_requests(NEWS, "https://scoped.com/tag.js", ResourceType::Script);
    assert_eq!(
        (script_on_site, image_on_site, script_elsewhere),
        (RequestDecision::Block, RequestDecision::Pass, RequestDecision::Pass)
    );
}

#[test]
fn should_block_a_first_party_alias_that_points_at_a_tracker() {
    let decision = RulesScenario::given_published_data()
        .when_page_loads_image("https://www.example.org/", "https://metrics.example.org/p.gif");
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_stand_down_on_the_sites_duckduckgo_lists_as_broken() {
    let decision = RulesScenario::given_published_data()
        .when_page_loads_image("https://www.broken.org/", "https://tracker.com/pixel.gif");
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_leave_a_navigation_to_a_tracker_alone() {
    let decision = ContentBlockerRequestPolicy::from_documents(
        &TrackerDataSet::from_json(
            &json!({"trackers": {"tracker.com": {"domain": "tracker.com", "default": "block", "rules": []}}}),
        ),
        &PrivacyConfiguration::new(json!({"features": {}}), "1.0.0"),
        Arc::new(KnownSuffixes),
    )
    .decide(&NetworkRequest::new("https://tracker.com/", None, ResourceType::MainFrame, 1));
    assert_eq!(decision, RequestDecision::Pass);
}

#[test]
fn should_block_a_tracker_asked_for_by_a_page_it_does_not_know() {
    let policy = RulesScenario::given_published_data().policy;
    let decision = policy.decide(&NetworkRequest::new("https://tracker.com/pixel.gif", None, ResourceType::Image, 1));
    assert_eq!(decision, RequestDecision::Block);
}

#[test]
fn should_skip_a_rule_whose_pattern_cannot_be_read_and_keep_the_others() {
    let data_set = TrackerDataSet::from_json(&json!({
        "trackers": {
            "broken-rule.com": {"domain": "broken-rule.com", "default": "ignore", "rules": [{"rule": "broken-rule\\.com\\/(unclosed"}]},
            "tracker.com": {"domain": "tracker.com", "default": "block", "rules": []}
        }
    }));
    let policy = ContentBlockerRequestPolicy::from_documents(
        &data_set,
        &PrivacyConfiguration::new(json!({"features": {}}), "1.0.0"),
        Arc::new(KnownSuffixes),
    );
    let broken =
        policy.decide(&NetworkRequest::new("https://broken-rule.com/(unclosed", Some(NEWS), ResourceType::Image, 1));
    let blocked = policy.decide(&NetworkRequest::new("https://tracker.com/p.gif", Some(NEWS), ResourceType::Image, 1));
    assert_eq!((broken, blocked), (RequestDecision::Pass, RequestDecision::Block));
}
