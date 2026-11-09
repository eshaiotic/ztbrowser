//! The tests of one feature, a test binary such as `ztbrowser-core::generic_hiding`, added up.

use crate::milliseconds::Milliseconds;
use crate::status_counts::StatusCounts;
use crate::test_case::TestCase;

#[derive(Clone, Debug, PartialEq)]
pub struct FeatureSummary {
    name: String,
    counts: StatusCounts,
    duration: Milliseconds,
}

impl FeatureSummary {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_owned(), counts: StatusCounts::default(), duration: Milliseconds::default() }
    }

    pub fn add(&mut self, case: &TestCase) {
        self.counts.record(case.status());
        self.duration = Milliseconds::new(self.duration.value() + case.duration().value());
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn counts(&self) -> &StatusCounts {
        &self.counts
    }

    pub fn duration(&self) -> Milliseconds {
        self.duration
    }
}
