//! The animation of one mouse-wheel step, shaped like Chromium's.
//!
//! WebKitGTK moves a wheel step of 93 px in about 90 ms and starts at full speed, which looks like a jerk next
//! to Chromium, which moves 120 px in about 180 ms and eases in and out. ZTBrowser animates the steps itself
//! with a page script. This type holds the numbers and the two curves; the script only plays them back.
//!
//! The script works on a *progress table*: entry `i` of `SAMPLES` is how far, from 0 to 1, the animation has
//! got after `i / (SAMPLES - 1)` of its duration. Two tables exist. A step that starts from rest eases in and
//! out. A step that arrives while the page is already moving only eases out, so a quick series of steps flows
//! on instead of slowing down and starting over with every step.

use serde_json::Value;
use serde_json::json;

use crate::scrolling::cubic_bezier_easing::CubicBezierEasing;

#[derive(Debug, Clone, PartialEq)]
pub struct SmoothWheelScroll {
    pixels_per_notch: f64,
    duration_ms: f64,
    start_curve: Vec<f64>,
    continue_curve: Vec<f64>,
}

impl SmoothWheelScroll {
    /// Entries of a progress table: 65 gives 64 intervals, fine enough to be smooth at any refresh rate.
    const SAMPLES: u32 = 65;
    /// CSS `ease-in-out`: slow, fast in the middle, slow.
    const EASE_IN_OUT: CubicBezierEasing = CubicBezierEasing::new(0.42, 0.0, 0.58, 1.0);
    /// Fast at once and slowing down: the continuation of a motion that is already under way.
    const EASE_OUT: CubicBezierEasing = CubicBezierEasing::new(0.0, 0.0, 0.58, 1.0);

    #[must_use]
    pub fn chromium_like() -> Self {
        Self {
            pixels_per_notch: 120.0,
            duration_ms: 180.0,
            start_curve: Self::table_of(Self::EASE_IN_OUT),
            continue_curve: Self::table_of(Self::EASE_OUT),
        }
    }

    /// Pixels the page moves for one notch of the wheel.
    #[must_use]
    pub fn pixels_per_notch(&self) -> f64 {
        self.pixels_per_notch
    }

    #[must_use]
    pub fn duration_ms(&self) -> f64 {
        self.duration_ms
    }

    /// The progress table of a step that starts from rest.
    #[must_use]
    pub fn start_curve(&self) -> &[f64] {
        &self.start_curve
    }

    /// The progress table of a step that arrives while the page is moving.
    #[must_use]
    pub fn continue_curve(&self) -> &[f64] {
        &self.continue_curve
    }

    /// What the page script is given: `pixelsPerNotch`, `durationMs`, `startCurve` and `continueCurve`.
    #[must_use]
    pub fn script_configuration(&self) -> Value {
        json!({
            "pixelsPerNotch": self.pixels_per_notch,
            "durationMs": self.duration_ms,
            "startCurve": self.start_curve,
            "continueCurve": self.continue_curve,
        })
    }

    fn table_of(easing: CubicBezierEasing) -> Vec<f64> {
        (0..Self::SAMPLES).map(|index| easing.progress_at(f64::from(index) / f64::from(Self::SAMPLES - 1))).collect()
    }
}
