//! An icon a page advertises, as a candidate for a web app's application icon.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IconSource {
    /// `<link rel="icon">`: usually 16-32 px, blurry in an application menu.
    Favicon,
    /// `<link rel="apple-touch-icon">`: 180 px by convention.
    AppleTouchIcon,
    /// An entry of the web app manifest's `icons`: made for launchers.
    Manifest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteIcon {
    pub url: String,
    /// Largest declared side in pixels, if declared.
    pub size: Option<u32>,
    pub source: IconSource,
    /// Maskable icons carry extra padding for circular masks and look shrunk as a plain icon.
    pub is_maskable: bool,
}

impl SiteIcon {
    /// `"any"` in `sizes` marks a scalable (SVG) icon.
    const SCALABLE_SIZE: u32 = 512;

    /// `"192x192 512x512"` gives 512; `"any"` gives the scalable size.
    pub(crate) fn largest_declared_side(sizes: &str) -> Option<u32> {
        sizes
            .split_whitespace()
            .filter_map(|size| {
                if size.eq_ignore_ascii_case("any") {
                    return Some(Self::SCALABLE_SIZE);
                }
                let (width, height) =
                    size.to_ascii_lowercase().split_once('x').map(|(w, h)| (w.parse::<u32>(), h.parse::<u32>()))?;
                Some(width.ok()?.max(height.ok()?))
            })
            .max()
    }

    #[must_use]
    pub fn new(url: &str, size: Option<u32>, source: IconSource) -> Self {
        Self { url: url.to_owned(), size, source, is_maskable: false }
    }

    #[must_use]
    pub fn maskable(url: &str, size: Option<u32>) -> Self {
        Self { is_maskable: true, ..Self::new(url, size, IconSource::Manifest) }
    }
}
