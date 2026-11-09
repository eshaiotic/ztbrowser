//! The WebKit rule lists ZTBrowser Guard builds from community filter lists.
//!
//! Each Guard rule list is compiled as a WebKit list of its own, for two WebKit limits: a list holds at most
//! 150 000 rules, and an exception (`ignore-previous-rules`) only cancels rules of its own list. That is why
//! uBlock Origin's "Unbreak" exceptions are read into every list, and why every list ends with the sites the
//! user switched protection off for.
//!
//! Brave's `adblock` crate (MPL-2.0) converts Adblock Plus syntax into WebKit's JSON, as Brave does for its
//! iOS browser. Filters without `$third-party` apply to a site's own requests too, which is what Brave Shields
//! does only in their "aggressive" mode.

use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use adblock::content_blocking::CbAction;
use adblock::content_blocking::CbRule;
use adblock::content_blocking::CbTrigger;
use adblock::content_blocking::CbType;
use adblock::content_blocking::ignore_previous_fp_documents;
use adblock::filters::network::FilterPart;
use adblock::filters::network::NetworkFilter;
use adblock::filters::network::NetworkFilterMaskHelper;
use adblock::lists::FilterSet;
use adblock::lists::ParseOptions;

use crate::guard::filter_list_source::FilterListSource;
use crate::guard::generic_hiding::GenericHiding;
use crate::protection::content_blocker_rule::ContentBlockerRule;
use crate::protection::content_blocker_rule::ResourceType;
use crate::protection::content_blocker_rule::RuleAction;
use crate::protection::content_blocker_rule::Trigger;
use crate::protection::content_blocker_rule::UrlFilter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuardRuleList {
    name: &'static str,
    sources: &'static [FilterListSource],
    blocks_link_pings: bool,
}

impl GuardRuleList {
    /// Ads, malware and scam sites, and the Polish sites' own ad servers.
    pub const ADS: Self = Self {
        name: "guard-ads",
        sources: &[
            FilterListSource::EASYLIST,
            FilterListSource::UBLOCK_FILTERS,
            FilterListSource::UBLOCK_BADWARE,
            FilterListSource::PETER_LOWE,
            FilterListSource::POLISH_FILTERS,
            FilterListSource::UBLOCK_UNBREAK,
        ],
        blocks_link_pings: false,
    };
    /// Trackers, analytics and link-click pings.
    pub const PRIVACY: Self = Self {
        name: "guard-privacy",
        sources: &[FilterListSource::EASYPRIVACY, FilterListSource::UBLOCK_PRIVACY, FilterListSource::UBLOCK_UNBREAK],
        blocks_link_pings: true,
    };
    /// Cookie consent notices.
    pub const ANNOYANCES: Self = Self {
        name: "guard-annoyances",
        sources: &[FilterListSource::COOKIE_NOTICES, FilterListSource::UBLOCK_UNBREAK],
        blocks_link_pings: false,
    };
    pub const ALL: [Self; 3] = [Self::ADS, Self::PRIVACY, Self::ANNOYANCES];
    /// WebKit rejects a whole list above this many rules (`maxRuleCount` in its `ContentExtensionParser`).
    pub const MAX_RULES: usize = 150_000;
    /// The element Guard's page script hides to learn whether WebKit applies a list's element hiding.
    pub const PROBE_ELEMENT: &'static str = "ztbrowser-guard-probe";
    /// Bump when the conversion changes (an `adblock` upgrade included), so lists compiled by an older
    /// ZTBrowser are rebuilt.
    const CONVERSION_REVISION: u32 = 2;

