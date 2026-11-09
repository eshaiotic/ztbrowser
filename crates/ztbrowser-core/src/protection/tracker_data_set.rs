//! DuckDuckGo's Tracker Data Set (TDS): which third-party domains track people and who owns them.
//!
//! The TDS is a JSON document published by DuckDuckGo (`macos-tds.json`). Each *tracker* is a domain with
//! a default action (block it everywhere, or ignore it unless a specific rule matches) and optional per-URL
//! *rules*. Each *entity* is a company owning several domains; a company's own domains are never blocked on
//! its own sites (loading `fbcdn.net` on `facebook.com` is first-party in spirit). *CNAMEs* map "cloaked"
//! first-party subdomains that secretly point at a tracker.
//!
//! Parsing is deliberately lenient, mirroring DuckDuckGo's Swift decoder: a malformed rule is skipped rather
//! than invalidating the whole data set, so one bad upstream entry cannot switch tracker blocking off.

use std::collections::BTreeMap;
use std::collections::HashMap;

use serde_json::Value;

use crate::document::lenient_json::LenientJson;
use crate::protection::domain_name::DomainName;

/// What to do with requests to a tracker when no more specific rule applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrackerAction {
    Block,
    Ignore,
}

impl TrackerAction {
    fn parse(node: LenientJson<'_>) -> Option<Self> {
        match node.as_str()? {
            "block" => Some(Self::Block),
            "ignore" => Some(Self::Ignore),
            _ => None,
        }
    }
}

/// Restricts a rule to some first-party sites (`domains`) and/or request kinds (`types`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuleMatching {
    pub domains: Option<Vec<String>>,
    pub types: Option<Vec<String>>,
}

impl RuleMatching {
    fn parse(node: LenientJson<'_>) -> Option<Self> {
        node.is_object()
            .then(|| Self { domains: node.field("domains").strings(), types: node.field("types").strings() })
    }
}

/// A URL pattern (already a regular expression in the TDS) with its own action and scope.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TrackerRule {
    pub pattern: String,
    pub action: Option<TrackerAction>,
    pub options: Option<RuleMatching>,
    pub exceptions: Option<RuleMatching>,
}

impl TrackerRule {
    fn parse(node: LenientJson<'_>) -> Option<Self> {
        Some(Self {
            pattern: node.field("rule").as_str()?.to_owned(),
            action: TrackerAction::parse(node.field("action")),
            options: RuleMatching::parse(node.field("options")),
            exceptions: RuleMatching::parse(node.field("exceptions")),
        })
    }
}

/// A tracking domain, the company that owns it, and how to treat its requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnownTracker {
    pub domain: Option<String>,
    pub default_action: Option<TrackerAction>,
    pub owner_name: Option<String>,
    /// What the tracker is for (`Advertising`, `Analytics`, `Online Payment`, ...), as DuckDuckGo labels it.
    pub categories: Vec<String>,
    pub rules: Vec<TrackerRule>,
}

impl KnownTracker {
    fn parse(node: LenientJson<'_>) -> Option<Self> {
        node.is_object().then(|| Self {
            domain: node.field("domain").as_str().map(str::to_owned),
            default_action: TrackerAction::parse(node.field("default")),
            owner_name: node.field("owner").field("name").as_str().map(str::to_owned),
            categories: node.field("categories").strings().unwrap_or_default(),
            rules: node.field("rules").items().filter_map(TrackerRule::parse).collect(),
        })
    }

    /// The same tracker served from another domain (used for CNAME-cloaked trackers).
    #[must_use]
    pub fn relocated_to(&self, domain: &str) -> Self {
        Self { domain: Some(domain.to_owned()), ..self.clone() }
    }
}

/// Trackers are kept sorted by domain so the generated rule list, and therefore its cache key, is
/// deterministic across runs.
#[derive(Debug, Clone, Default)]
pub struct TrackerDataSet {
    trackers: BTreeMap<String, KnownTracker>,
    entity_domains: HashMap<String, Vec<String>>,
    cnames: BTreeMap<String, String>,
}

impl TrackerDataSet {
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let root = LenientJson::new(document);
        let trackers = root
            .field("trackers")
            .entries()
            .filter_map(|(domain, node)| KnownTracker::parse(node).map(|tracker| (domain.to_owned(), tracker)))
            .collect();
        let entity_domains = root
            .field("entities")
            .entries()
            .filter(|(_, entity)| entity.is_object())
            .map(|(name, entity)| (name.to_owned(), entity.field("domains").strings().unwrap_or_default()))
            .collect();
        let cnames = root
            .field("cnames")
            .entries()
            .filter_map(|(alias, target)| target.as_str().map(|target| (alias.to_owned(), target.to_owned())))
            .collect();
        Self { trackers, entity_domains, cnames }
    }

    pub fn trackers(&self) -> impl Iterator<Item = &KnownTracker> {
        self.trackers.values()
    }

    #[must_use]
    pub fn tracker(&self, domain: &str) -> Option<&KnownTracker> {
        self.trackers.get(domain)
    }

    /// `(cloaked first-party alias, CNAME target)` pairs.
    pub fn cnames(&self) -> impl Iterator<Item = (&str, &str)> {
        self.cnames.iter().map(|(alias, target)| (alias.as_str(), target.as_str()))
    }

    /// All domains of the company that owns a tracker, or `None` if the owner is unknown.
    #[must_use]
    pub fn related_domains(&self, owner_name: Option<&str>) -> Option<&[String]> {
        self.entity_domains.get(owner_name?).map(Vec::as_slice)
    }

    /// The tracker hiding behind `host`: a first-party name whose CNAME points at a tracker. The alias may be
    /// the host itself or one of its parent domains.
    #[must_use]
    pub fn tracker_cloaked_as(&self, host: &str) -> Option<&KnownTracker> {
        let target =
            DomainName::new(host).self_and_parent_domains().find_map(|candidate| self.cnames.get(candidate))?;
        self.find_tracker_by_cname(target)
    }

    /// The tracker a CNAME points at, matching the target or any of its parent domains.
    #[must_use]
    pub fn find_tracker_by_cname(&self, cname_target: &str) -> Option<&KnownTracker> {
        let target = DomainName::new(cname_target);
        target.self_and_parent_domains().find_map(|candidate| self.trackers.get(candidate))
    }
}
