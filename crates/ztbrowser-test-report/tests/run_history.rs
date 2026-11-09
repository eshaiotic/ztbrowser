//! The history of runs behind the report's cards.

use ztbrowser_test_report::milliseconds::Milliseconds;
use ztbrowser_test_report::recorded_run::RecordedRun;
use ztbrowser_test_report::run_history::RunHistory;
use ztbrowser_test_report::run_timestamp::RunTimestamp;
use ztbrowser_test_report::status_counts::StatusCounts;

struct HistoryScenario {
    history: RunHistory,
}

impl HistoryScenario {
    fn given_empty_history() -> Self {
        Self { history: RunHistory::default() }
    }

    fn given_saved_history(text: &str) -> Self {
        Self { history: RunHistory::from_json(text) }
    }

    fn run_at_minute(minute: u32, counts: StatusCounts) -> RecordedRun {
        let started_at = RunTimestamp::parse(&format!("2026-11-03T{:02}:{:02}:00+01:00", minute / 60, minute % 60))
            .expect("valid timestamp");
        RecordedRun::new("UNIT", started_at, "abc1234", Milliseconds::new(2600.0), counts)
    }

    fn with_runs(mut self, count: u32, counts: StatusCounts) -> Self {
        for minute in 0..count {
            self.history.record(Self::run_at_minute(minute, counts));
        }
        self
    }

    fn when_run_is_recorded_again(mut self, minute: u32) -> HistoryOutcome {
        self.history.record(Self::run_at_minute(minute, StatusCounts::new(1, 0, 0, 0)));
        HistoryOutcome { history: self.history }
    }

    fn when_saved_and_loaded(self) -> HistoryOutcome {
        HistoryOutcome { history: RunHistory::from_json(&self.history.to_json()) }
    }

    fn when_read(self) -> HistoryOutcome {
        HistoryOutcome { history: self.history }
    }
}

struct HistoryOutcome {
    history: RunHistory,
}

impl HistoryOutcome {
    fn then_holds_runs(&self, expected: usize) {
        assert_eq!(self.history.len(), expected);
    }

    fn then_oldest_kept_run_started_at(&self, expected: &str) {
        assert_eq!(self.history.runs().first().map(|run| run.started_at().as_str()), Some(expected));
    }

    fn then_card_shows_runs(&self, expected: usize) {
        let summary = self.history.suite_summary("UNIT", 20).expect("suite should have runs");
        assert_eq!(summary.recent().len(), expected);
    }

    fn then_card_pass_rate_is(&self, expected: &str) {
        let summary = self.history.suite_summary("UNIT", 20).expect("suite should have runs");
        assert_eq!(summary.pass_rate().to_string(), expected);
    }

    fn then_first_run_counts_are(&self, expected: StatusCounts) {
        assert_eq!(self.history.runs().first().map(|run| *run.counts()), Some(expected));
    }
}

#[test]
fn should_not_add_the_same_run_twice_when_the_report_is_regenerated() {
    HistoryScenario::given_empty_history()
        .with_runs(3, StatusCounts::new(10, 0, 0, 0))
        .when_run_is_recorded_again(1)
        .then_holds_runs(3);
}

#[test]
fn should_keep_only_the_newest_two_hundred_runs_of_a_suite() {
    HistoryScenario::given_empty_history()
        .with_runs(205, StatusCounts::new(1, 0, 0, 0))
        .when_read()
        .then_oldest_kept_run_started_at("2026-11-03T00:05:00+01:00");
}

#[test]
fn should_show_the_last_twenty_runs_on_a_card() {
    HistoryScenario::given_empty_history()
        .with_runs(25, StatusCounts::new(1, 0, 0, 0))
        .when_read()
        .then_card_shows_runs(20);
}

#[test]
fn should_count_flaky_tests_as_passed_and_leave_skipped_ones_out_of_the_pass_rate() {
    HistoryScenario::given_empty_history()
        .with_runs(2, StatusCounts::new(6, 1, 2, 5))
        .when_read()
        .then_card_pass_rate_is("88.90%");
}

#[test]
fn should_keep_the_counts_when_saved_and_loaded() {
    HistoryScenario::given_empty_history()
        .with_runs(1, StatusCounts::new(300, 2, 1, 1))
        .when_saved_and_loaded()
        .then_first_run_counts_are(StatusCounts::new(300, 2, 1, 1));
}

#[test]
fn should_start_empty_from_a_damaged_file() {
    HistoryScenario::given_saved_history("{ not json").when_read().then_holds_runs(0);
}
