//! A CSS `cubic-bezier(x1, y1, x2, y2)` easing: how far an animation has got (0 to 1) after a share of its time.

/// The curve from (0, 0) to (1, 1) bent towards two control points, as `transition-timing-function` draws it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CubicBezierEasing {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

impl CubicBezierEasing {
    /// Bisection stops when the time is this close; far below a thousandth of a pixel for any step.
    const PRECISION: f64 = 1e-9;

    #[must_use]
    pub const fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// The progress (0 to 1) after `time` (0 to 1) of the animation's duration.
    #[must_use]
    pub fn progress_at(&self, time: f64) -> f64 {
        if time <= 0.0 {
            return 0.0;
        }
        if time >= 1.0 {
            return 1.0;
        }
        let (mut low, mut high) = (0.0, 1.0);
        let mut parameter = time;
        while high - low > Self::PRECISION {
            parameter = f64::midpoint(low, high);
            if Self::coordinate(self.x1, self.x2, parameter) < time {
                low = parameter;
            } else {
                high = parameter;
            }
        }
        Self::coordinate(self.y1, self.y2, parameter)
    }

    /// One coordinate of the Bézier curve at `parameter`, the end points being 0 and 1.
    fn coordinate(first: f64, second: f64, parameter: f64) -> f64 {
        let rest = 1.0 - parameter;
        3.0 * rest * rest * parameter * first + 3.0 * rest * parameter * parameter * second + parameter.powi(3)
    }
}
