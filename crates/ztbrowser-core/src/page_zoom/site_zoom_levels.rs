//! The zoom the user chose for each site, kept per profile, so that a site always opens at its own size.
//!
//! As in DuckDuckGo's macOS browser, a site is a registrable domain (`mbank.pl` for `online.mbank.pl`): every
//! page and subdomain of a site shares one size. A host without one (an IP address, `localhost`) is a site
//! of its own. Only sizes other than 100 % are stored. The list names sites the user visited, so the Fire
//! button deletes it together with the rest of the browsing data.

use std::collections::BTreeMap;

use serde_json::Map;
use serde_json::Value;
use serde_json::json;
use url::Url;

use crate::page_zoom::zoom_level::ZoomLevel;
use crate::web_app::registrable_domain_lookup::RegistrableDomainLookup;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SiteZoomLevels {
    levels: BTreeMap<String, ZoomLevel>,
}

impl SiteZoomLevels {
    const SITES: &'static str = "sites";

    /// Stored sizes; anything that is not one of the zoom sizes is dropped.
    #[must_use]
    pub fn from_json(document: &Value) -> Self {
        let stored = document.get(Self::SITES).and_then(Value::as_object);
        let levels = stored
            .into_iter()
            .flatten()
            .filter_map(|(site, percent)| {
                Some((site.to_ascii_lowercase(), percent.as_u64().and_then(ZoomLevel::from_percent)?))
            })
            .filter(|(site, level)| !site.is_empty() && !level.is_actual_size())
            .collect();
        Self { levels }
    }

    #[must_use]
    pub fn to_json(&self) -> Value {
        let sites: Map<String, Value> =
            self.levels.iter().map(|(site, level)| (site.clone(), json!(level.percent()))).collect();
        json!({ Self::SITES: sites })
    }

    /// The site whose pages share the zoom of `address`; `None` for pages that are not on the web.
    #[must_use]
    pub fn site_of(address: &str, lookup: &dyn RegistrableDomainLookup) -> Option<String> {
        let url = Url::parse(address).ok().filter(|url| matches!(url.scheme(), "http" | "https"))?;
        url.host_str().map(|host| lookup.site_of(host))
    }

    /// The size a page at `address` opens at.
    #[must_use]
    pub fn level_for(&self, address: &str, lookup: &dyn RegistrableDomainLookup) -> ZoomLevel {
        Self::site_of(address, lookup)
            .and_then(|site| self.levels.get(&site).copied())
            .unwrap_or(ZoomLevel::ACTUAL_SIZE)
    }

    /// Gives the site of `address` the size `level`, and names that site; pages off the web have none.
    pub fn remember(
        &mut self,
        address: &str,
        level: ZoomLevel,
        lookup: &dyn RegistrableDomainLookup,
    ) -> Option<String> {
        let site = Self::site_of(address, lookup)?;
        if level.is_actual_size() {
            self.levels.remove(&site);
        } else {
            self.levels.insert(site.clone(), level);
        }
        Some(site)
    }

    pub fn forget_all(&mut self) {
        self.levels.clear();
    }
}
