//! What the frame gaps of a scripted scroll say, checked with the numbers of the 2026-11 engine comparison.

use ztbrowser_bench::frame_statistics::FrameStatistics;

struct StatisticsScenario {
    statistics: FrameStatistics,
}

impl StatisticsScenario {
    fn given_gaps_ms(gaps: &[f64]) -> Self {
        Self { statistics: FrameStatistics::from_gaps_ms(gaps) }
    }

    fn given_steady_gaps(gap_ms: f64, count: usize) -> Self {
        Self::given_gaps_ms(&vec![gap_ms; count])
    }

    fn then_frames_per_second_is_about(&self, expected: f64) {
        let actual = self.statistics.frames_per_second();
        assert!((actual - expected).abs() < 0.1, "fps {actual} is not about {expected}");
    }

    fn then_percentile_gap_is(&self, percent: u32, expected: f64) {
        assert!((self.statistics.percentile_gap_ms(percent) - expected).abs() < 1e-9);
    }

    fn then_frames_longer_than_33_ms_are(&self, expected: usize) {
        assert_eq!(self.statistics.frames_longer_than_ms(33.0), expected);
    }

    fn then_longest_gap_is(&self, expected: f64) {
        assert!((self.statistics.longest_gap_ms() - expected).abs() < 1e-9);
    }
}

#[test]
fn should_report_60_fps_for_16_7_ms_gaps() {
    StatisticsScenario::given_steady_gaps(16.667, 480).then_frames_per_second_is_about(60.0);
}

#[test]
fn should_count_frames_longer_than_33_ms() {
    StatisticsScenario::given_gaps_ms(&[16.0, 34.0, 33.0, 80.0, 16.0]).then_frames_longer_than_33_ms_are(2);
}

#[test]
fn should_return_the_nearest_rank_p95_of_unsorted_gaps() {
    let mut gaps: Vec<f64> = (1..=100).map(f64::from).rev().collect();
    gaps.swap(0, 50);
    StatisticsScenario::given_gaps_ms(&gaps).then_percentile_gap_is(95, 95.0);
}

#[test]
fn should_report_the_longest_gap() {
    StatisticsScenario::given_gaps_ms(&[16.0, 120.0, 17.0]).then_longest_gap_is(120.0);
}

#[test]
fn should_report_zero_for_an_empty_run() {
    let scenario = StatisticsScenario::given_gaps_ms(&[]);
    scenario.then_frames_per_second_is_about(0.0);
    scenario.then_percentile_gap_is(95, 0.0);
}
