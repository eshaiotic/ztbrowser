//! The community filter lists ZTBrowser Guard downloads at run time, and how a download is recognised as one.

use ztbrowser_core::guard::filter_list_source::FilterListSource;
use ztbrowser_core::guard::guard_rule_list::GuardRuleList;

struct DownloadScenario {
    body: &'static str,
}

impl DownloadScenario {
    fn given_downloaded(body: &'static str) -> Self {
        Self { body }
    }

    fn then_is_accepted(&self) {
        assert!(FilterListSource::accepts(self.body), "{:?} rejected", self.body);
    }

    fn then_is_rejected(&self) {
        assert!(!FilterListSource::accepts(self.body), "{:?} accepted", self.body);
    }
}

#[test]
fn should_accept_an_adblock_plus_list() {
    DownloadScenario::given_downloaded("[Adblock Plus 2.0]\n! Title: EasyList\n||ads.example^\n").then_is_accepted();
}

#[test]
fn should_accept_a_list_that_starts_with_a_comment() {
    DownloadScenario::given_downloaded("\n! Title: uBlock filters\n||ads.example^\n").then_is_accepted();
}

#[test]
fn should_reject_a_web_page_served_in_place_of_the_list() {
    DownloadScenario::given_downloaded("<!DOCTYPE html><title>Hotel Wi-Fi login</title>").then_is_rejected();
}

#[test]
fn should_reject_an_empty_download() {
    DownloadScenario::given_downloaded(" \n").then_is_rejected();
}

#[test]
fn should_download_every_list_a_guard_rule_list_reads() {
    let missing: Vec<&str> = GuardRuleList::ALL
        .iter()
        .flat_map(|list| list.sources())
        .filter(|source| !FilterListSource::ALL.contains(source))
        .map(|source| source.name())
        .collect();

    assert!(missing.is_empty(), "never downloaded: {missing:?}");
}

#[test]
fn should_download_lists_over_https_only() {
    let insecure: Vec<&str> =
        FilterListSource::ALL.iter().map(|source| source.url()).filter(|url| !url.starts_with("https://")).collect();

    assert!(insecure.is_empty(), "{insecure:?}");
}

#[test]
fn should_name_every_list_uniquely() {
    let mut ids: Vec<&str> = FilterListSource::ALL.iter().map(|source| source.id()).collect();
    ids.sort_unstable();
    ids.dedup();

    assert_eq!(ids.len(), FilterListSource::ALL.len());
}
