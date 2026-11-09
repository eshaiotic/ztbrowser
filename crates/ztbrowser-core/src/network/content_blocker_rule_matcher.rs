//! Whether one content-blocking rule applies to one request.

use std::collections::HashSet;
use std::sync::OnceLock;

use regex::Regex;
use regex::RegexBuilder;

use crate::protection::content_blocker_rule::ContentBlockerRule;
use crate::protection::content_blocker_rule::LoadType;
use crate::protection::content_blocker_rule::ResourceType;
use crate::protection::content_blocker_rule::RuleAction;
use crate::protection::content_blocker_rule::Trigger;

/// What a rule's trigger is compared with.
pub struct RequestFacts<'a> {
    pub address: &'a str,
    /// Host of the page that makes the request; `None` when the page is not known.
    pub page_host: Option<&'a str>,
    pub kind: ResourceType,
    pub is_third_party: bool,
}

/// Domains as WebKit writes them in `if-domain` and `unless-domain`: `*example.com` is the domain and its
/// subdomains, `example.com` only itself.
struct DomainList {
    exact: HashSet<String>,
    with_subdomains: HashSet<String>,
}

impl DomainList {
    fn new(patterns: &[String]) -> Self {
        let mut list = Self { exact: HashSet::new(), with_subdomains: HashSet::new() };
        for pattern in patterns {
            let pattern = pattern.to_ascii_lowercase();
            match pattern.strip_prefix('*') {
                Some(domain) => list.with_subdomains.insert(domain.trim_start_matches('.').to_owned()),
                None => list.exact.insert(pattern),
            };
        }
        list
    }

    fn contains(&self, host: &str) -> bool {
        let host = host.to_ascii_lowercase();
        if self.exact.contains(&host) {
            return true;
        }
        let mut suffix = host.as_str();
        loop {
            if self.with_subdomains.contains(suffix) {
                return true;
            }
            match suffix.split_once('.') {
                Some((_, parent)) => suffix = parent,
                None => return false,
            }
        }
    }
}

pub struct RuleMatcher {
    trigger: Trigger,
    action: RuleAction,
    if_domain: Option<DomainList>,
    unless_domain: Option<DomainList>,
    /// Compiled on first use: most rules belong to trackers no page ever asks for. `None` when the pattern
    /// cannot be read, which makes the rule never apply (one bad entry must not disable the others).
    pattern: OnceLock<Option<Regex>>,
}

impl RuleMatcher {
    /// URL patterns are small; a pattern that needs more than this to compile is not one of DuckDuckGo's.
    const REGEX_SIZE_LIMIT: usize = 1 << 20;

    #[must_use]
    pub fn new(rule: ContentBlockerRule) -> Self {
        Self {
            if_domain: rule.trigger.if_domain.as_deref().map(DomainList::new),
            unless_domain: rule.trigger.unless_domain.as_deref().map(DomainList::new),
            trigger: rule.trigger,
            action: rule.action,
            pattern: OnceLock::new(),
        }
    }

    #[must_use]
    pub fn action(&self) -> RuleAction {
        self.action
    }

    #[must_use]
    pub fn url_filter(&self) -> &str {
        self.trigger.url_filter.as_str()
    }

    /// The cheap conditions first, the pattern last.
    #[must_use]
    pub fn applies_to(&self, request: &RequestFacts<'_>) -> bool {
        // Rules for the context of a popup concern windows opened by a click; the engine reports no popups.
        if self.trigger.load_context.is_some() {
            return false;
        }
        let load = if request.is_third_party { LoadType::ThirdParty } else { LoadType::FirstParty };
        self.trigger.load_type.as_ref().is_none_or(|kinds| kinds.contains(&load))
            && self.trigger.resource_type.as_ref().is_none_or(|kinds| kinds.contains(&request.kind))
            && self.page_is_listed(request.page_host)
            && self.page_is_not_excluded(request.page_host)
            && self.pattern().is_some_and(|pattern| pattern.is_match(request.address))
    }

    fn page_is_listed(&self, page_host: Option<&str>) -> bool {
        self.if_domain.as_ref().is_none_or(|list| page_host.is_some_and(|host| list.contains(host)))
    }

    fn page_is_not_excluded(&self, page_host: Option<&str>) -> bool {
        self.unless_domain.as_ref().is_none_or(|list| !page_host.is_some_and(|host| list.contains(host)))
    }

    fn pattern(&self) -> Option<&Regex> {
        self.pattern
            .get_or_init(|| {
                RegexBuilder::new(self.trigger.url_filter.as_str())
                    .case_insensitive(true)
                    .size_limit(Self::REGEX_SIZE_LIMIT)
                    .build()
                    .ok()
            })
            .as_ref()
    }
}
