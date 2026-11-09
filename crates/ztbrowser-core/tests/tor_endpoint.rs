//! Learning where the Tor window's Tor listens, and trusting only loopback addresses.

use ztbrowser_core::tor::tor_endpoint::TorEndpoint;

struct TorAnnouncementScenario {
    text: String,
}

impl TorAnnouncementScenario {
    fn given_text(text: &str) -> Self {
        Self { text: text.to_owned() }
    }

    fn when_read_as_control_port_file(&self) -> EndpointOutcome {
        EndpointOutcome { endpoint: TorEndpoint::from_control_port_file(&self.text) }
    }

    fn when_read_as_socks_listeners(&self) -> EndpointOutcome {
        EndpointOutcome { endpoint: TorEndpoint::from_listeners(&self.text) }
    }
}

struct EndpointOutcome {
    endpoint: Option<TorEndpoint>,
}

impl EndpointOutcome {
    fn then_proxies_identity_through(&self, identity: u32, uri: &str) {
        assert_eq!(self.endpoint.map(|endpoint| endpoint.socks_proxy_uri(identity)).as_deref(), Some(uri));
    }

    fn then_is_at(&self, address: &str) {
        assert_eq!(self.endpoint.map(|endpoint| endpoint.to_string()).as_deref(), Some(address));
    }

    fn then_is_not_trusted(&self) {
        assert_eq!(self.endpoint, None);
    }
}

#[test]
fn should_read_control_port_tor_wrote_to_its_file() {
    TorAnnouncementScenario::given_text("PORT=127.0.0.1:39871\n")
        .when_read_as_control_port_file()
        .then_is_at("127.0.0.1:39871");
}

#[test]
fn should_proxy_each_identity_through_first_socks_listener_with_its_own_credentials() {
    TorAnnouncementScenario::given_text("\"127.0.0.1:41924\" \"[::1]:41925\"")
        .when_read_as_socks_listeners()
        .then_proxies_identity_through(3, "socks5://identity3:tor@127.0.0.1:41924");
}

#[test]
fn should_not_trust_a_listener_reachable_from_other_machines() {
    TorAnnouncementScenario::given_text("\"0.0.0.0:9050\"").when_read_as_socks_listeners().then_is_not_trusted();
}

#[test]
fn should_not_trust_a_half_written_control_port_file() {
    TorAnnouncementScenario::given_text("PORT=127.0.0.1:").when_read_as_control_port_file().then_is_not_trusted();
}
