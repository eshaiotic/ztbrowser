//! Pluggable transports: programs that disguise the connection to a Tor bridge where Tor itself is blocked.
//!
//! A bridge is an unlisted Tor relay. On networks that block Tor, the connection to it is additionally
//! disguised by a transport: `obfs4` makes it look like random bytes, `webtunnel` like ordinary HTTPS,
//! `snowflake` sends it through volunteers' browsers over WebRTC, `meek_lite` hides it behind a large
//! content delivery network. Tor starts the transport program and talks to it locally.

use crate::language::message_catalogue::MessageCatalogue;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PluggableTransport {
    Obfs4,
    WebTunnel,
    Snowflake,
    MeekLite,
}

impl PluggableTransport {
    pub const ALL: [Self; 4] = [Self::Obfs4, Self::WebTunnel, Self::Snowflake, Self::MeekLite];

    /// The transport named at the start of a bridge line (`obfs4`).
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|transport| transport.name() == name)
    }

    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Obfs4 => "obfs4",
            Self::WebTunnel => "webtunnel",
            Self::Snowflake => "snowflake",
            Self::MeekLite => "meek_lite",
        }
    }

    /// Programs that provide this transport, best first. `lyrebird` is the Tor Project's current program for
    /// most transports; distributions often still ship its predecessor `obfs4proxy` (no `webtunnel`) and
    /// Snowflake's own `snowflake-client`.
    #[must_use]
    pub fn program_names(self) -> &'static [&'static str] {
        match self {
            Self::Obfs4 | Self::MeekLite => &["lyrebird", "obfs4proxy"],
            Self::WebTunnel => &["lyrebird"],
            Self::Snowflake => &["snowflake-client", "lyrebird"],
        }
    }

    /// The package to install for this transport, as shown when the program is missing.
    #[must_use]
    pub fn package_hint(self, catalogue: &MessageCatalogue) -> &str {
        match self {
            Self::Obfs4 | Self::MeekLite => {
                catalogue.text("obfs4proxy (Debian, Ubuntu), obfs4 (Fedora) or obfs4proxy from the AUR (Arch)")
            }
            Self::WebTunnel => "lyrebird",
            Self::Snowflake => {
                catalogue.text("snowflake-client (Debian, Ubuntu) or snowflake-pt-client from the AUR (Arch)")
            }
        }
    }
}
