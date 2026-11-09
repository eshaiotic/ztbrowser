//! The web app the browser proposes to install from the open page, before the user confirms its name.
//!
//! The manifest wins where it has an opinion (it is written for exactly this purpose); the page fills the
//! gaps. The start page must stay on the page's own origin: a manifest hosted on a CDN cannot turn "install
//! this bank" into an app that opens somewhere else.

use url::Url;

use crate::web_app::page_metadata::PageMetadata;
use crate::web_app::site_icon::SiteIcon;
use crate::web_app::web_app_manifest::WebAppManifest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebAppDraft {
    pub name: String,
    pub start_url: String,
    /// Manifest icons first, then the page's own icon links.
    pub icons: Vec<SiteIcon>,
}

impl WebAppDraft {
    #[must_use]
    pub fn new(page_url: &str, page: &PageMetadata, manifest: Option<&WebAppManifest>) -> Self {
        let page_address = Url::parse(page_url).ok();
        let host = page_address.as_ref().and_then(Url::host_str).unwrap_or(page_url).to_owned();
        let manifest_start = manifest.and_then(|manifest| manifest.start_url.as_deref()).filter(|start| {
            let start_origin = Url::parse(start).ok().map(|start| start.origin());
            start_origin.is_some() && start_origin == page_address.as_ref().map(Url::origin)
        });
        let name = manifest.and_then(WebAppManifest::display_name).or(page.title.as_deref()).unwrap_or(&host);
        let manifest_icons = manifest.map(|manifest| manifest.icons.as_slice()).unwrap_or_default();
        Self {
            name: name.to_owned(),
            start_url: manifest_start.unwrap_or(page_url).to_owned(),
            icons: manifest_icons.iter().chain(&page.icons).cloned().collect(),
        }
    }
}
