//! A list of sites where every subdomain of a listed domain counts as listed too.

use std::collections::BTreeSet;

use crate::protection::domain_name::DomainName;

#[derive(Debug, Clone, Default)]
pub struct SiteList {
    domains: BTreeSet<DomainName>,
}

impl SiteList {
    pub fn new<'domain>(domains: impl IntoIterator<Item = &'domain str>) -> Self {
        let domains = domains.into_iter().filter(|domain| !domain.trim().is_empty()).map(DomainName::new).collect();
        Self { domains }
    }

    /// Listed domains in alphabetical order.
    pub fn domains(&self) -> impl Iterator<Item = &str> {
        self.domains.iter().map(DomainName::as_str)
    }

    /// `covers("www.shop.com")` is true when `shop.com` or `www.shop.com` is listed.
    ///
    /// The host is checked label by label from the left, so `notshop.com` never matches `shop.com`.
    #[must_use]
    pub fn covers(&self, host: &str) -> bool {
        DomainName::new(host).self_and_parent_domains().any(|candidate| self.domains.contains(candidate))
    }
}
