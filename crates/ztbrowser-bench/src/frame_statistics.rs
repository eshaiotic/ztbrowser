//! What the gaps between animation frames say about smoothness.

use std::cmp::Ordering;

/// The measured gaps (milliseconds between two consecutive frames) of one scripted scroll.
#[derive(Debug, Clone)]
pub struct FrameStatistics {
    sorted_gaps_ms: Vec<f64>,
}

impl FrameStatistics {
    #[must_use]
    pub fn from_gaps_ms(gaps_ms: &[f64]) -> Self {
        let mut sorted_gaps_ms = gaps_ms.to_vec();
        sorted_gaps_ms.sort_by(|left, right| left.partial_cmp(right).unwrap_or(Ordering::Equal));
        Self { sorted_gaps_ms }
    }

    #[must_use]
    pub fn frame_count(&self) -> usize {
        self.sorted_gaps_ms.len()
    }

    /// Frames drawn per second over the whole run; zero when nothing was measured.
    #[must_use]
    pub fn frames_per_second(&self) -> f64 {
        let total_ms: f64 = self.sorted_gaps_ms.iter().sum();
        if total_ms <= 0.0 {
            return 0.0;
        }
        Self::as_float(self.frame_count()) * 1000.0 / total_ms
    }

    /// Nearest-rank percentile of the gaps; `percent` is 1..=100. Zero when nothing was measured.
    #[must_use]
    pub fn percentile_gap_ms(&self, percent: u32) -> f64 {
        let count = self.frame_count();
        if count == 0 {
            return 0.0;
        }
        let percent = usize::try_from(percent.clamp(1, 100)).unwrap_or(100);
        let rank = (count * percent).div_ceil(100).max(1);
        self.sorted_gaps_ms[rank - 1]
    }

    #[must_use]
    pub fn longest_gap_ms(&self) -> f64 {
        self.sorted_gaps_ms.last().copied().unwrap_or(0.0)
    }

    #[must_use]
    pub fn frames_longer_than_ms(&self, threshold_ms: f64) -> usize {
        self.sorted_gaps_ms.iter().filter(|gap| **gap > threshold_ms).count()
    }

    fn as_float(count: usize) -> f64 {
        f64::from(u32::try_from(count).unwrap_or(u32::MAX))
    }
}
