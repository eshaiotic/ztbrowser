//! DuckDuckGo's remote privacy configuration (`macos-config.json`).
//!
//! The configuration switches protections on and off without shipping a new app: each *feature* has a
//! `state` and a list of *exception* sites where it is known to break pages. `unprotectedTemporary` lists
//! sites where every protection is paused while DuckDuckGo fixes breakage.
//!
//! Features may require a minimum app version (`minSupportedVersion`). DuckDuckGo compares it with the macOS
//! app version; ZTBrowser compares it with the *compatibility version*: the macOS release whose Content Scope
//! Scripts are bundled with ZTBrowser. Bump both together.

use std::collections::HashSet;

use serde_json::Value;

use crate::document::lenient_json::LenientJson;
use crate::protection::app_version::AppVersion;
use crate::protection::domain_name::DomainName;
use crate::protection::site_list::SiteList;
use crate::protection::tracker_allowlist_entry::TrackerAllowlistEntry;

#[derive(Debug, Clone)]
pub struct PrivacyConfiguration {
    document: Value,
    compatibility_version: AppVersion,
}

impl PrivacyConfiguration {
    /// The macOS app version whose Content Scope Scripts are bundled; the configuration gates features on it.
    pub const COMPATIBILITY_VERSION: &'static str = "1.214.0";

    #[must_use]
    pub fn new(document: Value, compatibility_version: &str) -> Self {
        Self { document, compatibility_version: AppVersion::parse(compatibility_version) }
    }

    /// The raw JSON, which Content Scope Scripts read directly.
    #[must_use]
    pub fn document(&self) -> &Value {
        &self.document
    }

    #[must_use]
    pub fn version(&self) -> u64 {
        self.root().field("version").as_u64().unwrap_or(0)
    }

    #[must_use]
    pub fn is_enabled(&self, feature: &str) -> bool {
        let feature = self.feature(feature);
        if feature.field("state").as_str() != Some("enabled") {
            return false;
        }
        feature
            .field("minSupportedVersion")
            .as_str()
            .is_none_or(|minimum| AppVersion::parse(minimum) <= self.compatibility_version)
    }

    /// Whether `feature` protects `host`, given the sites the user switched protection off for.
    #[must_use]
    pub fn is_enabled_for_site(&self, feature: &str, host: &str, user_unprotected_domains: &[String]) -> bool {
        if !self.is_enabled(feature) {
            return false;
        }
        let host = DomainName::new(host);
        if user_unprotected_domains.iter().any(|domain| DomainName::new(domain) == host) {
            return false;
        }
        let temporary = self.temporary_unprotected_domains();
        if SiteList::new(temporary.iter().map(String::as_str)).covers(host.as_str()) {
            return false;
        }
        let exceptions = self.exception_domains(feature);
        !SiteList::new(exceptions.iter().map(String::as_str)).covers(host.as_str())
    }

    #[must_use]
    pub fn exception_domains(&self, feature: &str) -> Vec<String> {
        Self::listed_domains(self.feature(feature).field("exceptions"))
    }

    #[must_use]
    pub fn temporary_unprotected_domains(&self) -> Vec<String> {
        Self::listed_domains(self.root().field("unprotectedTemporary"))
    }

    #[must_use]
    pub fn tracker_allowlist(&self) -> Vec<TrackerAllowlistEntry> {
        if !self.is_enabled("trackerAllowlist") {
            return vec![];
        }
        let trackers = self.feature("trackerAllowlist").field("settings").field("allowlistedTrackers");
        let rules = trackers.entries().flat_map(|(_, tracker)| tracker.field("rules").items());
        rules.filter_map(TrackerAllowlistEntry::from_config_rule).collect()
    }

    #[must_use]
    pub fn tracking_parameters(&self) -> HashSet<String> {
        if !self.is_enabled("trackingParameters") {
            return HashSet::new();
        }
        let parameters = self.feature("trackingParameters").field("settings").field("parameters");
        parameters.strings().unwrap_or_default().into_iter().collect()
    }

    fn root(&self) -> LenientJson<'_> {
        LenientJson::new(&self.document)
    }

    fn feature(&self, feature: &str) -> LenientJson<'_> {
        self.root().field("features").field(feature)
    }

    /// Domains of `[{"domain": "x.com", "reason": "..."}]` lists; blank entries are dropped.
    fn listed_domains(entries: LenientJson<'_>) -> Vec<String> {
        let domains = entries.items().filter_map(|entry| entry.field("domain").as_str());
        domains.map(str::trim).filter(|domain| !domain.is_empty()).map(str::to_owned).collect()
    }
}
