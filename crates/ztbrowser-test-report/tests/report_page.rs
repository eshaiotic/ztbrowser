//! The rendered report page.

use ztbrowser_test_report::junit_results::JunitResults;
use ztbrowser_test_report::report_page::ReportPage;
use ztbrowser_test_report::run_history::RunHistory;
use ztbrowser_test_report::test_run_report::RunContext;
use ztbrowser_test_report::test_run_report::TestRunReport;

struct PageScenario {
    testcases: String,
    branch: String,
}

impl PageScenario {
    fn given_passing_run() -> Self {
        Self {
            testcases: r#"<testcase name="should_work" classname="core::zoom" time="0.002"></testcase>"#.to_owned(),
            branch: "main".to_owned(),
        }
    }

    fn given_failing_run_with_message(message: &str) -> Self {
        Self {
            testcases: format!(
                r#"<testcase name="should_work" classname="core::zoom" time="0.002"><failure message="m" type="t">thread 'x' panicked at tests/zoom.rs:3:5:
{message}</failure></testcase>"#
            ),
            branch: "main".to_owned(),
        }
    }

    fn with_branch(mut self, branch: &str) -> Self {
        branch.clone_into(&mut self.branch);
        self
    }

    fn when_rendered(&self) -> PageOutcome {
        let xml = format!(
            r#"<testsuites timestamp="2026-11-03T09:14:04+01:00" time="0.01"><testsuite name="core::zoom">{}</testsuite></testsuites>"#,
            self.testcases
        );
        let results = JunitResults::parse(&xml).expect("scenario XML should parse");
        let history = RunHistory::default();
        let context = RunContext { branch: self.branch.clone(), commit: "abc1234".to_owned(), now_unix_seconds: 0 };
        let report = TestRunReport::new(&results, None, &history, &context);
        let page = ReportPage::new(&report);
        PageOutcome { file_name: page.file_name(), html: page.render() }
    }
}

struct PageOutcome {
    file_name: String,
    html: String,
}

impl PageOutcome {
    fn then_file_name_is(&self, expected: &str) {
        assert_eq!(self.file_name, expected);
    }

    fn then_contains(&self, expected: &str) {
        assert!(self.html.contains(expected), "page should contain {expected}");
    }

    fn then_does_not_contain(&self, unexpected: &str) {
        assert!(!self.html.contains(unexpected), "page should not contain {unexpected}");
    }
}

#[test]
fn should_name_the_file_after_the_run() {
    PageScenario::given_passing_run().when_rendered().then_file_name_is("test-report-20261103-0914.html");
}

#[test]
fn should_badge_a_green_run_as_passed() {
    PageScenario::given_passing_run().when_rendered().then_contains("</i>PASSED</span>");
}

#[test]
fn should_hide_the_failed_tests_table_when_nothing_failed() {
    PageScenario::given_passing_run().when_rendered().then_does_not_contain("Testy nieudane");
}

#[test]
fn should_show_a_failed_test_with_its_message() {
    PageScenario::given_failing_run_with_message("assertion failed: zoom.is_valid()")
        .when_rendered()
        .then_contains("assertion failed: zoom.is_valid()");
}

#[test]
fn should_escape_markup_in_failure_messages() {
    PageScenario::given_failing_run_with_message("expected <script>alert(1)</script>")
        .when_rendered()
        .then_does_not_contain("<script>alert(1)</script>");
}

#[test]
fn should_escape_markup_in_the_branch_name() {
    PageScenario::given_passing_run()
        .with_branch("feature/<b>bold</b>")
        .when_rendered()
        .then_contains("feature/&lt;b&gt;bold&lt;/b&gt;");
}

#[test]
fn should_say_coverage_was_not_measured_without_a_summary() {
    PageScenario::given_passing_run().when_rendered().then_contains("brak pomiaru w tym przebiegu");
}
