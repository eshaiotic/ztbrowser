//! What the report works out about the latest run: verdict, features, rankings and durations.

use ztbrowser_test_report::coverage_summary::CoverageSummary;
use ztbrowser_test_report::junit_results::JunitResults;
use ztbrowser_test_report::report_verdict::ReportVerdict;
use ztbrowser_test_report::run_history::RunHistory;
use ztbrowser_test_report::test_run_report::RunContext;
use ztbrowser_test_report::test_run_report::TestRunReport;

struct ReportScenario {
    cases: Vec<String>,
    coverage: Option<CoverageSummary>,
    history: RunHistory,
    context: RunContext,
}

impl ReportScenario {
    fn given_run() -> Self {
        Self {
            cases: Vec::new(),
            coverage: None,
            history: RunHistory::default(),
            context: RunContext { branch: "main".to_owned(), commit: "abc1234".to_owned(), now_unix_seconds: 0 },
        }
    }

    fn with_test(mut self, feature: &str, name: &str, seconds: f64, outcome: &str) -> Self {
        let body = match outcome {
            "failed" => {
                "<failure message=\"m\" type=\"t\">thread 'x' panicked at tests/a.rs:1:1:\nassertion failed: ok</failure>"
            }
            "flaky" => {
                "<flakyFailure message=\"m\" type=\"t\">thread 'x' panicked at tests/a.rs:1:1:\nfirst attempt fails</flakyFailure>"
            }
            "skipped" => "<skipped/>",
            _ => "",
        };
        self.cases.push(format!(r#"<testcase name="{name}" classname="{feature}" time="{seconds}">{body}</testcase>"#));
        self
    }

    fn with_many_passing_tests(mut self, count: usize) -> Self {
        for index in 0..count {
            self = self.with_test(
                "core::zoom",
                &format!("test_{index:02}"),
                f64::from(u32::try_from(index).unwrap_or(0)) / 1000.0,
                "passed",
            );
        }
        self
    }

    fn with_line_coverage(mut self, covered: u32, count: u32) -> Self {
        let totals = serde_json::json!({"lines": {"covered": covered, "count": count}});
        self.coverage = CoverageSummary::from_totals(&totals);
        self
    }

    fn when_reported(&self) -> ReportOutcome {
        let xml = format!(
            r#"<testsuites timestamp="2026-11-03T09:14:04+01:00" time="1"><testsuite name="all">{}</testsuite></testsuites>"#,
            self.cases.concat()
        );
        let results = JunitResults::parse(&xml).expect("scenario XML should parse");
        let report = TestRunReport::new(&results, self.coverage.as_ref(), &self.history, &self.context);
        ReportOutcome {
            verdict: report.verdict(),
            feature_order: report.features().iter().map(|feature| feature.name().to_owned()).collect(),
            frequent_failures: report.frequent_failures(),
            slowest: report.slowest().iter().map(|case| case.title().to_owned()).collect(),
            average_milliseconds: report.durations().average.value(),
        }
    }
}

struct ReportOutcome {
    verdict: ReportVerdict,
    feature_order: Vec<String>,
    frequent_failures: Vec<(String, u32)>,
    slowest: Vec<String>,
    average_milliseconds: f64,
}

impl ReportOutcome {
    fn then_verdict_is(&self, expected: ReportVerdict) {
        assert_eq!(self.verdict, expected);
    }

    fn then_features_are_ordered(&self, expected: &[&str]) {
        assert_eq!(self.feature_order, expected);
    }

    fn then_most_frequent_failure_is(&self, kind: &str, count: u32) {
        assert_eq!(self.frequent_failures.first(), Some(&(kind.to_owned(), count)));
    }

    fn then_slowest_list_has(&self, count: usize, first: &str) {
        assert_eq!((self.slowest.len(), self.slowest.first().map(String::as_str)), (count, Some(first)));
    }

    fn then_average_test_took(&self, expected: f64) {
        assert!((self.average_milliseconds - expected).abs() < 1e-9, "average {}", self.average_milliseconds);
    }
}

#[test]
fn should_fail_the_report_when_a_test_failed_even_with_low_coverage() {
    ReportScenario::given_run()
        .with_test("core::a", "broken", 0.001, "failed")
        .with_line_coverage(50, 100)
        .when_reported()
        .then_verdict_is(ReportVerdict::Failed);
}

#[test]
fn should_flag_coverage_at_ninety_percent_before_flaky_tests() {
    ReportScenario::given_run()
        .with_test("core::a", "unstable", 0.001, "flaky")
        .with_line_coverage(90, 100)
        .when_reported()
        .then_verdict_is(ReportVerdict::CoverageAtOrBelowThreshold);
}

#[test]
fn should_call_a_run_with_a_flaky_test_flaky() {
    ReportScenario::given_run()
        .with_test("core::a", "unstable", 0.001, "flaky")
        .when_reported()
        .then_verdict_is(ReportVerdict::Flaky);
}

#[test]
fn should_pass_a_green_run_without_coverage() {
    ReportScenario::given_run()
        .with_test("core::a", "fine", 0.001, "passed")
        .when_reported()
        .then_verdict_is(ReportVerdict::Passed);
}

#[test]
fn should_list_failing_features_before_flaky_ones_and_the_rest_by_name() {
    ReportScenario::given_run()
        .with_test("core::b", "fine", 0.001, "passed")
        .with_test("core::z", "unstable", 0.001, "flaky")
        .with_test("core::a", "fine", 0.001, "passed")
        .with_test("core::y", "broken", 0.001, "failed")
        .when_reported()
        .then_features_are_ordered(&["core::y", "core::z", "core::a", "core::b"]);
}

#[test]
fn should_rank_failure_kinds_by_frequency() {
    ReportScenario::given_run()
        .with_test("core::a", "first", 0.001, "failed")
        .with_test("core::a", "second", 0.001, "failed")
        .with_test("core::a", "third", 0.001, "flaky")
        .when_reported()
        .then_most_frequent_failure_is("assert!", 2);
}

#[test]
fn should_list_the_ten_slowest_tests_slowest_first() {
    ReportScenario::given_run().with_many_passing_tests(15).when_reported().then_slowest_list_has(10, "test_14");
}

#[test]
fn should_leave_skipped_tests_out_of_the_average_time() {
    ReportScenario::given_run()
        .with_test("core::a", "fast", 0.002, "passed")
        .with_test("core::a", "slow", 0.004, "passed")
        .with_test("core::a", "ignored", 0.0, "skipped")
        .when_reported()
        .then_average_test_took(3.0);
}
