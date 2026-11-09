//! Everything the report page shows about the latest run, worked out from the run's results, its coverage
//! and the run history: totals, features, slowest tests, failure rankings, durations and the verdict.

use crate::coverage_summary::CoverageSummary;
use crate::feature_summary::FeatureSummary;
use crate::junit_results::JunitResults;
use crate::milliseconds::Milliseconds;
use crate::report_verdict::ReportVerdict;
use crate::run_history::RunHistory;
use crate::status_counts::StatusCounts;
use crate::test_case::TestCase;
use crate::test_status::TestStatus;

/// Where the report says the code ran, and when it was generated.
pub struct RunContext {
    pub branch: String,
    pub commit: String,
    pub now_unix_seconds: i64,
}

/// Total, average and longest time of the tests that ran.
pub struct TestDurations {
    pub total: Milliseconds,
    pub average: Milliseconds,
    pub longest: Milliseconds,
}

pub struct TestRunReport<'run> {
    results: &'run JunitResults,
    coverage: Option<&'run CoverageSummary>,
    history: &'run RunHistory,
    context: &'run RunContext,
}

impl<'run> TestRunReport<'run> {
    /// The suite every nextest run is recorded under; ZTBrowser has one kind of test.
    pub const SUITE: &'static str = "UNIT";
    /// How many recent runs a card shows.
    pub const RUN_WINDOW: usize = 20;
    /// How many tests the "slowest tests" table lists.
    pub const SLOWEST_LIMIT: usize = 10;

    pub fn new(
        results: &'run JunitResults,
        coverage: Option<&'run CoverageSummary>,
        history: &'run RunHistory,
        context: &'run RunContext,
    ) -> Self {
        Self { results, coverage, history, context }
    }

    pub fn results(&self) -> &'run JunitResults {
        self.results
    }

    pub fn coverage(&self) -> Option<&'run CoverageSummary> {
        self.coverage
    }

    pub fn history(&self) -> &'run RunHistory {
        self.history
    }

    pub fn context(&self) -> &'run RunContext {
        self.context
    }

    pub fn totals(&self) -> StatusCounts {
        self.results.counts()
    }

    /// Features with failures first, then flaky ones, then by name.
    pub fn features(&self) -> Vec<FeatureSummary> {
        let mut features: Vec<FeatureSummary> = Vec::new();
        for case in self.results.cases() {
            if !features.iter().any(|feature| feature.name() == case.feature()) {
                features.push(FeatureSummary::new(case.feature()));
            }
            if let Some(feature) = features.iter_mut().find(|feature| feature.name() == case.feature()) {
                feature.add(case);
            }
        }
        features.sort_by(|left, right| {
            let (left_counts, right_counts) = (left.counts(), right.counts());
            right_counts
                .of(TestStatus::Failed)
                .cmp(&left_counts.of(TestStatus::Failed))
                .then(right_counts.of(TestStatus::Flaky).cmp(&left_counts.of(TestStatus::Flaky)))
                .then_with(|| left.name().cmp(right.name()))
        });
        features
    }

    pub fn cases_with_status(&self, status: TestStatus) -> Vec<&'run TestCase> {
        self.results.cases().iter().filter(|case| case.status() == status).collect()
    }

    pub fn slowest(&self) -> Vec<&'run TestCase> {
        let mut cases: Vec<&TestCase> = self.results.cases().iter().collect();
        cases.sort_by(|left, right| right.duration().value().total_cmp(&left.duration().value()));
        cases.truncate(Self::SLOWEST_LIMIT);
        cases
    }

    /// Failure kinds of the failed and flaky tests, most frequent first.
    pub fn frequent_failures(&self) -> Vec<(String, u32)> {
        let kinds = self
            .results
            .cases()
            .iter()
            .filter(|case| matches!(case.status(), TestStatus::Failed | TestStatus::Flaky))
            .map(|case| case.failure().map_or("Inny błąd", |failure| failure.kind()).to_owned());
        Self::ranked(kinds)
    }

    /// Features by number of failed and flaky tests, most unstable first; stable features are left out.
    pub fn unstable_features(&self) -> Vec<(String, u32)> {
        let mut ranking: Vec<(String, u32)> = self
            .features()
            .iter()
            .map(|feature| {
                let counts = feature.counts();
                (feature.name().to_owned(), counts.of(TestStatus::Failed) + counts.of(TestStatus::Flaky))
            })
            .filter(|(_, count)| *count > 0)
            .collect();
        ranking.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        ranking
    }

    pub fn durations(&self) -> TestDurations {
        let executed: Vec<f64> = self
            .results
            .cases()
            .iter()
            .filter(|case| case.status() != TestStatus::Skipped)
            .map(|case| case.duration().value())
            .collect();
        let total: f64 = executed.iter().sum();
        let count = u32::try_from(executed.len()).unwrap_or(u32::MAX);
        TestDurations {
            total: Milliseconds::new(total),
            average: Milliseconds::new(if count == 0 { 0.0 } else { total / f64::from(count) }),
            longest: Milliseconds::new(executed.iter().copied().fold(0.0, f64::max)),
        }
    }

    pub fn verdict(&self) -> ReportVerdict {
        let totals = self.totals();
        if totals.of(TestStatus::Failed) > 0 {
            ReportVerdict::Failed
        } else if self.coverage.is_some_and(|coverage| !coverage.is_above_threshold()) {
            ReportVerdict::CoverageAtOrBelowThreshold
        } else if totals.of(TestStatus::Flaky) > 0 {
            ReportVerdict::Flaky
        } else {
            ReportVerdict::Passed
        }
    }

    fn ranked(labels: impl Iterator<Item = String>) -> Vec<(String, u32)> {
        let mut ranking: Vec<(String, u32)> = Vec::new();
        for label in labels {
            match ranking.iter_mut().find(|(known, _)| *known == label) {
                Some((_, count)) => *count += 1,
                None => ranking.push((label, 1)),
            }
        }
        ranking.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
        ranking
    }
}
