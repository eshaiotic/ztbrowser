//! The public suffix list (<https://publicsuffix.org>) compiled into the binary.

use crate::web_app::registrable_domain_lookup::RegistrableDomainLookup;

/// Answers "is this a public suffix?" from the `psl` crate's copy of the list, so the engine adapter needs no
/// platform library to tell `mail.google.com` and `accounts.google.com` belong to one site.
#[derive(Debug, Clone, Copy, Default)]
pub struct PublicSuffixLookup;

impl RegistrableDomainLookup for PublicSuffixLookup {
    fn is_public_suffix(&self, domain: &str) -> bool {
        psl::suffix(domain.as_bytes()).is_some_and(|suffix| suffix.is_known() && suffix.as_bytes() == domain.as_bytes())
    }
}
