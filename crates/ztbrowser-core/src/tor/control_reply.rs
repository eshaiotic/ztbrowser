//! Replies and events on Tor's control port, assembled from the lines Tor sends.
//!
//! Every line starts with a three-digit status and a separator: `250-` (more lines follow), `250+` (a data
//! block follows, ended by a line holding only `.`), `250 ` (the reply's last line). `2xx` is success, `5xx` a
//! refusal, and `650` an asynchronous event, which Tor sends between replies, never inside one.

use std::fmt;

/// A line that is not part of Tor's control protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MalformedReply {
    pub line: String,
}

impl fmt::Display for MalformedReply {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "not a Tor control reply: {:?}", self.line)
    }
}

impl std::error::Error for MalformedReply {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlReply {
    status: u16,
    lines: Vec<String>,
}

impl ControlReply {
    const EVENT_STATUS: u16 = 650;
    const STATUS_EVENT_PREFIX: &'static str = "STATUS_CLIENT ";

    #[must_use]
    pub fn status(&self) -> u16 {
        self.status
    }

    #[must_use]
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// An asynchronous event rather than the answer to a command.
    #[must_use]
    pub fn is_event(&self) -> bool {
        self.status == Self::EVENT_STATUS
    }

    /// The first line's text: `OK`, or the reason of a refusal (`Authentication failed: ...`).
    #[must_use]
    pub fn message(&self) -> &str {
        self.lines.first().map_or("", String::as_str)
    }

    /// The value Tor gave for `key` in a `GETINFO` answer (`key=value`).
    #[must_use]
    pub fn value(&self, key: &str) -> Option<&str> {
        self.lines.iter().find_map(|line| line.strip_prefix(key)?.strip_prefix('='))
    }

    /// The report of a `STATUS_CLIENT` event (`NOTICE BOOTSTRAP PROGRESS=...`).
    #[must_use]
    pub fn status_report(&self) -> Option<&str> {
        if !self.is_event() {
            return None;
        }
        self.message().strip_prefix(Self::STATUS_EVENT_PREFIX)
    }
}

/// Collects lines into replies.
#[derive(Debug, Default)]
pub struct ControlReplyReader {
    status: Option<u16>,
    lines: Vec<String>,
    /// The line whose data block is being read, with the data read so far.
    data: Option<String>,
}

impl ControlReplyReader {
    const DATA_END: &'static str = ".";

    /// Takes one line (without its line ending); returns the reply it completes, if any.
    ///
    /// # Errors
    /// [`MalformedReply`] for a line that is not a control reply line, or whose status differs from the
    /// lines before it in the same reply.
    pub fn push_line(&mut self, line: &str) -> Result<Option<ControlReply>, MalformedReply> {
        if line == Self::DATA_END && self.data.is_some() {
            let finished = self.data.take();
            self.lines.extend(finished);
            return Ok(None);
        }
        if let Some(data) = &mut self.data {
            data.push('\n');
            data.push_str(line.strip_prefix('.').filter(|_| line.starts_with("..")).unwrap_or(line));
            return Ok(None);
        }
        let malformed = || MalformedReply { line: line.to_owned() };
        let (code, rest) = line.split_at_checked(3).ok_or_else(malformed)?;
        let status = code.parse::<u16>().ok().filter(|_| code.bytes().all(|byte| byte.is_ascii_digit()));
        let status = status.ok_or_else(malformed)?;
        if self.status.is_some_and(|current| current != status) {
            return Err(malformed());
        }
        self.status = Some(status);
        let mut separator_and_text = rest.chars();
        let separator = separator_and_text.next().ok_or_else(malformed)?;
        let text = separator_and_text.as_str().to_owned();
        match separator {
            '-' => self.lines.push(text),
            '+' => self.data = Some(text),
            ' ' => {
                self.lines.push(text);
                self.status = None;
                return Ok(Some(ControlReply { status, lines: std::mem::take(&mut self.lines) }));
            }
            _ => return Err(malformed()),
        }
        Ok(None)
    }
}
