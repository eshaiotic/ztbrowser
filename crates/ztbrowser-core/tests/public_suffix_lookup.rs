//! Finding the site of a host with the compiled-in public suffix list.

use ztbrowser_core::network::public_suffix_lookup::PublicSuffixLookup;
use ztbrowser_core::web_app::registrable_domain_lookup::RegistrableDomainLookup;

struct SiteScenario;

impl SiteScenario {
    fn site_of(host: &str) -> String {
        PublicSuffixLookup.site_of(host)
    }
}

#[test]
fn should_give_subdomains_of_one_company_the_same_site() {
    assert_eq!(SiteScenario::site_of("mail.google.com"), SiteScenario::site_of("accounts.google.com"));
}

#[test]
fn should_keep_one_label_in_front_of_a_multi_label_suffix() {
    assert_eq!(SiteScenario::site_of("www.bbc.co.uk"), "bbc.co.uk");
}

#[test]
fn should_not_join_unrelated_sites_under_a_public_suffix() {
    assert_ne!(SiteScenario::site_of("bbc.co.uk"), SiteScenario::site_of("evil.co.uk"));
}

#[test]
fn should_treat_a_private_hosting_suffix_as_a_suffix() {
    assert_ne!(SiteScenario::site_of("alice.github.io"), SiteScenario::site_of("bob.github.io"));
}

#[test]
fn should_use_the_host_itself_for_localhost_and_addresses() {
    assert_eq!(SiteScenario::site_of("localhost"), "localhost");
    assert_eq!(SiteScenario::site_of("127.0.0.1"), "127.0.0.1");
}
