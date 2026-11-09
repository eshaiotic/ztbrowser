//! Registrable domains (`google.com` for `mail.google.com`, `mbank.pl` for `online.mbank.pl`).
//!
//! libsoup ships the Public Suffix List that WebKit itself uses, so a web app agrees with WebKit's cookie and
//! ITP logic about what counts as the same site.
//!
//! Only `soup::tld_domain_is_public_suffix` is used: the `soup::tld_get_base_domain` binding (soup3 0.9) is
//! unsound, it reads its result from a pointer into a temporary copy of the host name that is already freed.

use ztbrowser_core::web_app::registrable_domain_lookup::RegistrableDomainLookup;

pub struct PublicSuffixList;

impl RegistrableDomainLookup for PublicSuffixList {
    fn is_public_suffix(&self, domain: &str) -> bool {
        soup::tld_domain_is_public_suffix(domain)
    }
}
