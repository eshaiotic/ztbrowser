//! How percentages and durations are printed in the report.

use ztbrowser_test_report::milliseconds::Milliseconds;
use ztbrowser_test_report::percent::Percent;

struct PrintedValueScenario {
    printed: String,
}

impl PrintedValueScenario {
    fn given_share(part: u32, whole: u32) -> Self {
        Self { printed: Percent::of(part, whole).to_string() }
    }

    fn given_duration(milliseconds: f64) -> Self {
        Self { printed: Milliseconds::new(milliseconds).to_string() }
    }

    fn when_printed(self) -> PrintedValueOutcome {
        PrintedValueOutcome { printed: self.printed }
    }
}

struct PrintedValueOutcome {
    printed: String,
}

impl PrintedValueOutcome {
    fn then_reads(&self, expected: &str) {
        assert_eq!(self.printed, expected);
    }
}

#[test]
fn should_never_round_a_share_up_to_one_hundred_percent() {
    PrintedValueScenario::given_share(99_999, 100_000).when_printed().then_reads("99.99%");
}

#[test]
fn should_print_an_empty_whole_as_zero_percent() {
    PrintedValueScenario::given_share(0, 0).when_printed().then_reads("0.00%");
}

#[test]
fn should_print_short_durations_in_milliseconds() {
    PrintedValueScenario::given_duration(850.4).when_printed().then_reads("850 ms");
}

#[test]
fn should_print_seconds_with_one_decimal() {
    PrintedValueScenario::given_duration(2626.0).when_printed().then_reads("2.6s");
}

#[test]
fn should_print_minutes_and_seconds() {
    PrintedValueScenario::given_duration(184_000.0).when_printed().then_reads("3m 4s");
}

#[test]
fn should_print_hours_and_minutes() {
    PrintedValueScenario::given_duration(3_720_000.0).when_printed().then_reads("1h 2m");
}

#[test]
fn should_print_a_missing_duration_as_a_dash() {
    PrintedValueScenario::given_duration(0.0).when_printed().then_reads("—");
}