    /// The name the compiled list is stored under.
    #[must_use]
    pub fn name(self) -> &'static str {
        self.name
    }

    #[must_use]
    pub fn sources(self) -> &'static [FilterListSource] {
        self.sources
    }

    /// Whether the list would hold any rule; WebKit rejects an empty list.
    #[must_use]
    pub fn has_rules(self, downloaded: &HashMap<FilterListSource, String>) -> bool {
        self.blocks_link_pings || self.sources.iter().any(|source| downloaded.contains_key(source))
    }

    /// Identifies what the list is built from: its filter lists, the user's unprotected sites and the
    /// conversion itself. An unchanged fingerprint means the list compiled last time is still right, so the
    /// conversion (a few tenths of a second and tens of megabytes) is skipped.
    #[must_use]
    pub fn fingerprint(
        self,
        downloaded: &HashMap<FilterListSource, String>,
        user_unprotected_domains: &[String],
    ) -> String {
        let mut hasher = DefaultHasher::new();
        (self.name, Self::CONVERSION_REVISION, user_unprotected_domains).hash(&mut hasher);
        for source in self.sources {
            (source.id(), downloaded.get(source)).hash(&mut hasher);
        }
        format!("{:016x}", hasher.finish())
    }

    /// What WebKit hides when it applies the list's element hiding on a page: an element of
    /// [`Self::PROBE_ELEMENT`] whose class is the list's name. Guard's page script hides the list's generic
    /// selectors only where WebKit hides this probe, so every exception in the list (an `$elemhide` filter, the
    /// user's shield) switches generic hiding off exactly where it switches WebKit's hiding off.
    #[must_use]
    pub fn probe_selector(self) -> String {
        format!("{}.{}", Self::PROBE_ELEMENT, self.name)
    }

    /// The list as WebKit reads it (a JSON array) and its generic element hiding, left to the page script:
    /// link-click pings blocked (privacy list), the downloaded filters converted, then one rule standing
    /// everything down on the sites the user switched protection off for. Filter lists not downloaded yet are
    /// left out.
    #[must_use]
    pub fn converted(
        self,
        downloaded: &HashMap<FilterListSource, String>,
        user_unprotected_domains: &[String],
    ) -> ConvertedGuardList {
        let mut leading = Vec::new();
        if self.blocks_link_pings {
            let pings = Trigger { resource_type: Some(vec![ResourceType::Ping]), ..Trigger::new(UrlFilter::any_url()) };
            leading.push(ContentBlockerRule::new(pings, RuleAction::Block).to_json().to_string());
        }
        let mut closing = Vec::new();
        if !user_unprotected_domains.is_empty() {
            let unprotected =
                Trigger { if_domain: Some(user_unprotected_domains.to_vec()), ..Trigger::new(UrlFilter::any_url()) };
            closing.push(ContentBlockerRule::new(unprotected, RuleAction::IgnorePreviousRules).to_json().to_string());
        }
        let (converted, generic_hiding) =
            self.webkit_rules(downloaded, Self::MAX_RULES - leading.len() - closing.len());
        let content_rules =
            format!("[{}]", leading.into_iter().chain(converted).chain(closing).collect::<Vec<String>>().join(","));
        ConvertedGuardList { content_rules, generic_hiding }
    }

    /// The filters as WebKit rules, at most `room` of them, each serialised on its own (a whole list as
    /// `serde_json::Value`s would take ten times the memory), with the generic hiding taken out.
    fn webkit_rules(self, downloaded: &HashMap<FilterListSource, String>, room: usize) -> (Vec<String>, GenericHiding) {
        let mut filters = FilterSet::new(true);
        let mut generic_hiding = GenericHiding::default();
        for text in self.sources.iter().filter_map(|source| downloaded.get(source)) {
            filters.add_filter_list(text.clone(), ParseOptions::default());
            for site in Self::generic_hiding_exceptions(text) {
                generic_hiding.switch_off_on(&site);
            }
        }
        let Ok((converted, _)) = filters.into_content_blocking() else { return (Vec::new(), generic_hiding) };
        let mut rules = Vec::with_capacity(converted.len());
        for rule in converted {
            match Self::generic_selector(&rule) {
                Some((selector, unless_domains)) => generic_hiding.add(selector, unless_domains),
                None => rules.push(rule),
            }
        }
        self.place_page_exception_and_probe(&mut rules, !generic_hiding.is_empty());
        Self::fit(&mut rules, room);
        (rules.iter().filter_map(|rule| serde_json::to_string(rule).ok()).collect(), generic_hiding)
    }

    /// The sites of `@@||bank.pl^$generichide` exceptions, which `adblock` cannot convert for WebKit: generic
    /// hiding breaks them. Exceptions narrowed further (a path, `$domain=`) are left out.
    fn generic_hiding_exceptions(text: &str) -> impl Iterator<Item = String> {
        text.lines()
            .filter(|line| line.starts_with("@@") && (line.contains("ghide") || line.contains("generichide")))
            .filter_map(|line| NetworkFilter::parse(line, false, ParseOptions::default()).ok())
            .filter(|filter| {
                filter.is_exception()
                    && filter.is_generic_hide()
                    && filter.is_hostname_anchor()
                    && matches!(filter.filter, FilterPart::Empty)
                    && filter.opt_domains.is_none()
            })
            .filter_map(|filter| filter.hostname.map(Cow::into_owned))
            .filter(|site| !site.contains('*'))
    }

    /// The selector of an element-hiding rule for every site, or every site but some (`~bank.pl##.ad`).
    fn generic_selector(rule: &CbRule) -> Option<(&str, &[String])> {
        let trigger = &rule.trigger;
        let is_generic = rule.action.typ == CbType::CssDisplayNone
            && trigger.url_filter == UrlFilter::any_url().as_str()
            && trigger.if_domain.is_none()
            && trigger.resource_type.is_none()
            && trigger.load_type.is_empty()
            && trigger.if_top_url.is_none()
            && trigger.unless_top_url.is_none();
        let selector = rule.action.selector.as_deref().filter(|_| is_generic)?;
        Some((selector, trigger.unless_domain.as_deref().unwrap_or_default()))
    }

    /// `adblock` closes its output with an exception for first-party documents, so that no filter blocks the
    /// page the user asked for. WebKit picks a page's hidden elements on that same load, though, so at the end
    /// the exception would cancel all element hiding: it moves ahead of the hiding rules and exceptions,
    /// followed by the probe (see [`Self::probe_selector`]), which those exceptions can then cancel.
    fn place_page_exception_and_probe(self, rules: &mut Vec<CbRule>, with_probe: bool) {
        let mut placed = Vec::new();
        let page_exception = ignore_previous_fp_documents();
        if rules.last() == Some(&page_exception) {
            rules.pop();
            placed.push(page_exception);
        }
        if with_probe {
            placed.push(CbRule {
                trigger: CbTrigger { url_filter: UrlFilter::any_url().as_str().to_owned(), ..CbTrigger::default() },
                action: CbAction { typ: CbType::CssDisplayNone, selector: Some(self.probe_selector()) },
            });
        }
        let first_hiding_or_exception = rules
            .iter()
            .position(|rule| matches!(rule.action.typ, CbType::CssDisplayNone | CbType::IgnorePreviousRules))
            .unwrap_or(rules.len());
        rules.splice(first_hiding_or_exception..first_hiding_or_exception, placed);
    }

    /// Drops the blocking rules just before the first exception until the rules fit: losing a few filters
    /// beats WebKit rejecting the whole list, and every exception is kept so that nothing they repair breaks
    /// again.
    fn fit(rules: &mut Vec<CbRule>, room: usize) {
        let excess = rules.len().saturating_sub(room);
        if excess > 0 {
            let first_exception =
                rules.iter().position(|rule| rule.action.typ == CbType::IgnorePreviousRules).unwrap_or(rules.len());
            rules.drain(first_exception.saturating_sub(excess)..first_exception);
        }
    }
}

/// A Guard list converted: the WebKit rules, and the generic element hiding Guard's page script applies.
pub struct ConvertedGuardList {
    content_rules: String,
    generic_hiding: GenericHiding,
}

impl ConvertedGuardList {
    #[must_use]
    pub fn content_rules(&self) -> &str {
        &self.content_rules
    }

    #[must_use]
    pub fn into_generic_hiding(self) -> GenericHiding {
        self.generic_hiding
    }

    #[must_use]
    pub fn into_parts(self) -> (String, GenericHiding) {
        (self.content_rules, self.generic_hiding)
    }
}
