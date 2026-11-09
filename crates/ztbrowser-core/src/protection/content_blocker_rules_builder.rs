//! Builds WebKit content-blocking rules from the Tracker Data Set.
//!
//! A faithful port of DuckDuckGo's `TrackerRadarKit.ContentBlockerRulesBuilder` (Apache-2.0,
//! <https://github.com/duckduckgo/TrackerRadarKit>). Keeping the algorithm identical means ZTBrowser inherits
//! years of DuckDuckGo's breakage fixes: the same TDS produces the same blocking decisions as on macOS.
//!
//! Rule list layout (order matters: a later `ignore-previous-rules` entry overrides earlier blocks):
//!
//! 1. per tracker: block the whole domain (if its default is `block`), then its URL-specific rules;
//! 2. the same for CNAME-cloaked trackers, which also apply to first-party requests;
//! 3. exceptions: unprotected sites (user's own list + DuckDuckGo's temporary list) and allowlisted URLs.

use std::collections::HashSet;

use crate::protection::content_blocker_rule::ContentBlockerRule;
use crate::protection::content_blocker_rule::LoadContext;
use crate::protection::content_blocker_rule::LoadType;
use crate::protection::content_blocker_rule::ResourceType;
use crate::protection::content_blocker_rule::RuleAction;
use crate::protection::content_blocker_rule::Trigger;
use crate::protection::content_blocker_rule::UrlFilter;
use crate::protection::privacy_configuration::PrivacyConfiguration;
use crate::protection::tracker_allowlist_entry::TrackerAllowlistEntry;
use crate::protection::tracker_data_set::KnownTracker;
use crate::protection::tracker_data_set::RuleMatching;
use crate::protection::tracker_data_set::TrackerAction;
use crate::protection::tracker_data_set::TrackerDataSet;
use crate::protection::tracker_data_set::TrackerRule;

const THIRD_PARTY_ONLY: &[LoadType] = &[LoadType::ThirdParty];
const ANY_PARTY: &[LoadType] = &[LoadType::FirstParty, LoadType::ThirdParty];

/// Sites and URLs where tracker blocking must not apply.
#[derive(Debug, Clone, Default)]
pub struct RuleExceptions {
    /// Sites the user switched protection off for. Exact domains, no subdomains (as on macOS).
    pub user_unprotected_domains: Vec<String>,
    /// DuckDuckGo's temporary list plus `contentBlocking` exceptions. Subdomains included.
    pub temporary_unprotected_domains: Vec<String>,
    pub tracker_allowlist: Vec<TrackerAllowlistEntry>,
}

impl RuleExceptions {
    /// Privacy configuration feature that switches tracker blocking on and lists its site exceptions.
    pub const CONTENT_BLOCKING: &'static str = "contentBlocking";

    /// The exceptions DuckDuckGo publishes in the privacy configuration, plus the user's own list.
    #[must_use]
    pub fn from_configuration(configuration: &PrivacyConfiguration, user_unprotected_domains: Vec<String>) -> Self {
        let mut temporary_unprotected_domains = configuration.temporary_unprotected_domains();
        temporary_unprotected_domains.extend(configuration.exception_domains(Self::CONTENT_BLOCKING));
        Self {
            user_unprotected_domains,
            temporary_unprotected_domains,
            tracker_allowlist: configuration.tracker_allowlist(),
        }
    }

    /// `ignore-previous-rules` entries that must come after every blocking rule.
    fn rules(&self) -> Vec<ContentBlockerRule> {
        let unprotected_sites: Vec<String> = self
            .user_unprotected_domains
            .iter()
            .cloned()
            .chain(self.temporary_unprotected_domains.iter().map(|domain| format!("*{domain}")))
            .collect();
        let mut rules = vec![];
        if !unprotected_sites.is_empty() {
            let trigger = Trigger {
                if_domain: Some(unprotected_sites),
                load_type: Some(ANY_PARTY.to_vec()),
                ..Trigger::new(UrlFilter::any_url())
            };
            rules.push(ContentBlockerRule::new(trigger, RuleAction::IgnorePreviousRules));
        }
        for entry in &self.tracker_allowlist {
            let trigger = Trigger {
                if_domain: ContentBlockerRulesBuilder::wildcards(entry.domains.as_deref()),
                load_type: Some(ANY_PARTY.to_vec()),
                ..Trigger::new(UrlFilter::matching_allowlisted_url(&entry.rule))
            };
            rules.push(ContentBlockerRule::new(trigger, RuleAction::IgnorePreviousRules));
        }
        rules
    }
}

