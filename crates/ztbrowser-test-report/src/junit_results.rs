//! The results of one cargo-nextest run, read from the JUnit XML file nextest writes when its profile has a
//! `junit` section (`.config/nextest.toml`, profile `report`).
//!
//! A `<testcase>` with a `<failure>` or `<error>` child failed (after its retries, which appear as
//! `<rerunFailure>`); one with a `<flakyFailure>` failed first and passed when retried; `<skipped>` marks a
//! skipped test. The time of retried attempts is added to the test's own, so slow flaky tests show up as slow.

use roxmltree::Document;
use roxmltree::Node;

use crate::failure_message::FailureMessage;
use crate::milliseconds::Milliseconds;
use crate::run_timestamp::RunTimestamp;
use crate::status_counts::StatusCounts;
use crate::test_case::TestCase;
use crate::test_status::TestStatus;

const FAILED_ELEMENTS: [&str; 2] = ["failure", "error"];
const FLAKY_ELEMENTS: [&str; 2] = ["flakyFailure", "flakyError"];
const RETRIED_ELEMENTS: [&str; 4] = ["flakyFailure", "flakyError", "rerunFailure", "rerunError"];

#[derive(Clone, Debug, PartialEq)]
pub struct JunitResults {
    started_at: RunTimestamp,
    wall_clock: Milliseconds,
    cases: Vec<TestCase>,
}

impl JunitResults {
    /// # Errors
    ///
    /// When the text is not JUnit XML from nextest: not XML, no `<testsuites>` root, or no run timestamp.
    pub fn parse(xml: &str) -> Result<Self, String> {
        let document = Document::parse(xml).map_err(|error| format!("plik JUnit nie jest poprawnym XML: {error}"))?;
        let root = document.root_element();
        if !root.has_tag_name("testsuites") {
            return Err("plik JUnit nie zaczyna się od <testsuites>".to_owned());
        }
        let started_at = root
            .attribute("timestamp")
            .and_then(RunTimestamp::parse)
            .ok_or_else(|| "plik JUnit nie podaje czasu przebiegu (<testsuites timestamp>)".to_owned())?;
        let cases = root.descendants().filter(|node| node.has_tag_name("testcase")).map(Self::test_case).collect();
        Ok(Self { started_at, wall_clock: Milliseconds::from_seconds(Self::seconds(root)), cases })
    }

    pub fn started_at(&self) -> &RunTimestamp {
        &self.started_at
    }

    pub fn wall_clock(&self) -> Milliseconds {
        self.wall_clock
    }

    pub fn cases(&self) -> &[TestCase] {
        &self.cases
    }

    pub fn counts(&self) -> StatusCounts {
        let mut counts = StatusCounts::default();
        for case in &self.cases {
            counts.record(case.status());
        }
        counts
    }

    fn test_case(node: Node<'_, '_>) -> TestCase {
        let child = |names: &[&str]| node.children().find(|child| names.contains(&child.tag_name().name()));
        let (status, failure) = if let Some(failure) = child(&FAILED_ELEMENTS) {
            (TestStatus::Failed, Some(failure))
        } else if let Some(failure) = child(&FLAKY_ELEMENTS) {
            (TestStatus::Flaky, Some(failure))
        } else if child(&["skipped"]).is_some() {
            (TestStatus::Skipped, None)
        } else {
            (TestStatus::Passed, None)
        };
        let retried: f64 = node
            .children()
            .filter(|child| RETRIED_ELEMENTS.contains(&child.tag_name().name()))
            .map(Self::seconds)
            .sum();
        let feature = node
            .attribute("classname")
            .or_else(|| node.parent_element().and_then(|suite| suite.attribute("name")))
            .unwrap_or("?");
        let case = TestCase::new(
            feature,
            node.attribute("name").unwrap_or("?"),
            status,
            Milliseconds::from_seconds(Self::seconds(node) + retried),
        );
        match failure {
            Some(failure) => case.with_failure(FailureMessage::from_junit_failure(
                &Self::own_text(failure),
                failure.attribute("message").unwrap_or_default(),
                failure.attribute("type").unwrap_or_default(),
            )),
            None => case,
        }
    }

    /// The element's own text, without the `<system-out>` / `<system-err>` children nested in it.
    fn own_text(node: Node<'_, '_>) -> String {
        node.children().filter(Node::is_text).filter_map(|text| text.text()).collect()
    }

    fn seconds(node: Node<'_, '_>) -> f64 {
        node.attribute("time").and_then(|time| time.parse::<f64>().ok()).filter(|time| time.is_finite()).unwrap_or(0.0)
    }
}
