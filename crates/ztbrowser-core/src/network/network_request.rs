//! One request a page is about to make, as the policies see it.

use url::Url;

use crate::network::resource_type::ResourceType;
use crate::web_app::registrable_domain_lookup::RegistrableDomainLookup;

/// The page a request is made from, as far as the engine told us.
#[derive(Debug, Clone, PartialEq, Eq)]
enum FirstParty {
    /// The engine gave no page (workers, some internal requests).
    Unknown,
    /// A page without a host of its own (`data:`, `about:blank`, a sandboxed frame).
    Opaque,
    Host(String),
}

#[derive(Debug, Clone)]
pub struct NetworkRequest {
    address: String,
    parsed: Option<Url>,
    first_party: FirstParty,
    resource_type: ResourceType,
    request_id: u64,
}

impl NetworkRequest {
    /// `first_party` is the address of the page that makes the request. A `blob:` page counts as the site
    /// that created it. An address that does not parse is kept as is and has no host.
    #[must_use]
    pub fn new(address: &str, first_party: Option<&str>, resource_type: ResourceType, request_id: u64) -> Self {
        Self {
            address: address.to_owned(),
            parsed: Url::parse(address).ok(),
            first_party: first_party.map_or(FirstParty::Unknown, Self::first_party_of),
            resource_type,
            request_id,
        }
    }

    fn first_party_of(page: &str) -> FirstParty {
        let Ok(mut parsed) = Url::parse(page) else { return FirstParty::Opaque };
        if parsed.scheme() == "blob" {
            let Ok(creator) = Url::parse(parsed.path()) else { return FirstParty::Opaque };
            parsed = creator;
        }
        parsed.host_str().map_or(FirstParty::Opaque, |host| FirstParty::Host(host.to_ascii_lowercase()))
    }

    #[must_use]
    pub fn address(&self) -> &str {
        &self.address
    }

    #[must_use]
    pub fn host(&self) -> Option<&str> {
        self.parsed.as_ref().and_then(Url::host_str)
    }

    /// Host of the page making the request; `None` when the page is unknown or has no host.
    #[must_use]
    pub fn first_party_host(&self) -> Option<&str> {
        match &self.first_party {
            FirstParty::Host(host) => Some(host),
            FirstParty::Unknown | FirstParty::Opaque => None,
        }
    }

    #[must_use]
    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }

    #[must_use]
    pub fn request_id(&self) -> u64 {
        self.request_id
    }

    /// `http`, `https`, `ws` and `wss` go over the network; `data:`, `blob:`, `file:` and the like do not.
    #[must_use]
    pub fn goes_over_the_network(&self) -> bool {
        self.parsed.as_ref().is_some_and(|url| matches!(url.scheme(), "http" | "https" | "ws" | "wss"))
    }

    #[must_use]
    pub fn is_plain_http(&self) -> bool {
        self.parsed.as_ref().is_some_and(|url| url.scheme() == "http")
    }

    /// True for the request that loads the page itself (typed address, clicked link), not its sub-resources.
    #[must_use]
    pub fn is_navigation(&self) -> bool {
        self.resource_type == ResourceType::MainFrame
    }

    /// Whether the request goes to another site than the page's. A navigation never does. A sub-resource whose
    /// page is unknown or has no host counts as third-party: when in doubt, protections apply.
    #[must_use]
    pub fn is_third_party(&self, sites: &dyn RegistrableDomainLookup) -> bool {
        let Some(host) = self.host() else { return false };
        if self.is_navigation() {
            return false;
        }
        self.first_party_host().is_none_or(|page| sites.site_of(host) != sites.site_of(page))
    }
}
