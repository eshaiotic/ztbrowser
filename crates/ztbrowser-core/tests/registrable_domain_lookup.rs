//! Finding the registrable domain of a host from a public suffix list that only answers "is this a suffix?".

use ztbrowser_core::web_app::registrable_domain_lookup::RegistrableDomainLookup;

/// Test double for the public suffix list, with a multi-label and a private (hosting) suffix.
struct KnownSuffixes;

impl RegistrableDomainLookup for KnownSuffixes {
    fn is_public_suffix(&self, domain: &str) -> bool {
        matches!(domain, "com" | "uk" | "co.uk" | "s3.amazonaws.com")
    }
}

struct HostScenario {
    host: &'static str,
}

impl HostScenario {
    fn given_host(host: &'static str) -> Self {
        Self { host }
    }

    fn when_looked_up(&self) -> Option<String> {
        KnownSuffixes.registrable_domain(self.host)
    }
}

#[test]
fn should_strip_subdomains_down_to_the_registrable_domain() {
    assert_eq!(HostScenario::given_host("mail.google.com").when_looked_up().as_deref(), Some("google.com"));
}

#[test]
fn should_keep_one_label_in_front_of_a_multi_label_suffix() {
    assert_eq!(HostScenario::given_host("www.bbc.co.uk").when_looked_up().as_deref(), Some("bbc.co.uk"));
}

#[test]
fn should_use_the_longest_matching_suffix() {
    assert_eq!(
        HostScenario::given_host("www.bucket.s3.amazonaws.com").when_looked_up().as_deref(),
        Some("bucket.s3.amazonaws.com")
    );
}

#[test]
fn should_treat_an_unlisted_top_level_domain_as_a_suffix() {
    assert_eq!(HostScenario::given_host("wiki.intranet.corp").when_looked_up().as_deref(), Some("intranet.corp"));
}

#[test]
fn should_ignore_the_trailing_dot_of_a_fully_qualified_host() {
    assert_eq!(HostScenario::given_host("mail.google.com.").when_looked_up().as_deref(), Some("google.com"));
}

#[test]
fn should_find_nothing_for_a_public_suffix_itself() {
    assert_eq!(HostScenario::given_host("co.uk").when_looked_up(), None);
}

#[test]
fn should_find_nothing_for_a_single_label_host() {
    assert_eq!(HostScenario::given_host("localhost").when_looked_up(), None);
}

#[test]
fn should_find_nothing_for_an_ipv4_address() {
    assert_eq!(HostScenario::given_host("192.168.1.10").when_looked_up(), None);
}

#[test]
fn should_find_nothing_for_an_ipv6_address() {
    assert_eq!(HostScenario::given_host("[::1]").when_looked_up(), None);
}
