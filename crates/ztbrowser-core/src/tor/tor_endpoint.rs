//! A local address of ZTBrowser's own Tor: its control port or its SOCKS port.
//!
//! Tor picks both ports itself (`auto`), so no other program can take them first, and reports them: the
//! control port in the file named by `ControlPortWriteToFile` (`PORT=127.0.0.1:41923`), the SOCKS port in the
//! answer to `GETINFO net/listeners/socks` (`"127.0.0.1:41924"`). Only loopback addresses are accepted: the
//! Tor window's traffic must never be handed to a proxy somewhere else on the network.

use std::fmt;
use std::net::SocketAddr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TorEndpoint {
    address: SocketAddr,
}

impl TorEndpoint {
    const PORT_FILE_KEY: &'static str = "PORT=";

    /// The control port named in Tor's port file, when it is a loopback address.
    #[must_use]
    pub fn from_control_port_file(text: &str) -> Option<Self> {
        text.lines().find_map(|line| line.trim().strip_prefix(Self::PORT_FILE_KEY)).and_then(Self::parse)
    }

    /// The first loopback listener in the value of `GETINFO net/listeners/socks` (quoted, space-separated).
    #[must_use]
    pub fn from_listeners(value: &str) -> Option<Self> {
        value.split_whitespace().map(|listener| listener.trim_matches('"')).find_map(Self::parse)
    }

    #[must_use]
    pub fn socket_address(self) -> SocketAddr {
        self.address
    }

    /// `socks5://identity3:tor@127.0.0.1:41924` for `identity` 3. `GLib`'s SOCKS5 client hands host names to
    /// Tor, so Tor resolves every name and the local DNS resolver never learns which sites the Tor window
    /// visits. Tor puts streams sent with different SOCKS credentials on different circuits
    /// (`IsolateSOCKSAuth`, on by default), so a new identity never shares a circuit with the one before it.
    #[must_use]
    pub fn socks_proxy_uri(self, identity: u32) -> String {
        format!("socks5://identity{identity}:tor@{}", self.address)
    }

    fn parse(text: &str) -> Option<Self> {
        let address = text.parse::<SocketAddr>().ok()?;
        (address.ip().is_loopback() && address.port() != 0).then_some(Self { address })
    }
}

impl fmt::Display for TorEndpoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.address.fmt(formatter)
    }
}
