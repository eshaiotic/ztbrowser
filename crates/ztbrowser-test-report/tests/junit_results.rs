//! Reading cargo-nextest's JUnit file: statuses, durations with retries, and where tests failed.

use ztbrowser_test_report::junit_results::JunitResults;
use ztbrowser_test_report::test_status::TestStatus;

struct JunitScenario {
    xml: String,
}

impl JunitScenario {
    /// A run like the one nextest writes with `retries = 1`: one test passes, one is flaky, one fails twice.
    fn given_run_with_a_passing_a_flaky_and_a_failing_test() -> Self {
        Self {
            xml: r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="nextest-run" tests="3" failures="1" errors="0" timestamp="2026-11-03T09:14:23.416+01:00" time="0.009">
  <testsuite name="probe" tests="3" disabled="0" errors="0" failures="1">
    <testcase name="tests::passes" classname="probe" time="0.003"></testcase>
    <testcase name="tests::flaky_once" classname="probe" time="0.003">
      <flakyFailure time="0.004" message="thread &apos;tests::flaky_once&apos; panicked at src/lib.rs:10:69" type="test failure with exit code 101">thread 'tests::flaky_once' panicked at src/lib.rs:10:69:
first attempt fails
        <system-out>running 1 test</system-out>
      </flakyFailure>
    </testcase>
    <testcase name="tests::fails" classname="probe" time="0.004">
      <failure message="thread &apos;tests::fails&apos; panicked at src/lib.rs:6:28" type="test failure with exit code 101">thread 'tests::fails' panicked at src/lib.rs:6:28:
assertion `left == right` failed</failure>
      <rerunFailure time="0.005" message="m" type="test failure with exit code 101">again</rerunFailure>
    </testcase>
  </testsuite>
</testsuites>"#
                .to_owned(),
        }
    }

    fn given_xml(xml: &str) -> Self {
        Self { xml: xml.to_owned() }
    }

    fn when_parsed(&self) -> JunitOutcome {
        JunitOutcome { results: JunitResults::parse(&self.xml) }
    }
}

struct JunitOutcome {
    results: Result<JunitResults, String>,
}

impl JunitOutcome {
    fn parsed(&self) -> &JunitResults {
        self.results.as_ref().expect("the JUnit file should parse")
    }

    fn then_counts_are(&self, passed: u32, failed: u32, flaky: u32) {
        let counts = self.parsed().counts();
        assert_eq!(
            (counts.of(TestStatus::Passed), counts.of(TestStatus::Failed), counts.of(TestStatus::Flaky)),
            (passed, failed, flaky)
        );
    }

    fn then_test_took_milliseconds(&self, title: &str, expected: f64) {
        let case = self.parsed().cases().iter().find(|case| case.title() == title).expect("test should be listed");
        assert!((case.duration().value() - expected).abs() < 1e-9, "{title} took {}", case.duration().value());
    }

    fn then_test_failed_at(&self, title: &str, expected: &str) {
        let case = self.parsed().cases().iter().find(|case| case.title() == title).expect("test should be listed");
        assert_eq!(case.place(), expected);
    }

    fn then_feature_of_every_test_is(&self, expected: &str) {
        assert!(self.parsed().cases().iter().all(|case| case.feature() == expected));
    }

    fn then_run_took_milliseconds(&self, expected: f64) {
        assert!((self.parsed().wall_clock().value() - expected).abs() < 1e-9);
    }

    fn then_is_rejected(&self) {
        assert!(self.results.is_err());
    }
}

#[test]
fn should_count_passed_failed_and_flaky_tests() {
    JunitScenario::given_run_with_a_passing_a_flaky_and_a_failing_test().when_parsed().then_counts_are(1, 1, 1);
}

#[test]
fn should_add_the_time_of_retried_attempts_to_a_flaky_test() {
    JunitScenario::given_run_with_a_passing_a_flaky_and_a_failing_test()
        .when_parsed()
        .then_test_took_milliseconds("tests::flaky_once", 7.0);
}

#[test]
fn should_add_the_time_of_reruns_to_a_failed_test() {
    JunitScenario::given_run_with_a_passing_a_flaky_and_a_failing_test()
        .when_parsed()
        .then_test_took_milliseconds("tests::fails", 9.0);
}

#[test]
fn should_keep_where_a_failed_test_panicked() {
    JunitScenario::given_run_with_a_passing_a_flaky_and_a_failing_test()
        .when_parsed()
        .then_test_failed_at("tests::fails", "src/lib.rs:6:28");
}

#[test]
fn should_group_tests_by_their_test_binary() {
    JunitScenario::given_run_with_a_passing_a_flaky_and_a_failing_test()
        .when_parsed()
        .then_feature_of_every_test_is("probe");
}

#[test]
fn should_take_the_run_time_from_the_root_element() {
    JunitScenario::given_run_with_a_passing_a_flaky_and_a_failing_test().when_parsed().then_run_took_milliseconds(9.0);
}

#[test]
fn should_reject_text_that_is_not_xml() {
    JunitScenario::given_xml("not xml").when_parsed().then_is_rejected();
}

#[test]
fn should_reject_a_run_without_a_timestamp() {
    JunitScenario::given_xml(r#"<testsuites time="1"></testsuites>"#).when_parsed().then_is_rejected();
}
