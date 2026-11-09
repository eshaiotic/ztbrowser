//! A tracker URL that DuckDuckGo deliberately lets through because blocking it breaks sites.

use crate::document::lenient_json::LenientJson;
use crate::protection::domain_name::DomainName;

/// `rule` is a plain `host/path` prefix (not a regex); `domains: None` means "on every site".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackerAllowlistEntry {
    pub rule: String,
    pub domains: Option<Vec<String>>,
}

impl TrackerAllowlistEntry {
    /// The privacy configuration marks "every site" with this pseudo-domain.
    const ALL_SITES_MARKER: &'static str = "<all>";

    /// Reads one `trackerAllowlist` rule; site domains are normalised to the form WebKit expects.
    #[must_use]
    pub fn from_config_rule(node: LenientJson<'_>) -> Option<Self> {
        let rule = node.field("rule").as_str()?.to_owned();
        let domains = node.field("domains").strings().unwrap_or_default();
        if domains.iter().any(|domain| domain == Self::ALL_SITES_MARKER) {
            return Some(Self { rule, domains: None });
        }
        let normalized = domains.iter().map(|domain| DomainName::new(domain).to_string()).collect();
        Some(Self { rule, domains: Some(normalized) })
    }
}
