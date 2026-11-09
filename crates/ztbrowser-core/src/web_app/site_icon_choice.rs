//! Picks the sharpest icon for an installed web app.
//!
//! Order of preference: regular over maskable icons, then the larger size (anything from 512 px up counts as
//! "large enough", so the source decides between two large icons), then manifest over apple-touch-icon over
//! favicon. Undeclared sizes are assumed from convention.

use crate::web_app::site_icon::IconSource;
use crate::web_app::site_icon::SiteIcon;

pub struct SiteIconChoice<'icons> {
    icons: &'icons [SiteIcon],
}

impl<'icons> SiteIconChoice<'icons> {
    const LARGE_ENOUGH: u32 = 512;
    const ASSUMED_APPLE_TOUCH_SIZE: u32 = 180;
    const ASSUMED_FAVICON_SIZE: u32 = 32;
    const ASSUMED_MANIFEST_SIZE: u32 = 144;

    #[must_use]
    pub fn new(icons: &'icons [SiteIcon]) -> Self {
        Self { icons }
    }

    #[must_use]
    pub fn best(&self) -> Option<&'icons SiteIcon> {
        self.icons
            .iter()
            .max_by_key(|icon| (!icon.is_maskable, Self::effective_size(icon).min(Self::LARGE_ENOUGH), icon.source))
    }

    fn effective_size(icon: &SiteIcon) -> u32 {
        icon.size.unwrap_or(match icon.source {
            IconSource::Favicon => Self::ASSUMED_FAVICON_SIZE,
            IconSource::AppleTouchIcon => Self::ASSUMED_APPLE_TOUCH_SIZE,
            IconSource::Manifest => Self::ASSUMED_MANIFEST_SIZE,
        })
    }
}
