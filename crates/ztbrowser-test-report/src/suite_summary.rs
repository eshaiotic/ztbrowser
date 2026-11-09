//! A suite's recent history, as one card of the report shows it: pass rate over the last runs, a sparkline,
//! a strip of coloured bars, and the latest run in the footer.

use crate::percent::Percent;
use crate::recorded_run::RecordedRun;
use crate::run_verdict::RunVerdict;
use crate::test_status::TestStatus;

pub struct SuiteSummary<'history> {
    total_runs: usize,
    recent: Vec<&'history RecordedRun>,
}

impl<'history> SuiteSummary<'history> {
    /// `None` without any run: a card cannot summarise nothing.
    pub fn new(total_runs: usize, recent: Vec<&'history RecordedRun>) -> Option<Self> {
        (!recent.is_empty()).then_some(Self { total_runs, recent })
    }

    pub fn total_runs(&self) -> usize {
        self.total_runs
    }

    pub fn recent(&self) -> &[&'history RecordedRun] {
        &self.recent
    }

    pub fn latest(&self) -> &'history RecordedRun {
        self.recent[self.recent.len() - 1]
    }

    /// Executed tests of the recent runs that passed (flaky counted as passed), rounded to one decimal.
    pub fn pass_rate(&self) -> Percent {
        let counts = || self.recent.iter().map(|run| run.counts());
        let executed: u32 = counts().map(|counts| counts.total() - counts.of(TestStatus::Skipped)).sum();
        let succeeded: u32 = counts().map(|counts| counts.of(TestStatus::Passed) + counts.of(TestStatus::Flaky)).sum();
        if executed == 0 {
            return Percent::from_value(0.0);
        }
        Percent::from_value((f64::from(succeeded) / f64::from(executed) * 1000.0).round() / 10.0)
    }

    pub fn verdict(&self) -> RunVerdict {
        self.latest().counts().verdict()
    }
}
