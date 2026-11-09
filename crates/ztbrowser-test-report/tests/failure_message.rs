//! What the report keeps of a failed test's output: the panic message, its location and its kind.

use ztbrowser_test_report::failure_message::FailureMessage;

struct FailureScenario {
    output: String,
    summary: String,
    failure_type: String,
}

impl FailureScenario {
    fn given_panic_output(output: &str) -> Self {
        Self {
            output: output.to_owned(),
            summary: String::new(),
            failure_type: "test failure with exit code 101".to_owned(),
        }
    }

    fn with_failure_type(mut self, failure_type: &str) -> Self {
        failure_type.clone_into(&mut self.failure_type);
        self
    }

    fn with_summary(mut self, summary: &str) -> Self {
        summary.clone_into(&mut self.summary);
        self
    }

    fn when_read(&self) -> FailureOutcome {
        FailureOutcome { failure: FailureMessage::from_junit_failure(&self.output, &self.summary, &self.failure_type) }
    }
}

struct FailureOutcome {
    failure: FailureMessage,
}

impl FailureOutcome {
    fn then_message_is(&self, expected: &str) {
        assert_eq!(self.failure.message(), expected);
    }

    fn then_location_is(&self, expected: &str) {
        assert_eq!(self.failure.location(), Some(expected));
    }

    fn then_kind_is(&self, expected: &str) {
        assert_eq!(self.failure.kind(), expected);
    }
}

const ASSERT_EQ_PANIC: &str = "thread 'tests::sum' (2819270) panicked at tests/sum.rs:6:28:\n\
assertion `left == right` failed: sum must be three\n  left: 2\n right: 3\n\
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace";

#[test]
fn should_keep_the_line_after_the_panic_location_as_the_message() {
    FailureScenario::given_panic_output(ASSERT_EQ_PANIC)
        .when_read()
        .then_message_is("assertion `left == right` failed: sum must be three");
}

#[test]
fn should_keep_where_the_test_panicked() {
    FailureScenario::given_panic_output(ASSERT_EQ_PANIC).when_read().then_location_is("tests/sum.rs:6:28");
}

#[test]
fn should_classify_a_failed_assert_eq() {
    FailureScenario::given_panic_output(ASSERT_EQ_PANIC).when_read().then_kind_is("assert_eq!");
}

#[test]
fn should_classify_unwrap_on_none() {
    FailureScenario::given_panic_output(
        "thread 't' panicked at src/a.rs:1:1:\ncalled `Option::unwrap()` on a `None` value",
    )
    .when_read()
    .then_kind_is("Option::unwrap");
}

#[test]
fn should_classify_a_timeout_by_the_failure_type() {
    FailureScenario::given_panic_output("").with_failure_type("test timeout").when_read().then_kind_is("Test timeout");
}

#[test]
fn should_call_an_explicit_panic_another_error() {
    FailureScenario::given_panic_output("thread 't' panicked at src/a.rs:1:1:\nfirst attempt fails")
        .when_read()
        .then_kind_is("Inny błąd");
}

#[test]
fn should_fall_back_to_the_summary_when_the_output_is_empty() {
    FailureScenario::given_panic_output("")
        .with_summary("thread 't' panicked at src/a.rs:9:2")
        .when_read()
        .then_location_is("src/a.rs:9:2");
}
