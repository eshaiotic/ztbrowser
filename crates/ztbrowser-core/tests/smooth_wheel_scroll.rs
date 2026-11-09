//! How one step of a mouse wheel is animated: Chromium's feel (120 px over about 180 ms, easing in and out)
//! instead of WebKitGTK's abrupt 93 px in 90 ms.

use serde_json::Value;
use ztbrowser_core::scrolling::smooth_wheel_scroll::SmoothWheelScroll;

/// A display refreshing 60 times a second, which the animation is sampled by.
const FRAME_MS: f64 = 1000.0 / 60.0;
/// More frames than any step lasts; the sampler stops earlier, at the step's duration.
const FRAME_LIMIT: u32 = 600;

struct WheelStepScenario {
    scroll: SmoothWheelScroll,
}

impl WheelStepScenario {
    fn given_chromium_like_scrolling() -> Self {
        Self { scroll: SmoothWheelScroll::chromium_like() }
    }

    /// How far the page moves in each frame of one step that starts at rest.
    fn pixels_moved_per_frame_from_rest(&self) -> Vec<f64> {
        self.frames_of(self.scroll.start_curve())
    }

    /// The same for a step that arrives while the page is already moving.
    fn pixels_moved_per_frame_while_moving(&self) -> Vec<f64> {
        self.frames_of(self.scroll.continue_curve())
    }

    fn frames_of(&self, curve: &[f64]) -> Vec<f64> {
        let duration = self.scroll.duration_ms();
        let last_interval = u32::try_from(curve.len() - 1).expect("a short table");
        let at = |frame: u32| {
            let position = (f64::from(frame) * FRAME_MS / duration).min(1.0) * f64::from(last_interval);
            let below = (0..=last_interval).rev().find(|index| f64::from(*index) <= position).expect("position >= 0");
            let above = (below + 1).min(last_interval);
            let share = position - f64::from(below);
            let (below, above) = (usize::try_from(below).expect("small"), usize::try_from(above).expect("small"));
            (curve[below] * (1.0 - share) + curve[above] * share) * self.scroll.pixels_per_notch()
        };
        (1_u32..=FRAME_LIMIT)
            .take_while(|frame| f64::from(frame - 1) * FRAME_MS < duration)
            .map(|frame| at(frame) - at(frame - 1))
            .collect()
    }

    fn configuration(&self) -> Value {
        self.scroll.script_configuration()
    }
}

#[test]
fn should_move_one_notch_by_the_chromium_distance() {
    assert_eq!(WheelStepScenario::given_chromium_like_scrolling().scroll.pixels_per_notch(), 120.0);
}

#[test]
fn should_take_about_a_fifth_of_a_second_for_one_notch() {
    assert_eq!(WheelStepScenario::given_chromium_like_scrolling().scroll.duration_ms(), 180.0);
}

#[test]
fn should_reach_the_target_exactly_from_rest() {
    let moved: f64 = WheelStepScenario::given_chromium_like_scrolling().pixels_moved_per_frame_from_rest().iter().sum();
    assert!((moved - 120.0).abs() < 1e-6, "moved {moved}");
}

#[test]
fn should_start_gently_instead_of_jerking_from_rest() {
    let frames = WheelStepScenario::given_chromium_like_scrolling().pixels_moved_per_frame_from_rest();
    assert!(frames[0] <= 4.0, "first frame moved {}", frames[0]);
}

#[test]
fn should_never_move_a_frame_further_than_chromium_does() {
    let frames = WheelStepScenario::given_chromium_like_scrolling().pixels_moved_per_frame_from_rest();
    let fastest = frames.iter().copied().fold(0.0, f64::max);
    assert!(fastest <= 22.0, "fastest frame moved {fastest}");
}

#[test]
fn should_peak_around_the_middle_of_a_step_from_rest() {
    let frames = WheelStepScenario::given_chromium_like_scrolling().pixels_moved_per_frame_from_rest();
    let peak = frames
        .iter()
        .enumerate()
        .fold((0, 0.0), |best, (index, moved)| if *moved > best.1 { (index, *moved) } else { best })
        .0;
    assert!((3..=7).contains(&peak), "peak at frame {peak} of {}", frames.len());
}

#[test]
fn should_never_move_backwards() {
    let scenario = WheelStepScenario::given_chromium_like_scrolling();
    for frames in [scenario.pixels_moved_per_frame_from_rest(), scenario.pixels_moved_per_frame_while_moving()] {
        assert!(frames.iter().all(|moved| *moved >= 0.0), "{frames:?}");
    }
}

#[test]
fn should_keep_the_speed_when_another_notch_arrives_while_moving() {
    let frames = WheelStepScenario::given_chromium_like_scrolling().pixels_moved_per_frame_while_moving();
    assert!(frames[0] >= frames[frames.len() / 2], "a moving page must not slow down to start over: {frames:?}");
}

#[test]
fn should_reach_the_target_exactly_while_moving() {
    let moved: f64 =
        WheelStepScenario::given_chromium_like_scrolling().pixels_moved_per_frame_while_moving().iter().sum();
    assert!((moved - 120.0).abs() < 1e-6, "moved {moved}");
}

#[test]
fn should_hand_the_page_script_the_step_the_duration_and_both_curves() {
    let configuration = WheelStepScenario::given_chromium_like_scrolling().configuration();
    let lengths = (
        configuration["pixelsPerNotch"].clone(),
        configuration["durationMs"].clone(),
        configuration["startCurve"].as_array().map(Vec::len),
        configuration["continueCurve"].as_array().map(Vec::len),
    );
    assert_eq!(lengths, (Value::from(120.0), Value::from(180.0), Some(65), Some(65)));
}
