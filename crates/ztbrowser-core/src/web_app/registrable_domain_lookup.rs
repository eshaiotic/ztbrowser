//! Port to the public suffix list: which part of a host name one owner can register.
//!
//! `mail.google.com` and `accounts.google.com` share the registrable domain `google.com`, while
//! `bbc.co.uk` and `evil.co.uk` do not, because `co.uk` is a public suffix. An implementation only answers
//! whether a domain is a public suffix (the application asks libsoup's copy of the list, which WebKit uses
//! too; tests use a small fake); finding the registrable domain is the same for every list.

use std::net::IpAddr;

pub trait RegistrableDomainLookup {
    /// Whether `domain` (`com`, `co.uk`, `github.io`) is on the public suffix list.
    fn is_public_suffix(&self, domain: &str) -> bool;

    /// The longest listed suffix of `host` plus one label in front of it; an unlisted top-level domain counts
    /// as a suffix. `None` for hosts without a registrable domain (IP addresses, `localhost`, bare suffixes).
    fn registrable_domain(&self, host: &str) -> Option<String> {
        if host.starts_with('[') || host.parse::<IpAddr>().is_ok() {
            return None;
        }
        let host = host.strip_suffix('.').unwrap_or(host);
        let labels: Vec<&str> = host.split('.').collect();
        let suffix_start = (0..labels.len())
            .find(|&start| self.is_public_suffix(&labels[start..].join(".")))
            .unwrap_or(labels.len() - 1);
        (suffix_start > 0).then(|| labels[suffix_start - 1..].join("."))
    }

    /// The site `host` belongs to: its registrable domain, or the host itself when it has none (an IP address,
    /// `localhost`).
    fn site_of(&self, host: &str) -> String {
        let host = host.to_ascii_lowercase();
        self.registrable_domain(&host).unwrap_or(host)
    }
}
