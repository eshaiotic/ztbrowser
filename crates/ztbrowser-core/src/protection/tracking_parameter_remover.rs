//! Strips tracking parameters (`fbclid`, `gclid`, `utm_*`, ...) from a link before the browser opens it.
//!
//! Port of DuckDuckGo's `LinkCleaner.cleanTrackingParameters` and its byte-preserving filter: the query is
//! split on every `&`, a parameter's name ends at the first `=`, names match exactly (case-sensitive), and
//! everything else in the URL is kept byte for byte so signed or encoded links survive.
//!
//! In the browser, [`TrackingParameterRemover::content_rules`] stops a top-frame navigation that still
//! carries a tracking parameter before its request is sent; the browser then opens [`cleaned`] instead.
//! WebKitGTK cannot tell main-frame from iframe navigations in its navigation hook, but its content rules
//! can (`load-context: top-frame`), and iframes keep their parameters as on macOS. One difference from
//! macOS: rules see only the destination, so a link from an unprotected site to a protected one is cleaned.
//!
//! [`cleaned`]: TrackingParameterRemover::cleaned

use std::collections::BTreeSet;
use std::collections::HashSet;

use url::Url;

use crate::protection::content_blocker_rule::ContentBlockerRule;
use crate::protection::content_blocker_rule::LoadContext;
use crate::protection::content_blocker_rule::ResourceType;
use crate::protection::content_blocker_rule::RuleAction;
use crate::protection::content_blocker_rule::Trigger;
use crate::protection::content_blocker_rule::UrlFilter;
use crate::protection::domain_name::DomainName;
use crate::protection::privacy_configuration::PrivacyConfiguration;

const FEATURE: &str = "trackingParameters";

pub struct TrackingParameterRemover<'config> {
    configuration: &'config PrivacyConfiguration,
    user_unprotected_domains: &'config [String],
    parameters: HashSet<String>,
}

impl<'config> TrackingParameterRemover<'config> {
    #[must_use]
    pub fn new(configuration: &'config PrivacyConfiguration, user_unprotected_domains: &'config [String]) -> Self {
        Self { configuration, user_unprotected_domains, parameters: configuration.tracking_parameters() }
    }

    /// `url` without tracking parameters. `initiator` is the page the link was clicked on, if any.
    #[must_use]
    pub fn cleaned(&self, url: &str, initiator: Option<&str>) -> String {
        if self.parameters.is_empty() || self.is_excluded(url) || initiator.is_some_and(|page| self.is_excluded(page)) {
            return url.to_owned();
        }
        self.without_tracking_parameters(url).unwrap_or_else(|| url.to_owned())
    }

    /// WebKit rules that block top-frame navigations carrying a tracking parameter, except on sites where
    /// cleaning is switched off. Parameter names that are not plain words are skipped: one bad regex would
    /// make WebKit reject the whole rule list.
    #[must_use]
    pub fn content_rules(&self) -> Vec<ContentBlockerRule> {
        if !self.configuration.is_enabled(FEATURE) {
            return vec![];
        }
        let unprotected = self.unprotected_destinations();
        let parameters: BTreeSet<&str> =
            self.parameters.iter().map(String::as_str).filter(|name| Self::is_plain_word(name)).collect();
        parameters
            .into_iter()
            .map(|parameter| {
                let trigger = Trigger {
                    unless_domain: (!unprotected.is_empty()).then(|| unprotected.clone()),
                    resource_type: Some(vec![ResourceType::Document]),
                    load_context: Some(vec![LoadContext::TopFrame]),
                    ..Trigger::new(UrlFilter::carrying_query_parameter(parameter))
                };
                ContentBlockerRule::new(trigger, RuleAction::Block)
            })
            .collect()
    }

    /// WebKit `unless-domain` entries: `*site` covers subdomains (as the configuration lists do); the user's
    /// own list is exact, as [`PrivacyConfiguration::is_enabled_for_site`] treats it.
    fn unprotected_destinations(&self) -> Vec<String> {
        let mut site_lists = self.configuration.temporary_unprotected_domains();
        site_lists.extend(self.configuration.exception_domains(FEATURE));
        let with_subdomains = site_lists.iter().map(|domain| format!("*{}", DomainName::new(domain)));
        let exact = self.user_unprotected_domains.iter().map(|domain| DomainName::new(domain).to_string());
        with_subdomains.chain(exact).collect::<BTreeSet<_>>().into_iter().collect()
    }

    fn is_plain_word(name: &str) -> bool {
        !name.is_empty() && name.chars().all(|character| character.is_ascii_alphanumeric() || "_-.".contains(character))
    }

    /// Non-web URLs count as excluded, exactly as upstream.
    fn is_excluded(&self, url: &str) -> bool {
        let Ok(parsed) = Url::parse(url) else { return true };
        let Some(host) = parsed.host_str().filter(|_| matches!(parsed.scheme(), "http" | "https")) else { return true };
        !self.configuration.is_enabled_for_site(FEATURE, host, self.user_unprotected_domains)
    }

    fn without_tracking_parameters(&self, url: &str) -> Option<String> {
        let (before_fragment, fragment) = match url.split_once('#') {
            Some((head, fragment)) => (head, Some(fragment)),
            None => (url, None),
        };
        let (path, query) = before_fragment.split_once('?')?;
        let items: Vec<&str> = query.split('&').collect();
        let kept: Vec<&str> = items.iter().copied().filter(|item| !self.is_tracking_item(item)).collect();
        if kept.len() == items.len() {
            return None;
        }
        let mut cleaned = path.to_owned();
        if !kept.is_empty() {
            cleaned.push('?');
            cleaned.push_str(&kept.join("&"));
        }
        if let Some(fragment) = fragment {
            cleaned.push('#');
            cleaned.push_str(fragment);
        }
        Some(cleaned)
    }

    fn is_tracking_item(&self, item: &str) -> bool {
        let name = item.split_once('=').map_or(item, |(name, _)| name);
        self.parameters.contains(name)
    }
}
