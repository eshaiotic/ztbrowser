//! One attempt to read or set a cookie, as the engine reports it.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CookieAccess {
    cookie_domain: String,
    first_party_host: Option<String>,
    is_third_party: bool,
}

impl CookieAccess {
    /// `first_party_host` is the host of the page the cookie is used on; `None` when the engine has no page
    /// (an address-bar navigation).
    #[must_use]
    pub fn new(cookie_domain: &str, first_party_host: Option<&str>, is_third_party: bool) -> Self {
        Self {
            cookie_domain: cookie_domain.trim_start_matches('.').to_ascii_lowercase(),
            first_party_host: first_party_host.map(str::to_ascii_lowercase),
            is_third_party,
        }
    }

    /// The domain the cookie belongs to, without a leading dot.
    #[must_use]
    pub fn cookie_domain(&self) -> &str {
        &self.cookie_domain
    }

    #[must_use]
    pub fn first_party_host(&self) -> Option<&str> {
        self.first_party_host.as_deref()
    }

    /// As the engine classified it. A cookie whose page is unknown stays third-party when the engine says so:
    /// when in doubt, protections apply.
    #[must_use]
    pub fn is_third_party(&self) -> bool {
        self.is_third_party
    }
}
