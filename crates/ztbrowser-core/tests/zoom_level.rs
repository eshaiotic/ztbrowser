//! The sizes a page can be drawn at, and how zoom commands step between them.

use ztbrowser_core::page_zoom::zoom_level::ZoomLevel;

struct ZoomScenario {
    level: ZoomLevel,
}

impl ZoomScenario {
    fn given_page_at_actual_size() -> Self {
        Self { level: ZoomLevel::ACTUAL_SIZE }
    }

    fn given_page_at(percent: u64) -> Self {
        Self { level: ZoomLevel::from_percent(percent).expect("the scenario starts at a zoom step") }
    }

    fn given_webkit_reports_factor(factor: f64) -> Self {
        Self { level: ZoomLevel::nearest_to_factor(factor) }
    }

    fn when_zoomed_in(mut self, times: usize) -> Self {
        for _ in 0..times {
            self.level = self.level.zoomed_in();
        }
        self
    }

    fn when_zoomed_out(mut self, times: usize) -> Self {
        for _ in 0..times {
            self.level = self.level.zoomed_out();
        }
        self
    }

    fn then_page_is_drawn_at(&self, expected: &str) {
        assert_eq!(self.level.to_string(), expected);
    }

    fn then_cannot_zoom_in_further(&self) {
        assert!(!self.level.can_zoom_in());
    }

    fn then_cannot_zoom_out_further(&self) {
        assert!(!self.level.can_zoom_out());
    }
}

#[test]
fn should_step_in_from_actual_size_to_the_next_size() {
    ZoomScenario::given_page_at_actual_size().when_zoomed_in(1).then_page_is_drawn_at("115%");
}

#[test]
fn should_step_out_from_actual_size_to_the_next_size() {
    ZoomScenario::given_page_at_actual_size().when_zoomed_out(1).then_page_is_drawn_at("85%");
}

#[test]
fn should_take_coarser_steps_far_from_actual_size() {
    ZoomScenario::given_page_at(200).when_zoomed_in(1).then_page_is_drawn_at("250%");
}

#[test]
fn should_stop_at_the_largest_size() {
    ZoomScenario::given_page_at(250).when_zoomed_in(3).then_cannot_zoom_in_further();
}

#[test]
fn should_stop_at_the_smallest_size() {
    ZoomScenario::given_page_at(75).when_zoomed_out(3).then_cannot_zoom_out_further();
}

#[test]
fn should_return_exactly_to_the_starting_size_after_zooming_in_and_out() {
    ZoomScenario::given_page_at(85).when_zoomed_in(4).when_zoomed_out(4).then_page_is_drawn_at("85%");
}

#[test]
fn should_read_webkits_factor_as_the_nearest_size() {
    ZoomScenario::given_webkit_reports_factor(1.149_999).then_page_is_drawn_at("115%");
}

#[test]
fn should_continue_from_the_nearest_size_when_webkit_reports_an_odd_factor() {
    ZoomScenario::given_webkit_reports_factor(1.1).when_zoomed_in(1).then_page_is_drawn_at("125%");
}

#[test]
fn should_accept_only_its_own_sizes_from_storage() {
    assert_eq!(ZoomLevel::from_percent(110), None);
}