pub struct ContentBlockerRulesBuilder<'data> {
    data_set: &'data TrackerDataSet,
}

impl<'data> ContentBlockerRulesBuilder<'data> {
    #[must_use]
    pub fn new(data_set: &'data TrackerDataSet) -> Self {
        Self { data_set }
    }

    #[must_use]
    pub fn build_rules(&self, exceptions: &RuleExceptions) -> Vec<ContentBlockerRule> {
        let mut rules: Vec<ContentBlockerRule> =
            self.data_set.trackers().flat_map(|tracker| self.rules_for_tracker(tracker, THIRD_PARTY_ONLY)).collect();
        for cloaked in self.cname_cloaked_trackers() {
            rules.extend(self.rules_for_tracker(&cloaked, ANY_PARTY));
        }
        rules.extend(exceptions.rules());
        rules
    }

    fn cname_cloaked_trackers(&self) -> Vec<KnownTracker> {
        self.data_set
            .cnames()
            .filter_map(|(alias, target)| {
                self.data_set.find_tracker_by_cname(target).map(|tracker| tracker.relocated_to(alias))
            })
            .collect()
    }

    fn rules_for_tracker(&self, tracker: &KnownTracker, load_types: &[LoadType]) -> Vec<ContentBlockerRule> {
        let mut groups: Vec<Vec<ContentBlockerRule>> =
            tracker.rules.iter().map(|rule| self.rules_for_tracker_rule(rule, tracker, load_types)).collect();
        // Stable sort, larger groups first, exactly like upstream.
        groups.sort_by_key(|group| std::cmp::Reverse(group.len()));
        let mut seen = HashSet::new();
        let specific_rules = groups.into_iter().flatten().filter(|rule| seen.insert(rule.clone()));
        self.whole_domain_rules(tracker, load_types).into_iter().chain(specific_rules).collect()
    }

    fn whole_domain_rules(&self, tracker: &KnownTracker, load_types: &[LoadType]) -> Vec<ContentBlockerRule> {
        let Some(domain) = tracker.domain.as_deref() else { return vec![] };
        if tracker.default_action != Some(TrackerAction::Block) {
            return vec![];
        }
        let url_filter = UrlFilter::matching_domain(domain);
        let block = Trigger {
            unless_domain: self.owner_sites(tracker),
            load_type: Some(load_types.to_vec()),
            ..Trigger::new(url_filter.clone())
        };
        vec![ContentBlockerRule::new(block, RuleAction::Block), Self::allow_as_top_level_popup(url_filter, load_types)]
    }

    fn rules_for_tracker_rule(
        &self,
        rule: &TrackerRule,
        tracker: &KnownTracker,
        load_types: &[LoadType],
    ) -> Vec<ContentBlockerRule> {
        if tracker.default_action == Some(TrackerAction::Block) {
            self.rules_for_blocked_tracker(rule, tracker, load_types)
        } else {
            self.rules_for_ignored_tracker(rule, tracker, load_types)
        }
    }

    fn rules_for_ignored_tracker(
        &self,
        rule: &TrackerRule,
        tracker: &KnownTracker,
        load_types: &[LoadType],
    ) -> Vec<ContentBlockerRule> {
        let block_everywhere = || self.block(rule, tracker, load_types, None);
        if rule.action == Some(TrackerAction::Ignore) {
            return vec![block_everywhere(), Self::ignore_previous(rule, load_types, rule.options.as_ref())];
        }
        match (&rule.options, &rule.exceptions) {
            (None, None) => vec![
                block_everywhere(),
                Self::allow_as_top_level_popup(UrlFilter::matching_tds_pattern(&rule.pattern), load_types),
            ],
            (Some(options), Some(exceptions)) => vec![
                self.block(rule, tracker, load_types, Some(options)),
                Self::ignore_previous(rule, load_types, Some(exceptions)),
            ],
            (Some(options), None) => vec![self.block(rule, tracker, load_types, Some(options))],
            (None, Some(exceptions)) => {
                vec![block_everywhere(), Self::ignore_previous(rule, load_types, Some(exceptions))]
            }
        }
    }

