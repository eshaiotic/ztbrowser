//! Upgrades plain-HTTP requests to HTTPS ("HTTPS-first").
//!
//! DuckDuckGo decides with its Smarter Encryption list; ZTBrowser simply tries HTTPS for every public host and
//! falls back to HTTP for the rest of the session once a host fails over HTTPS. Hosts in the privacy
//! configuration's `https` exceptions, local-network hosts (which almost never have valid certificates) and
//! onion services (encrypted end to end by Tor; most offer only plain HTTP) are left alone.
//!
//! The upgrade itself is a WebKit `make-https` content rule, applied before any byte leaves the machine and
//! in every frame. A navigation hook cannot do that: WebKitGTK does not tell whether a navigation targets the
//! main frame or an iframe, so rewriting it there would turn iframe loads into whole-page navigations.

use std::collections::BTreeSet;

use url::Host;
use url::Url;

use crate::protection::content_blocker_rule::ContentBlockerRule;
use crate::protection::content_blocker_rule::RuleAction;
use crate::protection::content_blocker_rule::Trigger;
use crate::protection::content_blocker_rule::UrlFilter;
use crate::protection::domain_name::DomainName;
use crate::protection::site_list::SiteList;

#[derive(Debug, Clone, Default)]
pub struct HttpsUpgradePolicy {
    excluded: SiteList,
    failed_hosts: BTreeSet<DomainName>,
}

impl HttpsUpgradePolicy {
    #[must_use]
    pub fn new(excluded_domains: &[String]) -> Self {
        Self { excluded: SiteList::new(excluded_domains.iter().map(String::as_str)), failed_hosts: BTreeSet::new() }
    }

    /// Replaces the configuration's exception list; hosts that already failed over HTTPS stay remembered.
    pub fn set_excluded_domains(&mut self, excluded_domains: &[String]) {
        self.excluded = SiteList::new(excluded_domains.iter().map(String::as_str));
    }

    /// The HTTPS version of `url`, or `None` when the navigation should go ahead unchanged.
    #[must_use]
    pub fn upgraded(&self, url: &str) -> Option<String> {
        let mut parsed = Url::parse(url).ok().filter(|parsed| parsed.scheme() == "http")?;
        let Some(Host::Domain(host)) = parsed.host() else { return None };
        let host = DomainName::new(host);
        if host.is_local_network()
            || host.is_onion()
            || self.excluded.covers(host.as_str())
            || self.failed_hosts.contains(&host)
        {
            return None;
        }
        parsed.set_scheme("https").ok()?;
        Some(parsed.into())
    }

    /// WebKit rules: upgrade every plain-HTTP request, then exempt exactly the hosts [`Self::upgraded`] leaves
    /// alone. The exemptions are `ignore-previous-rules`, so these rules need a rule list of their own.
    #[must_use]
    pub fn content_rules(&self) -> Vec<ContentBlockerRule> {
        let address_hosts = [UrlFilter::plain_http_ipv4_host(), UrlFilter::plain_http_ipv6_host()];
        let special_domains = DomainName::LOCAL_NETWORK_DOMAINS
            .into_iter()
            .chain([DomainName::ONION_DOMAIN])
            .map(UrlFilter::plain_http_on_domain);
        let excluded = self.excluded.domains().map(UrlFilter::plain_http_on_domain);
        let failed = self.failed_hosts.iter().map(|host| UrlFilter::plain_http_on_exact_host(host.as_str()));
        let exemptions = address_hosts
            .into_iter()
            .chain([UrlFilter::plain_http_single_label_host()])
            .chain(special_domains)
            .chain(excluded)
            .chain(failed)
            .map(|filter| ContentBlockerRule::new(Trigger::new(filter), RuleAction::IgnorePreviousRules));
        let upgrade = ContentBlockerRule::new(Trigger::new(UrlFilter::plain_http()), RuleAction::MakeHttps);
        std::iter::once(upgrade).chain(exemptions).collect()
    }

    /// Remembers that `url`'s host could not be loaded over HTTPS, so it is not upgraded again.
    pub fn record_failed_upgrade(&mut self, url: &str) {
        if let Some(host) = Url::parse(url).ok().as_ref().and_then(Url::host_str) {
            self.failed_hosts.insert(DomainName::new(host));
        }
    }
}
