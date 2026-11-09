//! A site the user installed as its own app.

use url::Url;

use crate::web_app::web_app_id::WebAppId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledWebApp {
    pub id: WebAppId,
    /// Shown in the application menu and the window title.
    pub name: String,
    /// Always an `http` or `https` URL (checked on install and on load).
    pub start_url: String,
}

impl InstalledWebApp {
    /// The start page's host, e.g. `online.mbank.pl`.
    #[must_use]
    pub fn host(&self) -> String {
        Url::parse(&self.start_url).ok().and_then(|url| url.host_str().map(str::to_owned)).unwrap_or_default()
    }

    /// Whether `address` is one of the app's own pages: the same origin (scheme, host and port) as the start
    /// page. Narrower than the app's *site*, so installing `mail.google.com` claims no `www.google.com` page.
    #[must_use]
    pub fn is_own_page(&self, address: &str) -> bool {
        let (Ok(page), Ok(start)) = (Url::parse(address), Url::parse(&self.start_url)) else { return false };
        let origin = page.origin();
        origin.is_tuple() && origin == start.origin()
    }
}
