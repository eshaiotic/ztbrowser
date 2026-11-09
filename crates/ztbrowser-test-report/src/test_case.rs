//! One test of a run.

use crate::failure_message::FailureMessage;
use crate::milliseconds::Milliseconds;
use crate::test_status::TestStatus;

#[derive(Clone, Debug, PartialEq)]
pub struct TestCase {
    feature: String,
    title: String,
    status: TestStatus,
    duration: Milliseconds,
    failure: Option<FailureMessage>,
}

impl TestCase {
    /// `feature` is the test binary the test belongs to (`ztbrowser-core::generic_hiding`), the report's
    /// unit of a "feature".
    pub fn new(feature: &str, title: &str, status: TestStatus, duration: Milliseconds) -> Self {
        Self { feature: feature.to_owned(), title: title.to_owned(), status, duration, failure: None }
    }

    #[must_use]
    pub fn with_failure(mut self, failure: FailureMessage) -> Self {
        self.failure = Some(failure);
        self
    }

    pub fn feature(&self) -> &str {
        &self.feature
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn status(&self) -> TestStatus {
        self.status
    }

    pub fn duration(&self) -> Milliseconds {
        self.duration
    }

    pub fn failure(&self) -> Option<&FailureMessage> {
        self.failure.as_ref()
    }

    /// Where the test failed (`tests/x.rs:12:5`), else the test binary it belongs to.
    pub fn place(&self) -> &str {
        self.failure.as_ref().and_then(FailureMessage::location).unwrap_or(&self.feature)
    }
}
