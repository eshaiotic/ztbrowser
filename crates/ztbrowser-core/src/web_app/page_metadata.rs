//! What the open page says about itself: title, manifest link and icon links.
//!
//! The browser collects these with a script in an isolated JavaScript world and hands over a JSON object
//! `{"title", "manifest", "icons": [{"href", "rel", "sizes"}]}` with absolute addresses. The page controls all
//! of it, so only `http(s)` addresses survive: no `javascript:`, `data:` or `file:` links reach the installer.

use serde_json::Value;
use url::Url;

use crate::document::lenient_json::LenientJson;
use crate::web_app::site_icon::IconSource;
use crate::web_app::site_icon::SiteIcon;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageMetadata {
    pub title: Option<String>,
    pub manifest_url: Option<String>,
    pub icons: Vec<SiteIcon>,
}

impl PageMetadata {
    /// The script that collects the metadata; it returns the JSON text [`PageMetadata::from_json`] reads.
    pub const COLLECTOR_SCRIPT: &'static str = r#"JSON.stringify({
        title: document.title,
        manifest: document.querySelector('link[rel~="manifest"]')?.href ?? null,
        icons: Array.from(document.querySelectorAll('link[rel~="icon"], link[rel~="apple-touch-icon"], link[rel~="apple-touch-icon-precomposed"]'))
            .map(link => ({href: link.href, rel: link.rel, sizes: link.getAttribute('sizes') ?? ''}))
    })"#;

    #[must_use]
    pub fn from_json(collected: &Value) -> Self {
        let root = LenientJson::new(collected);
        Self {
            title: root.field("title").as_str().map(str::trim).filter(|title| !title.is_empty()).map(str::to_owned),
            manifest_url: root.field("manifest").as_str().and_then(Self::web_address),
            icons: root.field("icons").items().filter_map(Self::icon).collect(),
        }
    }

    fn icon(link: LenientJson<'_>) -> Option<SiteIcon> {
        let url = Self::web_address(link.field("href").as_str()?)?;
        let is_touch_icon =
            link.field("rel").as_str()?.split_whitespace().any(|rel| rel.starts_with("apple-touch-icon"));
        let source = if is_touch_icon { IconSource::AppleTouchIcon } else { IconSource::Favicon };
        let size = link.field("sizes").as_str().and_then(SiteIcon::largest_declared_side);
        Some(SiteIcon::new(&url, size, source))
    }

    fn web_address(text: &str) -> Option<String> {
        Url::parse(text).ok().filter(|url| matches!(url.scheme(), "http" | "https")).map(String::from)
    }
}