    fn rules_for_blocked_tracker(
        &self,
        rule: &TrackerRule,
        tracker: &KnownTracker,
        load_types: &[LoadType],
    ) -> Vec<ContentBlockerRule> {
        if let (Some(options), Some(exceptions)) = (&rule.options, &rule.exceptions) {
            return vec![
                Self::ignore_previous(rule, load_types, None),
                self.block(rule, tracker, load_types, Some(options)),
                Self::ignore_previous(rule, load_types, Some(exceptions)),
            ];
        }
        if rule.action == Some(TrackerAction::Ignore) {
            return vec![Self::ignore_previous(rule, load_types, rule.options.as_ref())];
        }
        match (&rule.options, &rule.exceptions) {
            (Some(options), _) => {
                vec![
                    Self::ignore_previous(rule, load_types, None),
                    self.block(rule, tracker, load_types, Some(options)),
                ]
            }
            (None, Some(exceptions)) => vec![Self::ignore_previous(rule, load_types, Some(exceptions))],
            (None, None) => vec![self.block(rule, tracker, load_types, None)],
        }
    }

    fn block(
        &self,
        rule: &TrackerRule,
        tracker: &KnownTracker,
        load_types: &[LoadType],
        matching: Option<&RuleMatching>,
    ) -> ContentBlockerRule {
        let url_filter = UrlFilter::matching_tds_pattern(&rule.pattern);
        let trigger = match matching {
            None => Trigger {
                unless_domain: self.owner_sites(tracker),
                load_type: Some(load_types.to_vec()),
                ..Trigger::new(url_filter)
            },
            // Same as upstream: scoped blocks are always third-party only, whatever `load_types` says.
            Some(matching) => Trigger {
                if_domain: Self::wildcards(matching.domains.as_deref()),
                resource_type: Self::webkit_resource_types(matching.types.as_deref()),
                load_type: Some(THIRD_PARTY_ONLY.to_vec()),
                ..Trigger::new(url_filter)
            },
        };
        ContentBlockerRule::new(trigger, RuleAction::Block)
    }

    fn owner_sites(&self, tracker: &KnownTracker) -> Option<Vec<String>> {
        Self::wildcards(self.data_set.related_domains(tracker.owner_name.as_deref()))
    }

    fn allow_as_top_level_popup(url_filter: UrlFilter, load_types: &[LoadType]) -> ContentBlockerRule {
        let trigger = Trigger {
            resource_type: Some(vec![ResourceType::Popup]),
            load_type: Some(load_types.to_vec()),
            load_context: Some(vec![LoadContext::TopFrame]),
            ..Trigger::new(url_filter)
        };
        ContentBlockerRule::new(trigger, RuleAction::IgnorePreviousRules)
    }

    fn ignore_previous(
        rule: &TrackerRule,
        load_types: &[LoadType],
        matching: Option<&RuleMatching>,
    ) -> ContentBlockerRule {
        let trigger = Trigger {
            if_domain: matching.and_then(|matching| Self::wildcards(matching.domains.as_deref())),
            resource_type: matching.and_then(|matching| Self::webkit_resource_types(matching.types.as_deref())),
            load_type: Some(load_types.to_vec()),
            ..Trigger::new(UrlFilter::matching_tds_pattern(&rule.pattern))
        };
        ContentBlockerRule::new(trigger, RuleAction::IgnorePreviousRules)
    }

    /// WebKit's `*domain` form: the domain and all of its subdomains.
    fn wildcards(domains: Option<&[String]>) -> Option<Vec<String>> {
        domains.map(|domains| domains.iter().map(|domain| format!("*{domain}")).collect())
    }

    fn webkit_resource_types(types: Option<&[String]>) -> Option<Vec<ResourceType>> {
        Some(types?.iter().filter_map(|name| ResourceType::from_tds_name(name)).collect())
    }
}
