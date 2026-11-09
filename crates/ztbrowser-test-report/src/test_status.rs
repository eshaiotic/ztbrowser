//! The four classes the report sorts every test into.

/// Outcome of one test in a run.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TestStatus {
    Passed,
    Failed,
    /// Failed, then passed when nextest retried it. Still a defect to fix: the report shows it in yellow.
    Flaky,
    Skipped,
}

impl TestStatus {
    pub const ALL: [TestStatus; 4] = [TestStatus::Passed, TestStatus::Failed, TestStatus::Flaky, TestStatus::Skipped];

    pub fn key(self) -> &'static str {
        match self {
            TestStatus::Passed => "passed",
            TestStatus::Failed => "failed",
            TestStatus::Flaky => "flaky",
            TestStatus::Skipped => "skipped",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            TestStatus::Passed => "PASSED",
            TestStatus::Failed => "FAILED",
            TestStatus::Flaky => "FLAKY",
            TestStatus::Skipped => "SKIPPED",
        }
    }

    pub fn colour(self) -> &'static str {
        match self {
            TestStatus::Passed => "#34a853",
            TestStatus::Failed => "#e5534b",
            TestStatus::Flaky => "#d4a72c",
            TestStatus::Skipped => "#6e7681",
        }
    }
}
