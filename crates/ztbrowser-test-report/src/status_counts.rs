//! How many tests of a run (or of one feature) ended in each status.

use crate::percent::Percent;
use crate::run_verdict::RunVerdict;
use crate::test_status::TestStatus;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StatusCounts {
    passed: u32,
    failed: u32,
    flaky: u32,
    skipped: u32,
}

impl StatusCounts {
    pub fn new(passed: u32, failed: u32, flaky: u32, skipped: u32) -> Self {
        Self { passed, failed, flaky, skipped }
    }

    pub fn record(&mut self, status: TestStatus) {
        *self.slot(status) += 1;
    }

    pub fn of(&self, status: TestStatus) -> u32 {
        match status {
            TestStatus::Passed => self.passed,
            TestStatus::Failed => self.failed,
            TestStatus::Flaky => self.flaky,
            TestStatus::Skipped => self.skipped,
        }
    }

    pub fn total(&self) -> u32 {
        self.passed + self.failed + self.flaky + self.skipped
    }

    /// Executed tests that passed, flaky ones counted as passed; skipped tests are left out of the whole.
    pub fn pass_rate(&self) -> Percent {
        Percent::of(self.passed + self.flaky, self.total() - self.skipped)
    }

    pub fn share_of(&self, status: TestStatus) -> Percent {
        Percent::of(self.of(status), self.total())
    }

    pub fn verdict(&self) -> RunVerdict {
        if self.failed > 0 {
            RunVerdict::Failing
        } else if self.flaky > 0 {
            RunVerdict::Flaky
        } else {
            RunVerdict::Passing
        }
    }

    fn slot(&mut self, status: TestStatus) -> &mut u32 {
        match status {
            TestStatus::Passed => &mut self.passed,
            TestStatus::Failed => &mut self.failed,
            TestStatus::Flaky => &mut self.flaky,
            TestStatus::Skipped => &mut self.skipped,
        }
    }
}
