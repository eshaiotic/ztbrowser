//! The badge in the report's title: the worst thing about the run, in this order of severity.

use crate::test_status::TestStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReportVerdict {
    Failed,
    CoverageAtOrBelowThreshold,
    Flaky,
    Passed,
}

impl ReportVerdict {
    pub fn label(self) -> &'static str {
        match self {
            ReportVerdict::Failed => "FAILED",
            ReportVerdict::CoverageAtOrBelowThreshold => "COVERAGE ≤90%",
            ReportVerdict::Flaky => "FLAKY",
            ReportVerdict::Passed => "PASSED",
        }
    }

    pub fn colour(self) -> &'static str {
        match self {
            ReportVerdict::Failed | ReportVerdict::CoverageAtOrBelowThreshold => TestStatus::Failed.colour(),
            ReportVerdict::Flaky => TestStatus::Flaky.colour(),
            ReportVerdict::Passed => TestStatus::Passed.colour(),
        }
    }
}
