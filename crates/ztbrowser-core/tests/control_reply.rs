//! Assembling the replies and events Tor sends on its control port.

use ztbrowser_core::tor::control_reply::ControlReply;
use ztbrowser_core::tor::control_reply::ControlReplyReader;
use ztbrowser_core::tor::control_reply::MalformedReply;

struct ControlPortScenario {
    lines: Vec<String>,
}

impl ControlPortScenario {
    fn given_tor_sends(lines: &[&str]) -> Self {
        Self { lines: lines.iter().map(|line| (*line).to_owned()).collect() }
    }

    fn when_read(&self) -> ControlPortOutcome {
        let mut reader = ControlReplyReader::default();
        let mut replies = Vec::new();
        for line in &self.lines {
            match reader.push_line(line) {
                Ok(reply) => replies.extend(reply),
                Err(malformed) => return ControlPortOutcome { replies, error: Some(malformed) },
            }
        }
        ControlPortOutcome { replies, error: None }
    }
}

struct ControlPortOutcome {
    replies: Vec<ControlReply>,
    error: Option<MalformedReply>,
}

impl ControlPortOutcome {
    fn only_reply(&self) -> &ControlReply {
        assert_eq!(self.replies.len(), 1, "expected exactly one reply in {:?}", self.replies);
        &self.replies[0]
    }

    fn then_succeeds_with(&self, key: &str, value: &str) {
        let reply = self.only_reply();
        assert_eq!((reply.is_success(), reply.value(key)), (true, Some(value)));
    }

    fn then_is_refused_with(&self, status: u16, message: &str) {
        let reply = self.only_reply();
        assert_eq!((reply.is_success(), reply.status(), reply.message()), (false, status, message));
    }

    fn then_reports_status(&self, report: &str) {
        assert_eq!(self.only_reply().status_report(), Some(report));
    }

    fn then_is_malformed(&self) {
        assert!(self.error.is_some(), "expected a malformed line in {:?}", self.replies);
    }
}

#[test]
fn should_read_value_from_multi_line_getinfo_reply() {
    ControlPortScenario::given_tor_sends(&["250-net/listeners/socks=\"127.0.0.1:41924\"", "250 OK"])
        .when_read()
        .then_succeeds_with("net/listeners/socks", "\"127.0.0.1:41924\"");
}

#[test]
fn should_read_data_block_with_dot_escaped_lines() {
    ControlPortScenario::given_tor_sends(&["250+config-text=", "SocksPort auto", "..hidden", ".", "250 OK"])
        .when_read()
        .then_succeeds_with("config-text", "\nSocksPort auto\n.hidden");
}

#[test]
fn should_report_refusal_with_its_reason() {
    ControlPortScenario::given_tor_sends(&["515 Authentication failed: Wrong length on authentication cookie."])
        .when_read()
        .then_is_refused_with(515, "Authentication failed: Wrong length on authentication cookie.");
}

#[test]
fn should_read_client_status_event() {
    ControlPortScenario::given_tor_sends(&["650 STATUS_CLIENT NOTICE BOOTSTRAP PROGRESS=75 TAG=enough_dirinfo"])
        .when_read()
        .then_reports_status("NOTICE BOOTSTRAP PROGRESS=75 TAG=enough_dirinfo");
}

#[test]
fn should_reject_line_that_is_not_part_of_the_protocol() {
    ControlPortScenario::given_tor_sends(&["HTTP/1.1 200 OK"]).when_read().then_is_malformed();
}

#[test]
fn should_reject_reply_whose_status_changes_midway() {
    ControlPortScenario::given_tor_sends(&["250-version=0.4.9", "550 OK"]).when_read().then_is_malformed();
}
