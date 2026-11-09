//! Reading bridge lines pasted by the user, so that only real bridges reach Tor's configuration.

use ztbrowser_core::tor::bridge_line::BridgeLine;
use ztbrowser_core::tor::bridge_line::BridgeLineProblem;
use ztbrowser_core::tor::bridge_line::InvalidBridgeLine;
use ztbrowser_core::tor::pluggable_transport::PluggableTransport;

struct PastedBridgesScenario {
    text: String,
}

impl PastedBridgesScenario {
    fn given_pasted(text: &str) -> Self {
        Self { text: text.to_owned() }
    }

    fn when_read(&self) -> BridgesOutcome {
        BridgesOutcome { parsed: BridgeLine::parse_all(&self.text) }
    }
}

struct BridgesOutcome {
    parsed: Result<Vec<BridgeLine>, InvalidBridgeLine>,
}

impl BridgesOutcome {
    fn bridges(&self) -> &[BridgeLine] {
        self.parsed.as_ref().expect("valid bridge lines")
    }

    fn then_reads_as(&self, lines: &[&str]) {
        assert_eq!(self.bridges().iter().map(BridgeLine::to_line).collect::<Vec<_>>(), lines);
    }

    fn then_needs(&self, transports: &[Option<PluggableTransport>]) {
        assert_eq!(self.bridges().iter().map(BridgeLine::transport).collect::<Vec<_>>(), transports);
    }

    fn then_is_rejected_because(&self, problem: &BridgeLineProblem) {
        assert_eq!(self.parsed.as_ref().err().map(|invalid| &invalid.problem), Some(problem));
    }
}

#[test]
fn should_read_obfs4_bridge_as_handed_out_by_the_tor_project() {
    PastedBridgesScenario::given_pasted(
        "obfs4 192.0.2.10:443 d9a82d2f9c2f65a18407b1d2b764f130847f8b5d cert=abc+/= iat-mode=0",
    )
    .when_read()
    .then_reads_as(&["obfs4 192.0.2.10:443 D9A82D2F9C2F65A18407B1D2B764F130847F8B5D cert=abc+/= iat-mode=0"]);
}

#[test]
fn should_accept_lines_copied_from_a_torrc_with_their_bridge_keyword_and_skip_blank_lines() {
    PastedBridgesScenario::given_pasted("Bridge 192.0.2.11:9001\n\n   \nbridge [2001:db8::1]:443\n")
        .when_read()
        .then_reads_as(&["192.0.2.11:9001", "[2001:db8::1]:443"]);
}

#[test]
fn should_tell_which_transport_each_bridge_needs() {
    PastedBridgesScenario::given_pasted("192.0.2.11:9001\nsnowflake 192.0.2.3:80 url=https://example.net/")
        .when_read()
        .then_needs(&[None, Some(PluggableTransport::Snowflake)]);
}

#[test]
fn should_reject_transport_it_cannot_run() {
    PastedBridgesScenario::given_pasted("scramblesuit 192.0.2.10:443")
        .when_read()
        .then_is_rejected_because(&BridgeLineProblem::UnknownTransport("scramblesuit".to_owned()));
}

#[test]
fn should_reject_bridge_named_by_host_instead_of_address() {
    PastedBridgesScenario::given_pasted("obfs4 bridge.example.net:443")
        .when_read()
        .then_is_rejected_because(&BridgeLineProblem::InvalidAddress("bridge.example.net:443".to_owned()));
}

#[test]
fn should_reject_transport_without_address() {
    PastedBridgesScenario::given_pasted("webtunnel")
        .when_read()
        .then_is_rejected_because(&BridgeLineProblem::MissingAddress);
}

#[test]
fn should_refuse_argument_that_would_start_a_comment_in_tors_configuration() {
    PastedBridgesScenario::given_pasted("obfs4 192.0.2.10:443 cert=abc#SocksPort=0.0.0.0:9050")
        .when_read()
        .then_is_rejected_because(&BridgeLineProblem::InvalidArgument("cert=abc#SocksPort=0.0.0.0:9050".to_owned()));
}

#[test]
fn should_refuse_argument_with_a_line_continuation() {
    PastedBridgesScenario::given_pasted("obfs4 192.0.2.10:443 cert=abc\\")
        .when_read()
        .then_is_rejected_because(&BridgeLineProblem::InvalidArgument("cert=abc\\".to_owned()));
}

#[test]
fn should_refuse_word_that_is_not_a_key_value_argument() {
    PastedBridgesScenario::given_pasted("obfs4 192.0.2.10:443 ExitNodes")
        .when_read()
        .then_is_rejected_because(&BridgeLineProblem::InvalidArgument("ExitNodes".to_owned()));
}
