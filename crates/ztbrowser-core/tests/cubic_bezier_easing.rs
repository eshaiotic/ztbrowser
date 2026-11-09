//! The CSS `cubic-bezier()` easing curve that shapes the wheel animation.

use ztbrowser_core::scrolling::cubic_bezier_easing::CubicBezierEasing;

struct EasingScenario {
    easing: CubicBezierEasing,
}

impl EasingScenario {
    fn given_curve(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { easing: CubicBezierEasing::new(x1, y1, x2, y2) }
    }

    fn progress_at(&self, time: f64) -> f64 {
        self.easing.progress_at(time)
    }
}

#[test]
fn should_start_at_nothing_and_end_at_everything() {
    let scenario = EasingScenario::given_curve(0.42, 0.0, 0.58, 1.0);
    assert_eq!((scenario.progress_at(0.0), scenario.progress_at(1.0)), (0.0, 1.0));
}

#[test]
fn should_be_a_straight_line_when_the_control_points_lie_on_it() {
    let scenario = EasingScenario::given_curve(1.0 / 3.0, 1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0);
    assert!((scenario.progress_at(0.25) - 0.25).abs() < 1e-6);
}

#[test]
fn should_be_half_way_in_the_middle_of_a_symmetric_ease_in_out() {
    assert!((EasingScenario::given_curve(0.42, 0.0, 0.58, 1.0).progress_at(0.5) - 0.5).abs() < 1e-6);
}

#[test]
fn should_lag_behind_a_straight_line_at_first_when_easing_in() {
    assert!(EasingScenario::given_curve(0.42, 0.0, 0.58, 1.0).progress_at(0.1) < 0.1);
}

#[test]
fn should_run_ahead_of_a_straight_line_at_first_when_easing_out() {
    assert!(EasingScenario::given_curve(0.0, 0.0, 0.58, 1.0).progress_at(0.1) > 0.1);
}

#[test]
fn should_clamp_times_outside_the_animation() {
    let scenario = EasingScenario::given_curve(0.42, 0.0, 0.58, 1.0);
    assert_eq!((scenario.progress_at(-1.0), scenario.progress_at(2.0)), (0.0, 1.0));
}
