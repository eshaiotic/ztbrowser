//! Sites where the user switched privacy protection off, kept per profile.
//!
//! Sites are stored without a leading `www.`; WebKit's `if-domain` and the Content Scope Scripts compare
//! host names exactly, so [`UnprotectedSites::hosts_to_match`] hands them both spellings.

use std::collections::BTreeSet;

use serde_json::Value;
use serde_json::json;

use crate::document::lenient_json::LenientJson;
use crate::protection::domain_name::DomainName;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UnprotectedSites {
    domains: BTreeSet<String>,
}

impl UnprotectedSites {
    const WWW_PREFIX: &'static str = "www.";

    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let stored = LenientJson::new(document).field("domains").strings().unwrap_or_default();
        Self {
            domains: stored.iter().map(|domain| Self::site_of(domain)).filter(|domain| !domain.is_empty()).collect(),
        }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({"domains": self.domains})
    }

    pub fn switch_protection_off(&mut self, host: &str) {
        let site = Self::site_of(host);
        if !site.is_empty() {
            self.domains.insert(site);
        }
    }

    pub fn switch_protection_on(&mut self, host: &str) {
        self.domains.remove(&Self::site_of(host));
    }

    #[must_use]
    pub fn is_unprotected(&self, host: &str) -> bool {
        self.domains.contains(&Self::site_of(host))
    }

    /// Stored sites, sorted.
    #[must_use]
    pub fn domains(&self) -> Vec<&str> {
        self.domains.iter().map(String::as_str).collect()
    }

    /// Every site with and without `www.`, for consumers that compare host names exactly.
    #[must_use]
    pub fn hosts_to_match(&self) -> Vec<String> {
        self.domains.iter().flat_map(|domain| [domain.clone(), format!("{}{domain}", Self::WWW_PREFIX)]).collect()
    }

    fn site_of(host: &str) -> String {
        let domain = DomainName::new(host);
        domain.as_str().strip_prefix(Self::WWW_PREFIX).unwrap_or(domain.as_str()).to_owned()
    }
}
