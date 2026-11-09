//! Reading cargo-llvm-cov's summary and judging it against the 90 % threshold.

use ztbrowser_test_report::coverage_summary::CoverageMetric;
use ztbrowser_test_report::coverage_summary::CoverageSummary;

struct CoverageScenario {
    export: String,
}

impl CoverageScenario {
    fn given_coverage(lines: (u32, u32), functions: (u32, u32), regions: (u32, u32)) -> Self {
        Self {
            export: format!(
                r#"{{"data":[{{"totals":{{"branches":{{"count":0,"covered":0,"percent":0}},"functions":{{"count":{},"covered":{}}},"lines":{{"count":{},"covered":{}}},"regions":{{"count":{},"covered":{}}}}}}}],"type":"llvm.coverage.json.export"}}"#,
                functions.1, functions.0, lines.1, lines.0, regions.1, regions.0
            ),
        }
    }

    fn given_export(export: &str) -> Self {
        Self { export: export.to_owned() }
    }

    fn when_read(&self) -> CoverageOutcome {
        CoverageOutcome { summary: CoverageSummary::from_llvm_cov_export(&self.export) }
    }
}

struct CoverageOutcome {
    summary: Result<CoverageSummary, String>,
}

impl CoverageOutcome {
    fn summary(&self) -> &CoverageSummary {
        self.summary.as_ref().expect("the export should parse")
    }

    fn then_metrics_are(&self, expected: &[&str]) {
        let names: Vec<&str> = self.summary().metrics().iter().map(CoverageMetric::name).collect();
        assert_eq!(names, expected);
    }

    fn then_lines_are(&self, expected: &str) {
        assert_eq!(self.summary().lines().map(|lines| lines.percent().to_string()).as_deref(), Some(expected));
    }

    fn then_is_above_threshold(&self, expected: bool) {
        assert_eq!(self.summary().is_above_threshold(), expected);
    }

    fn then_survives_the_history(&self) {
        let summary = self.summary();
        assert_eq!(CoverageSummary::from_totals(&summary.to_totals()).as_ref(), Some(summary));
    }

    fn then_is_rejected(&self) {
        assert!(self.summary.is_err());
    }
}

#[test]
fn should_leave_out_branch_coverage_that_was_not_measured() {
    CoverageScenario::given_coverage((95, 100), (95, 100), (95, 100)).when_read().then_metrics_are(&[
        "Lines",
        "Functions",
        "Regions",
    ]);
}

#[test]
fn should_truncate_the_line_share_to_two_decimals() {
    CoverageScenario::given_coverage((2, 3), (1, 1), (1, 1)).when_read().then_lines_are("66.66%");
}

#[test]
fn should_pass_when_every_measured_metric_is_above_ninety_percent() {
    CoverageScenario::given_coverage((91, 100), (95, 100), (99, 100)).when_read().then_is_above_threshold(true);
}

#[test]
fn should_fail_when_one_metric_is_at_ninety_percent() {
    CoverageScenario::given_coverage((95, 100), (95, 100), (90, 100)).when_read().then_is_above_threshold(false);
}

#[test]
fn should_keep_the_same_numbers_in_the_run_history() {
    CoverageScenario::given_coverage((8433, 9120), (760, 812), (10655, 12001)).when_read().then_survives_the_history();
}

#[test]
fn should_reject_an_export_without_totals() {
    CoverageScenario::given_export(r#"{"data":[]}"#).when_read().then_is_rejected();
}
