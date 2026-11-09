//! The size each site's pages are shown at: a person zooms one page and every page of that site follows.

use std::path::Path;

use ztbrowser_core::network::public_suffix_lookup::PublicSuffixLookup;
use ztbrowser_core::page_zoom::site_zoom_levels::SiteZoomLevels;
use ztbrowser_core::page_zoom::zoom_level::ZoomLevel;
use ztbrowser_core::storage::owned_json_file::OwnedJsonFile;

pub struct SiteZoomStore {
    levels: SiteZoomLevels,
    file: OwnedJsonFile,
}

impl SiteZoomStore {
    #[must_use]
    pub fn open(directory: &Path) -> Self {
        let file = OwnedJsonFile::new(directory.join("site-zoom-levels.json"));
        let levels = file.read().map(|document| SiteZoomLevels::from_json(&document)).unwrap_or_default();
        Self { levels, file }
    }

    #[must_use]
    pub fn level_for(&self, address: &str) -> ZoomLevel {
        self.levels.level_for(address, &PublicSuffixLookup)
    }

    #[must_use]
    pub fn zoomed_in(&mut self, address: &str) -> ZoomLevel {
        let level = self.level_for(address).zoomed_in();
        self.remember(address, level)
    }

    #[must_use]
    pub fn zoomed_out(&mut self, address: &str) -> ZoomLevel {
        let level = self.level_for(address).zoomed_out();
        self.remember(address, level)
    }

    #[must_use]
    pub fn reset(&mut self, address: &str) -> ZoomLevel {
        self.remember(address, ZoomLevel::ACTUAL_SIZE)
    }

    fn remember(&mut self, address: &str, level: ZoomLevel) -> ZoomLevel {
        if self.levels.remember(address, level, &PublicSuffixLookup).is_some() {
            // Sizes that cannot be written are lost at exit; the zoom still applies to this session.
            let _ = self.file.write(&self.levels.to_json());
            level
        } else {
            ZoomLevel::ACTUAL_SIZE
        }
    }
}
