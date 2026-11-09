//! Finds the rules that can apply to a request without testing them all.
//!
//! The Tracker Data Set yields tens of thousands of rules, and nearly every one starts by naming the tracker's
//! domain. The index files each rule under that domain, so a request to `cdn.ads.example.com` looks up
//! `cdn.ads.example.com`, `ads.example.com` and `example.com` and meets only the rules of those. A rule whose
//! pattern does not start with a plain domain (the exceptions for whole sites) is tried for every request.
//!
//! Rules are kept in the order they were built, because the last matching rule decides: an
//! `ignore-previous-rules` cancels every block before it.

use std::collections::HashMap;

use crate::network::content_blocker_rule_matcher::RequestFacts;
use crate::network::content_blocker_rule_matcher::RuleMatcher;
use crate::protection::content_blocker_rule::ContentBlockerRule;
use crate::protection::content_blocker_rule::RuleAction;
use crate::protection::content_blocker_rule::UrlFilter;

pub struct ContentBlockerRuleIndex {
    rules: Vec<RuleMatcher>,
    by_domain: HashMap<String, Vec<usize>>,
    for_every_request: Vec<usize>,
}

impl ContentBlockerRuleIndex {
    /// What may follow the domain in a pattern for the domain to be all the pattern says about the host.
    const AFTER_DOMAIN: [&'static str; 6] = ["[:/]", "\\/", "/", ":", "(:[0-9]+)?", ""];

    #[must_use]
    pub fn new(rules: Vec<ContentBlockerRule>) -> Self {
        let mut index =
            Self { rules: Vec::with_capacity(rules.len()), by_domain: HashMap::new(), for_every_request: vec![] };
        for (position, rule) in rules.into_iter().enumerate() {
            let matcher = RuleMatcher::new(rule);
            match Self::domain_of(matcher.url_filter()) {
                Some(domain) => index.by_domain.entry(domain).or_default().push(position),
                None => index.for_every_request.push(position),
            }
            index.rules.push(matcher);
        }
        index
    }

    /// Whether the rules, applied in order, end up blocking the request to `host`.
    #[must_use]
    pub fn blocks(&self, host: &str, request: &RequestFacts<'_>) -> bool {
        let mut blocked = false;
        for position in self.candidates(host) {
            let rule = &self.rules[position];
            // A block changes nothing once blocked, an ignore nothing while not: neither needs its pattern read.
            let is_relevant = match rule.action() {
                RuleAction::Block => !blocked,
                RuleAction::IgnorePreviousRules => blocked,
                RuleAction::MakeHttps => false,
            };
            if is_relevant && rule.applies_to(request) {
                blocked = rule.action() == RuleAction::Block;
            }
        }
        blocked
    }

    fn candidates(&self, host: &str) -> Vec<usize> {
        let host = host.to_ascii_lowercase();
        let mut found = self.for_every_request.clone();
        let mut suffix = host.as_str();
        loop {
            if let Some(positions) = self.by_domain.get(suffix) {
                found.extend_from_slice(positions);
            }
            match suffix.split_once('.') {
                Some((_, parent)) => suffix = parent,
                None => break,
            }
        }
        found.sort_unstable();
        found
    }

    /// The domain a pattern starts with, `None` when it starts with anything more complicated.
    fn domain_of(pattern: &str) -> Option<String> {
        let rest = pattern.strip_prefix(UrlFilter::SUBDOMAIN_PREFIX)?;
        let mut domain = String::new();
        let mut characters = rest.char_indices();
        while let Some((position, character)) = characters.next() {
            match character {
                '\\' => match characters.next() {
                    Some((_, '.')) => domain.push('.'),
                    Some((_, '-')) => domain.push('-'),
                    _ => return Self::domain_ending_at(domain, &rest[position..]),
                },
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => domain.push(character.to_ascii_lowercase()),
                _ => return Self::domain_ending_at(domain, &rest[position..]),
            }
        }
        Self::domain_ending_at(domain, "")
    }

    fn domain_ending_at(domain: String, remainder: &str) -> Option<String> {
        let ends_the_host = Self::AFTER_DOMAIN.iter().any(|ending| !ending.is_empty() && remainder.starts_with(ending))
            || remainder.is_empty();
        (ends_the_host && domain.contains('.')).then_some(domain)
    }
}
