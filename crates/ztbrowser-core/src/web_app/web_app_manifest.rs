//! A site's web app manifest (`<link rel="manifest">`): its app name, start page and launcher icons.
//!
//! Spec: <https://www.w3.org/TR/appmanifest/>. Relative addresses resolve against the manifest's own URL. A
//! `start_url` on another origin is ignored, as the spec requires, so a manifest cannot make the app open a
//! different site.

use serde_json::Value;
use url::Url;

use crate::document::lenient_json::LenientJson;
use crate::web_app::site_icon::IconSource;
use crate::web_app::site_icon::SiteIcon;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WebAppManifest {
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub start_url: Option<String>,
    pub icons: Vec<SiteIcon>,
}

impl WebAppManifest {
    #[must_use]
    pub fn from_json(document: &Value, manifest_url: &str) -> Self {
        let Ok(base) = Url::parse(manifest_url) else { return Self::default() };
        let root = LenientJson::new(document);
        let text = |field: &str| {
            root.field(field).as_str().map(str::trim).filter(|value| !value.is_empty()).map(str::to_owned)
        };
        Self {
            name: text("name"),
            short_name: text("short_name"),
            start_url: root.field("start_url").as_str().and_then(|start| Self::same_origin_url(&base, start)),
            icons: root.field("icons").items().filter_map(|icon| Self::icon(&base, icon)).collect(),
        }
    }

    /// The name for space-constrained places such as an application menu.
    #[must_use]
    pub fn display_name(&self) -> Option<&str> {
        self.short_name.as_deref().or(self.name.as_deref())
    }

    fn same_origin_url(base: &Url, address: &str) -> Option<String> {
        base.join(address).ok().filter(|url| url.origin() == base.origin()).map(String::from)
    }

    fn icon(base: &Url, icon: LenientJson<'_>) -> Option<SiteIcon> {
        let url = base.join(icon.field("src").as_str()?).ok().filter(|url| matches!(url.scheme(), "http" | "https"))?;
        let size = icon.field("sizes").as_str().and_then(SiteIcon::largest_declared_side);
        let purposes = icon.field("purpose").as_str().unwrap_or("any");
        let is_maskable = !purposes.split_whitespace().any(|purpose| purpose == "any");
        Some(SiteIcon { url: url.into(), size, source: IconSource::Manifest, is_maskable })
    }
}
