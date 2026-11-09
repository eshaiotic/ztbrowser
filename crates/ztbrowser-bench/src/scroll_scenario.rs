//! How the page is scrolled while frames are counted.

const SCRIPT_TEMPLATE: &str = include_str!("../pages/scroll.js");

/// Scroll speed and timing of one run. The defaults are the ones of the 2026-11 engine comparison, so new
/// numbers stay comparable with the ones in the performance report.
#[derive(Debug, Clone, Copy)]
pub struct ScrollScenario {
    speed_pixels_per_second: u32,
    duration_ms: u32,
    settle_ms: u32,
}

impl ScrollScenario {
    #[must_use]
    pub fn new(speed_pixels_per_second: u32, duration_ms: u32, settle_ms: u32) -> Self {
        Self { speed_pixels_per_second, duration_ms, settle_ms }
    }

    /// 1400 px/s for 8 s, starting 6 s after the page's `load` event so lazy content has settled.
    #[must_use]
    pub fn comparison_default() -> Self {
        Self::new(1400, 8000, 6000)
    }

    #[must_use]
    pub fn duration_ms(&self) -> u32 {
        self.duration_ms
    }

    /// The page script with this scenario's numbers and the two addresses it reports to.
    #[must_use]
    pub fn script(&self, loaded_url: &str, result_url: &str) -> String {
        SCRIPT_TEMPLATE
            .replace("/*SPEED_PX_PER_SECOND*/0", &self.speed_pixels_per_second.to_string())
            .replace("/*DURATION_MS*/0", &self.duration_ms.to_string())
            .replace("/*SETTLE_MS*/0", &self.settle_ms.to_string())
            .replace("/*LOADED_URL*/", loaded_url)
            .replace("/*RESULT_URL*/", result_url)
    }
}
