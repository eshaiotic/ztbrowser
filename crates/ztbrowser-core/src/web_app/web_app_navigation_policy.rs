//! Decides which navigations stay inside an installed web app's isolated session.
//!
//! Only a link the user clicked that leads to another site leaves the app: that is where phishing links in
//! mail come from, and the bank's or mailbox's session must never be exposed to them. Redirects and
//! script-driven navigations stay, because sign-in flows (accounts.google.com) and card payments (3-D Secure)
//! hop across sites and would break otherwise.

use url::Url;

use crate::web_app::registrable_domain_lookup::RegistrableDomainLookup;

/// What started a navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavigationTrigger {
    LinkClicked,
    RedirectOrScript,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebAppNavigation {
    StayInWebApp,
    /// Hand over to the regular browser (web pages) or the desktop (`mailto:`, `tel:`, ...).
    OpenInBrowser,
}

pub struct WebAppNavigationPolicy<'lookup> {
    app_site: Option<String>,
    lookup: &'lookup dyn RegistrableDomainLookup,
}

impl<'lookup> WebAppNavigationPolicy<'lookup> {
    /// Schemes of pages that belong to the app itself: popup placeholders, generated downloads, inline content.
    const IN_APP_SCHEMES: [&'static str; 3] = ["about", "blob", "data"];

    #[must_use]
    pub fn new(start_url: &str, lookup: &'lookup dyn RegistrableDomainLookup) -> Self {
        let app_site = Url::parse(start_url).ok().and_then(|url| Self::site_of(&url, lookup));
        Self { app_site, lookup }
    }

    #[must_use]
    pub fn decide(&self, target: &str, trigger: NavigationTrigger) -> WebAppNavigation {
        let Ok(url) = Url::parse(target) else { return WebAppNavigation::StayInWebApp };
        if Self::IN_APP_SCHEMES.contains(&url.scheme()) {
            return WebAppNavigation::StayInWebApp;
        }
        if !matches!(url.scheme(), "http" | "https") {
            return WebAppNavigation::OpenInBrowser;
        }
        if self.is_app_site(&url) || trigger == NavigationTrigger::RedirectOrScript {
            WebAppNavigation::StayInWebApp
        } else {
            WebAppNavigation::OpenInBrowser
        }
    }

    /// Whether `address` is a web page of the app's own site; a redirect may have taken the app elsewhere, and
    /// the window then has to say so, or a bank app could show a phishing page under the bank's name.
    #[must_use]
    pub fn is_app_page(&self, address: &str) -> bool {
        Url::parse(address).is_ok_and(|url| matches!(url.scheme(), "http" | "https") && self.is_app_site(&url))
    }

    fn is_app_site(&self, url: &Url) -> bool {
        self.app_site.is_some() && Self::site_of(url, self.lookup) == self.app_site
    }

    fn site_of(url: &Url, lookup: &dyn RegistrableDomainLookup) -> Option<String> {
        url.host_str().map(|host| lookup.site_of(host))
    }
}
