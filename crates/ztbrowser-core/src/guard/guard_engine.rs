//! ZTBrowser Guard's matching engine: Brave's `adblock` engine (MPL-2.0) over the downloaded community lists.
//!
//! One engine holds every list. It answers two questions fast enough to ask them for every request: whether a
//! request is blocked, and which elements a page hides (the selectors a site names, those naming no class or id,
//! and, once the page's classes and ids are known, the generic ones they call for).

use std::collections::HashSet;

use adblock::Engine;
use adblock::lists::FilterSet;
use adblock::lists::ParseOptions;
use adblock::request::Request;

use crate::network::resource_type::ResourceType;

/// The elements to hide on one page, from its address.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageCosmetics {
    /// Selectors to hide at once: those the lists name for the site, and those naming no class or id.
    pub hide_selectors: Vec<String>,
    /// Classes and ids whose generic rules must not apply on this page; hand them back with the page's classes.
    pub exceptions: HashSet<String>,
    /// Whether the lists forbid generic hiding on this page.
    pub generic_hiding_is_off: bool,
}

pub struct GuardEngine {
    engine: Engine,
}

impl GuardEngine {
    /// An engine over the text of Adblock Plus filter lists; none gives an engine that blocks and hides nothing.
    #[must_use]
    pub fn from_lists(lists: &[&str]) -> Self {
        let mut filters = FilterSet::new(false);
        for list in lists {
            let _ = filters.add_filter_list((*list).to_owned(), ParseOptions::default());
        }
        Self { engine: Engine::new_with_filter_set(filters) }
    }

    /// Whether the lists block this request. A navigation is never blocked: the person asked for that page.
    #[must_use]
    pub fn blocks(&self, address: &str, page: &str, resource: ResourceType) -> bool {
        let Some(kind) = Self::request_type(resource) else { return false };
        Request::new(address, page, kind, "get")
            .is_ok_and(|request| self.engine.check_network_request(&request).should_block())
    }

    #[must_use]
    pub fn page_cosmetics(&self, address: &str) -> PageCosmetics {
        let resources = self.engine.url_cosmetic_resources(address);
        let mut hide_selectors: Vec<String> = resources.hide_selectors.into_iter().collect();
        hide_selectors.sort_unstable();
        PageCosmetics { hide_selectors, exceptions: resources.exceptions, generic_hiding_is_off: resources.generichide }
    }

    /// The generic selectors that apply to a page showing these classes (without the dot) and ids (without the
    /// hash), unless an exception of the page cancels them.
    #[must_use]
    pub fn generic_selectors(&self, classes: &[&str], ids: &[&str], exceptions: &HashSet<String>) -> Vec<String> {
        self.engine.hidden_class_id_selectors(classes, ids, exceptions)
    }

    /// The request kinds `adblock` knows; `None` for a navigation.
    fn request_type(resource: ResourceType) -> Option<&'static str> {
        match resource {
            ResourceType::MainFrame => None,
            ResourceType::SubFrame => Some("subdocument"),
            ResourceType::Script => Some("script"),
            ResourceType::Stylesheet => Some("stylesheet"),
            ResourceType::Image => Some("image"),
            ResourceType::Font => Some("font"),
            ResourceType::Media => Some("media"),
            ResourceType::Xhr => Some("xmlhttprequest"),
            ResourceType::WebSocket => Some("websocket"),
            ResourceType::Other => Some("other"),
        }
    }
}
