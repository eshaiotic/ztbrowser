//! Page zoom of one profile: each site keeps the size the user gave it, on every page and in every window.
//!
//! WebKit keeps one zoom factor per web view and carries it across navigations, so a page zoomed on one site
//! would make the next site large too. Instead every page, once committed, is drawn at its own site's size,
//! and a change of zoom applies at once to every open page of that site, as in DuckDuckGo's macOS browser.

use std::cell::RefCell;

use glib::prelude::ObjectExt;
use webkit6::WebView;
use webkit6::prelude::WebViewExt;
use ztbrowser_core::page_zoom::site_zoom_levels::SiteZoomLevels;
use ztbrowser_core::page_zoom::zoom_level::ZoomLevel;

use crate::public_suffix_list::PublicSuffixList;
use crate::stored_file::StoredFile;

pub struct SiteZoom {
    file: StoredFile,
    levels: RefCell<SiteZoomLevels>,
    /// Pages to keep at their site's size; dropped once WebKit frees them.
    views: RefCell<Vec<glib::WeakRef<WebView>>>,
}

impl SiteZoom {
    const LOG_DOMAIN: &'static str = "ztbrowser";

    pub fn load(file: StoredFile) -> Self {
        let levels = file.read_json().map(|document| SiteZoomLevels::from_json(&document)).unwrap_or_default();
        Self { file, levels: RefCell::new(levels), views: RefCell::new(Vec::new()) }
    }

    /// Keeps `view` at its site's size whenever the size of the site changes.
    pub fn follow(&self, view: &WebView) {
        self.views.borrow_mut().push(view.downgrade());
    }

    /// Draws the page `view` just committed at the size of its site.
    pub fn apply(&self, view: &WebView) {
        let address = view.uri().unwrap_or_default();
        let level = self.levels.borrow().level_for(&address, &PublicSuffixList);
        view.set_zoom_level(level.factor());
    }

    /// The size `view` draws its page at.
    pub fn current(view: &WebView) -> ZoomLevel {
        ZoomLevel::nearest_to_factor(view.zoom_level())
    }

    /// Changes the size of `view`'s page and remembers it for the page's site, whose other pages follow.
    pub fn change(&self, view: &WebView, change: fn(ZoomLevel) -> ZoomLevel) {
        let level = change(Self::current(view));
        view.set_zoom_level(level.factor());
        let address = view.uri().unwrap_or_default();
        let Some(site) = self.levels.borrow_mut().remember(&address, level, &PublicSuffixList) else { return };
        if let Err(error) = self.file.write_json(&self.levels.borrow().to_json()) {
            glib::g_warning!(Self::LOG_DOMAIN, "cannot save zoom levels: {error}");
        }
        self.views.borrow_mut().retain(|followed| followed.upgrade().is_some());
        let views: Vec<WebView> = self.views.borrow().iter().filter_map(glib::WeakRef::upgrade).collect();
        for other in views.iter().filter(|other| *other != view) {
            let other_site = other.uri().and_then(|address| SiteZoomLevels::site_of(&address, &PublicSuffixList));
            if other_site.as_deref() == Some(site.as_str()) {
                other.set_zoom_level(level.factor());
            }
        }
    }

    /// Forgets the size of every site; part of burning the browsing data.
    pub fn forget_all(&self) {
        self.levels.borrow_mut().forget_all();
        if let Err(error) = self.file.remove() {
            glib::g_warning!(Self::LOG_DOMAIN, "cannot delete zoom levels: {error}");
        }
    }
}
