//! What a failed Rust test printed, reduced to what the report shows: the panic message, where it was
//! raised, and the kind of failure used to rank the most frequent ones.
//!
//! A panicking test prints `thread 'name' (pid) panicked at tests/file.rs:12:5:` and the message on the
//! next line, for example ``assertion `left == right` failed``.

const PANICKED_AT: &str = " panicked at ";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FailureMessage {
    message: String,
    location: Option<String>,
    timed_out: bool,
}

impl FailureMessage {
    /// `output` is the text of a JUnit `<failure>` element, `summary` its `message` attribute and
    /// `failure_type` its `type` attribute (nextest writes `test failure with exit code 101` or a timeout).
    pub fn from_junit_failure(output: &str, summary: &str, failure_type: &str) -> Self {
        let timed_out = failure_type.contains("timeout") || failure_type.contains("timed out");
        let lines: Vec<&str> = output.lines().map(str::trim).filter(|line| !line.is_empty()).collect();
        let panic_line = lines.iter().position(|line| line.contains(PANICKED_AT));
        let location = panic_line
            .and_then(|index| lines.get(index))
            .or_else(|| Some(&summary).filter(|text| text.contains(PANICKED_AT)))
            .and_then(|line| line.split(PANICKED_AT).nth(1))
            .map(|place| place.trim().trim_end_matches(':').to_owned());
        let message = match panic_line {
            Some(index) => lines.get(index + 1).copied().filter(|line| !line.starts_with("note: ")),
            None => lines.first().copied(),
        }
        .unwrap_or(summary)
        .to_owned();
        Self { message, location, timed_out }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    /// The failure class the "most frequent failures" ranking counts.
    pub fn kind(&self) -> &'static str {
        let message = self.message.as_str();
        if self.timed_out {
            "Test timeout"
        } else if message.starts_with("assertion `left == right` failed") {
            "assert_eq!"
        } else if message.starts_with("assertion `left != right` failed") {
            "assert_ne!"
        } else if message.starts_with("assertion failed") {
            "assert!"
        } else if message.contains("`Option::unwrap()` on a `None` value") {
            "Option::unwrap"
        } else if message.contains("`Result::unwrap()` on an `Err` value") {
            "Result::unwrap"
        } else {
            "Inny błąd"
        }
    }
}
