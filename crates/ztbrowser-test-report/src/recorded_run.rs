//! One suite of one past run, as the run history keeps it: when it started, on which commit, how long it
//! took, how its tests ended and, when measured, its coverage.

use serde_json::Value;
use serde_json::json;

use crate::coverage_summary::CoverageSummary;
use crate::milliseconds::Milliseconds;
use crate::run_timestamp::RunTimestamp;
use crate::status_counts::StatusCounts;
use crate::test_status::TestStatus;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordedRun {
    suite: String,
    started_at: RunTimestamp,
    commit: String,
    duration: Milliseconds,
    counts: StatusCounts,
    coverage: Option<CoverageSummary>,
}

impl RecordedRun {
    pub fn new(
        suite: &str,
        started_at: RunTimestamp,
        commit: &str,
        duration: Milliseconds,
        counts: StatusCounts,
    ) -> Self {
        Self { suite: suite.to_owned(), started_at, commit: commit.to_owned(), duration, counts, coverage: None }
    }

    #[must_use]
    pub fn with_coverage(mut self, coverage: Option<CoverageSummary>) -> Self {
        self.coverage = coverage;
        self
    }

    pub fn suite(&self) -> &str {
        &self.suite
    }

    pub fn started_at(&self) -> &RunTimestamp {
        &self.started_at
    }

    pub fn commit(&self) -> &str {
        &self.commit
    }

    pub fn duration(&self) -> Milliseconds {
        self.duration
    }

    pub fn counts(&self) -> &StatusCounts {
        &self.counts
    }

    pub fn coverage(&self) -> Option<&CoverageSummary> {
        self.coverage.as_ref()
    }

    /// The same suite started at the same moment is the same run, however often the report is regenerated.
    pub fn is_same_run_as(&self, other: &RecordedRun) -> bool {
        self.suite == other.suite && self.started_at == other.started_at
    }

    pub fn to_json(&self) -> Value {
        let stats: serde_json::Map<String, Value> =
            TestStatus::ALL.iter().map(|status| (status.key().to_owned(), json!(self.counts.of(*status)))).collect();
        json!({
            "suite": self.suite,
            "startedAt": self.started_at.as_str(),
            "commit": self.commit,
            "durationMs": self.duration.value(),
            "stats": stats,
            "coverage": self.coverage.as_ref().map(CoverageSummary::to_totals),
        })
    }

    /// `None` for an entry that does not have the shape `to_json` writes.
    pub fn from_json(entry: &Value) -> Option<Self> {
        let stats = entry.get("stats")?;
        let count = |status: TestStatus| -> Option<u32> { u32::try_from(stats.get(status.key())?.as_u64()?).ok() };
        let counts = StatusCounts::new(
            count(TestStatus::Passed)?,
            count(TestStatus::Failed)?,
            count(TestStatus::Flaky)?,
            count(TestStatus::Skipped)?,
        );
        let run = Self::new(
            entry.get("suite")?.as_str()?,
            RunTimestamp::parse(entry.get("startedAt")?.as_str()?)?,
            entry.get("commit")?.as_str()?,
            Milliseconds::new(entry.get("durationMs")?.as_f64()?),
            counts,
        );
        Some(run.with_coverage(entry.get("coverage").and_then(CoverageSummary::from_totals)))
    }
}
