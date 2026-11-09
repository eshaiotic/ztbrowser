//! Sites where the person allowed third-party requests although the profile blocks them by default.

use std::collections::BTreeSet;

use serde_json::Value;
use serde_json::json;

use crate::document::lenient_json::LenientJson;
use crate::protection::domain_name::DomainName;

#[derive(Debug, Clone, Default)]
pub struct SiteAllowList {
    sites: BTreeSet<DomainName>,
}

impl SiteAllowList {
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let sites = LenientJson::new(document)
            .field("allowedSites")
            .strings()
            .unwrap_or_default()
            .iter()
            .filter(|site| !site.trim().is_empty())
            .map(|site| DomainName::new(site))
            .collect();
        Self { sites }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({"allowedSites": self.sites.iter().map(DomainName::as_str).collect::<Vec<_>>()})
    }

    pub fn allow_third_parties_on(&mut self, site: &str) {
        self.sites.insert(DomainName::new(site));
    }

    pub fn forbid_third_parties_on(&mut self, site: &str) {
        self.sites.remove(&DomainName::new(site));
    }

    /// Allowed sites in alphabetical order, for the list the person can see and edit.
    #[must_use]
    pub fn sites(&self) -> Vec<&str> {
        self.sites.iter().map(DomainName::as_str).collect()
    }

    /// True for the listed site and all of its subdomains.
    #[must_use]
    pub fn allows_third_parties_on(&self, host: &str) -> bool {
        DomainName::new(host).self_and_parent_domains().any(|candidate| self.sites.contains(candidate))
    }
}
