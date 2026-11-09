//! A host name in the form WebKit rules and URL hosts use.

use std::borrow::Borrow;
use std::fmt;

/// Lower-case ASCII (punycode), without a trailing dot: `Żółw.PL.` becomes `xn--w-uga1v8h.pl`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct DomainName {
    ascii: String,
}

impl DomainName {
    /// Names (and their subdomains) that only resolve inside a home or office network.
    pub const LOCAL_NETWORK_DOMAINS: [&'static str; 5] = ["localhost", "local", "lan", "internal", "home.arpa"];
    /// Tor onion services (RFC 7686): reachable only through Tor, which encrypts them end to end.
    pub const ONION_DOMAIN: &'static str = "onion";

    #[must_use]
    pub fn new(raw: &str) -> Self {
        let cleaned = raw.trim().trim_end_matches('.');
        let ascii = idna::domain_to_ascii(cleaned).unwrap_or_else(|_| cleaned.to_lowercase());
        Self { ascii }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.ascii
    }

    /// `www.shop.com` yields `www.shop.com`, then `shop.com`; the bare top-level label is never yielded,
    /// so listing `shop.com` never accidentally covers every `.com` site.
    pub fn self_and_parent_domains(&self) -> impl Iterator<Item = &str> {
        let mut next = Some(self.ascii.as_str());
        std::iter::from_fn(move || {
            let current = next.filter(|candidate| candidate.contains('.'))?;
            next = current.split_once('.').map(|(_, parent)| parent);
            Some(current)
        })
    }

    /// Single-label names (`nas`), `localhost` and local suffixes (`printer.local`).
    #[must_use]
    pub fn is_local_network(&self) -> bool {
        !self.ascii.contains('.')
            || Self::LOCAL_NETWORK_DOMAINS.iter().any(|domain| {
                self.ascii == *domain || self.ascii.strip_suffix(domain).is_some_and(|rest| rest.ends_with('.'))
            })
    }

    /// An onion service address (`duckduckgogg42….onion`) or one of its subdomains.
    #[must_use]
    pub fn is_onion(&self) -> bool {
        self.ascii.strip_suffix(Self::ONION_DOMAIN).is_some_and(|rest| rest.ends_with('.'))
    }
}

impl Borrow<str> for DomainName {
    fn borrow(&self) -> &str {
        &self.ascii
    }
}

impl fmt::Display for DomainName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.ascii)
    }
}
