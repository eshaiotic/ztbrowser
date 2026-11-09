//! Upgrading plain-HTTP navigations to HTTPS, falling back once a site proves it has no HTTPS.

use regex::Regex;
use regex::RegexBuilder;
use ztbrowser_core::navigation::https_upgrade_policy::HttpsUpgradePolicy;
use ztbrowser_core::protection::content_blocker_rule::ContentBlockerRule;

/// Given: the privacy configuration excludes `legacy-portal.com` from HTTPS upgrades.
struct HttpsUpgradeScenario {
    policy: HttpsUpgradePolicy,
}

impl HttpsUpgradeScenario {
    fn given_https_first_browsing() -> Self {
        Self { policy: HttpsUpgradePolicy::new(&["legacy-portal.com".to_owned()]) }
    }

    fn after_https_failed_for(mut self, url: &str) -> Self {
        self.policy.record_failed_upgrade(url);
        self
    }

    /// `(action, url-filter)` of every WebKit rule, in order.
    fn when_upgrade_rules_are_built(&self) -> Vec<(String, String)> {
        let rules = self.policy.content_rules();
        rules
            .iter()
            .map(ContentBlockerRule::to_json)
            .map(|rule| {
                let filter = rule["trigger"]["url-filter"].as_str().unwrap_or_default().to_owned();
                (rule["action"]["type"].as_str().unwrap_or_default().to_owned(), filter)
            })
            .collect()
    }

    /// The exemption filters compiled as WebKit matches them: anywhere in the URL, case-insensitively.
    fn when_exemptions_are_compiled(&self) -> PlainHttpExemptions {
        let filters = self.when_upgrade_rules_are_built().into_iter().skip(1).map(|(_, filter)| filter);
        PlainHttpExemptions {
            filters: filters
                .map(|filter| RegexBuilder::new(&filter).case_insensitive(true).build().expect("valid url-filter"))
                .collect(),
        }
    }

    fn when_navigating_to(&self, url: &str) -> NavigationOutcome {
        NavigationOutcome { url: url.to_owned(), upgraded: self.policy.upgraded(url) }
    }
}

struct PlainHttpExemptions {
    filters: Vec<Regex>,
}

impl PlainHttpExemptions {
    fn then_keep_on_plain_http(&self, url: &str) {
        assert!(self.filters.iter().any(|filter| filter.is_match(url)), "not exempted: {url}");
    }

    fn then_upgrade(&self, url: &str) {
        let exempting: Vec<&str> = self.filters.iter().filter(|f| f.is_match(url)).map(Regex::as_str).collect();
        assert!(exempting.is_empty(), "{url} exempted by {exempting:?}");
    }
}

struct NavigationOutcome {
    url: String,
    upgraded: Option<String>,
}

impl NavigationOutcome {
    fn then_loads(&self, expected: &str) {
        assert_eq!(self.upgraded.as_deref(), Some(expected));
    }

    fn then_proceeds_unchanged(&self) {
        assert_eq!(self.upgraded, None, "url: {}", self.url);
    }
}

#[test]
fn should_upgrade_plain_http_navigation() {
    HttpsUpgradeScenario::given_https_first_browsing()
        .when_navigating_to("http://news.com/article?id=1")
        .then_loads("https://news.com/article?id=1");
}

#[test]
fn should_not_change_navigation_that_is_already_https() {
    HttpsUpgradeScenario::given_https_first_browsing()
        .when_navigating_to("https://news.com/")
        .then_proceeds_unchanged();
}

#[test]
fn should_not_upgrade_local_network_addresses() {
    let scenario = HttpsUpgradeScenario::given_https_first_browsing();

    for url in
        ["http://localhost:3000/", "http://192.168.0.1/", "http://[::1]/", "http://printer.local/", "http://nas/"]
    {
        scenario.when_navigating_to(url).then_proceeds_unchanged();
    }
}

#[test]
fn should_leave_onion_services_on_plain_http_because_tor_encrypts_them() {
    HttpsUpgradeScenario::given_https_first_browsing()
        .when_navigating_to("http://duckduckgogg42xjoc72x3sjasowoarfbgcmvfimaftt6twagswzczad.onion/")
        .then_proceeds_unchanged();
}

#[test]
fn should_upgrade_ordinary_site_whose_name_merely_contains_onion() {
    HttpsUpgradeScenario::given_https_first_browsing()
        .when_navigating_to("http://onion.example.com/")
        .then_loads("https://onion.example.com/");
}

#[test]
fn should_not_upgrade_site_excluded_by_privacy_configuration() {
    HttpsUpgradeScenario::given_https_first_browsing()
        .when_navigating_to("http://www.legacy-portal.com/")
        .then_proceeds_unchanged();
}

#[test]
fn should_stop_upgrading_host_after_failed_upgrade() {
    HttpsUpgradeScenario::given_https_first_browsing()
        .after_https_failed_for("https://old-blog.com/post")
        .when_navigating_to("http://old-blog.com/other")
        .then_proceeds_unchanged();
}

#[test]
fn should_upgrade_every_plain_http_request_before_it_is_sent() {
    let rules = HttpsUpgradeScenario::given_https_first_browsing().when_upgrade_rules_are_built();

    assert_eq!(rules[0], ("make-https".to_owned(), "^http://".to_owned()));
}

#[test]
fn should_express_every_exception_as_ignore_previous_rules() {
    let rules = HttpsUpgradeScenario::given_https_first_browsing().when_upgrade_rules_are_built();

    assert!(rules[1..].iter().all(|(action, _)| action == "ignore-previous-rules"));
}

#[test]
fn should_keep_local_network_and_excluded_hosts_on_plain_http() {
    let exemptions = HttpsUpgradeScenario::given_https_first_browsing()
        .after_https_failed_for("https://old-blog.com/post")
        .when_exemptions_are_compiled();

    for url in [
        "http://192.168.0.1/",
        "http://127.0.0.1:8080/admin?x=1",
        "http://[::1]/",
        "http://nas/",
        "http://nas:5000/files",
        "http://localhost:3000/",
        "http://printer.local/",
        "http://router.home.arpa/",
        "http://abcdefgh.onion/",
        "http://www.abcdefgh.onion:8080/forum",
        "http://www.legacy-portal.com/",
        "http://old-blog.com/other",
    ] {
        exemptions.then_keep_on_plain_http(url);
    }
}

#[test]
fn should_upgrade_public_sites_hiding_behind_user_info_that_looks_like_an_exempt_host() {
    let exemptions = HttpsUpgradeScenario::given_https_first_browsing()
        .after_https_failed_for("https://old-blog.com/post")
        .when_exemptions_are_compiled();

    for url in [
        "http://x:y@example.com/",
        "http://1:2@example.com/",
        "http://192.168.0.1:80@example.com/",
        "http://nas:x@example.com/",
        "http://localhost:3000@example.com/",
        "http://printer.local:x@example.com/",
        "http://abcdefgh.onion:x@example.com/",
        "http://onion.example.com/",
        "http://legacy-portal.com:x@example.com/",
        "http://old-blog.com:x@example.com/",
        "http://old-blog.com.example.com/",
    ] {
        exemptions.then_upgrade(url);
    }
}

#[test]
fn should_remember_failed_hosts_when_configuration_is_refreshed() {
    let mut scenario =
        HttpsUpgradeScenario::given_https_first_browsing().after_https_failed_for("https://old-blog.com/");

    scenario.policy.set_excluded_domains(&[]);

    scenario.when_navigating_to("http://old-blog.com/").then_proceeds_unchanged();
}
