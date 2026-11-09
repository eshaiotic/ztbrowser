//! The verdict on one recorded run, as the coloured bar in a card's strip of recent runs shows it.

use crate::test_status::TestStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunVerdict {
    Passing,
    Failing,
    Flaky,
}

impl RunVerdict {
    pub fn label(self) -> &'static str {
        match self {
            RunVerdict::Passing => "PASSING",
            RunVerdict::Failing => "FAILING",
            RunVerdict::Flaky => "FLAKY",
        }
    }

    pub fn colour(self) -> &'static str {
        match self {
            RunVerdict::Passing => TestStatus::Passed.colour(),
            RunVerdict::Failing => TestStatus::Failed.colour(),
            RunVerdict::Flaky => TestStatus::Flaky.colour(),
        }
    }
}
