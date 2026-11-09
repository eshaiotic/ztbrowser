//! The history of test runs the report draws its cards from, kept in `test-results/test-run-history.json`
//! (ignored by git). Every report generation records the current run; regenerating the report for the
//! same run does not add it twice. At most 200 runs per suite are kept.

use std::collections::HashMap;

use serde_json::Value;

use crate::recorded_run::RecordedRun;
use crate::suite_summary::SuiteSummary;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunHistory {
    runs: Vec<RecordedRun>,
}

impl RunHistory {
    pub const LIMIT_PER_SUITE: usize = 200;

    /// Reads a saved history. A missing, damaged or foreign file starts an empty history instead of
    /// stopping the report; entries that cannot be read are dropped.
    pub fn from_json(text: &str) -> Self {
        let runs = serde_json::from_str::<Value>(text)
            .ok()
            .and_then(|value| {
                value.as_array().map(|entries| entries.iter().filter_map(RecordedRun::from_json).collect())
            })
            .unwrap_or_default();
        let mut history = Self { runs };
        history.order_and_trim();
        history
    }

    pub fn to_json(&self) -> String {
        let entries: Vec<Value> = self.runs.iter().map(RecordedRun::to_json).collect();
        format!("{:#}\n", Value::Array(entries))
    }

    pub fn record(&mut self, run: RecordedRun) {
        self.runs.retain(|recorded| !recorded.is_same_run_as(&run));
        self.runs.push(run);
        self.order_and_trim();
    }

    pub fn len(&self) -> usize {
        self.runs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }

    pub fn runs(&self) -> &[RecordedRun] {
        &self.runs
    }

    /// The suite's last `window` runs, oldest first.
    pub fn suite_summary(&self, suite: &str, window: usize) -> Option<SuiteSummary<'_>> {
        let runs: Vec<&RecordedRun> = self.runs.iter().filter(|run| run.suite() == suite).collect();
        let recent = runs[runs.len().saturating_sub(window)..].to_vec();
        SuiteSummary::new(runs.len(), recent)
    }

    /// The last `window` runs that measured coverage, oldest first.
    pub fn coverage_runs(&self, window: usize) -> Vec<&RecordedRun> {
        let runs: Vec<&RecordedRun> = self.runs.iter().filter(|run| run.coverage().is_some()).collect();
        runs[runs.len().saturating_sub(window)..].to_vec()
    }

    /// Oldest first, and only the newest `LIMIT_PER_SUITE` runs of each suite.
    fn order_and_trim(&mut self) {
        self.runs.sort_by_key(|run| run.started_at().unix_seconds());
        let mut kept_per_suite: HashMap<String, usize> = HashMap::new();
        let mut newest_first: Vec<RecordedRun> = Vec::new();
        for run in self.runs.drain(..).rev() {
            let kept = kept_per_suite.entry(run.suite().to_owned()).or_default();
            *kept += 1;
            if *kept <= Self::LIMIT_PER_SUITE {
                newest_first.push(run);
            }
        }
        newest_first.reverse();
        self.runs = newest_first;
    }
}
