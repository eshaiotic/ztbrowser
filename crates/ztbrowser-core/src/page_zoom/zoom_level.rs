//! How large a page is drawn: one of a fixed set of sizes that zoom commands step between.
//!
//! The sizes are those of DuckDuckGo's macOS browser: close together around 100 %, where people fine-tune a
//! page they can almost read, and far apart at the ends. Fixed sizes also mean that zooming in and out again
//! returns exactly to where a page was, and that a stored size is always one the menu can show.

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ZoomLevel {
    percent: u16,
}

impl ZoomLevel {
    const SIZES: [u16; 11] = [50, 75, 85, 100, 115, 125, 150, 175, 200, 250, 300];

    /// The page as its site made it.
    pub const ACTUAL_SIZE: Self = Self { percent: 100 };

    /// The size of `percent`, when it is one of the sizes zoom steps between.
    #[must_use]
    pub fn from_percent(percent: u64) -> Option<Self> {
        Self::SIZES.into_iter().find(|size| u64::from(*size) == percent).map(|percent| Self { percent })
    }

    /// The size closest to a zoom factor as WebKit reports it (`1.0` is 100 %).
    #[must_use]
    pub fn nearest_to_factor(factor: f64) -> Self {
        let distance = |size: &u16| (f64::from(*size) / 100.0 - factor).abs();
        let nearest = Self::SIZES.into_iter().min_by(|one, other| distance(one).total_cmp(&distance(other)));
        nearest.map_or(Self::ACTUAL_SIZE, |percent| Self { percent })
    }

    /// The zoom factor WebKit draws this size with.
    #[must_use]
    pub fn factor(self) -> f64 {
        f64::from(self.percent) / 100.0
    }

    /// One size larger; the largest size stays.
    #[must_use]
    pub fn zoomed_in(self) -> Self {
        Self::SIZES.into_iter().find(|size| *size > self.percent).map_or(self, |percent| Self { percent })
    }

    /// One size smaller; the smallest size stays.
    #[must_use]
    pub fn zoomed_out(self) -> Self {
        Self::SIZES.into_iter().rev().find(|size| *size < self.percent).map_or(self, |percent| Self { percent })
    }

    #[must_use]
    pub fn can_zoom_in(self) -> bool {
        self.zoomed_in() != self
    }

    #[must_use]
    pub fn can_zoom_out(self) -> bool {
        self.zoomed_out() != self
    }

    #[must_use]
    pub fn is_actual_size(self) -> bool {
        self == Self::ACTUAL_SIZE
    }

    /// The size as a whole percentage, as a person reads it (`125`).
    #[must_use]
    pub fn percent(self) -> u16 {
        self.percent
    }
}

/// `115%`, as the menu and the zoom badge show it.
impl fmt::Display for ZoomLevel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}%", self.percent)
    }
}
