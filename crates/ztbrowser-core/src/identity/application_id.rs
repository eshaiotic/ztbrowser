//! GTK / D-Bus application identifiers.
//!
//! The desktop groups windows and shows icons by application id, so every installed web app gets its own id
//! (and runs as its own process): the mBank app is a separate program in the dock, not a browser window.
//! Extra browsing profiles get their own id too, so no two profiles ever share a process, and so does the Tor
//! window, whose process is the only one that runs ZTBrowser's private Tor.

use std::fmt;

use crate::identity::slug::Slug;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ApplicationId {
    value: String,
}

impl ApplicationId {
    const BROWSER: &'static str = "io.github.ztbrowser.ZTBrowser";
    const WEB_APP_NAMESPACE: &'static str = "WebApp";
    const PROFILE_NAMESPACE: &'static str = "Profile";
    const TOR_WINDOW: &'static str = "Tor";

    #[must_use]
    pub fn browser() -> Self {
        Self { value: Self::BROWSER.to_owned() }
    }

    /// `io.github.ztbrowser.ZTBrowser.Profile.<slug>`: every extra profile runs as its own process.
    #[must_use]
    pub fn browser_profile(slug: &Slug) -> Self {
        Self { value: format!("{}.{}.{slug}", Self::BROWSER, Self::PROFILE_NAMESPACE) }
    }

    /// `io.github.ztbrowser.ZTBrowser.Tor`: one process for every Tor window.
    #[must_use]
    pub fn tor_window() -> Self {
        Self { value: format!("{}.{}", Self::BROWSER, Self::TOR_WINDOW) }
    }

    /// `io.github.ztbrowser.ZTBrowser.WebApp.<slug>`; a slug always starts with a letter, as D-Bus requires.
    #[must_use]
    pub fn web_app(slug: &Slug) -> Self {
        Self { value: format!("{}.{}.{slug}", Self::BROWSER, Self::WEB_APP_NAMESPACE) }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl PartialEq<&str> for ApplicationId {
    fn eq(&self, other: &&str) -> bool {
        self.value == *other
    }
}

impl fmt::Display for ApplicationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.value)
    }
}
