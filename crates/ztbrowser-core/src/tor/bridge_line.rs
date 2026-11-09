//! One bridge, as `bridges.torproject.org` hands it out: `obfs4 192.0.2.1:443 <fingerprint> cert=... iat-mode=0`.
//!
//! A line names its transport (none for a plain bridge), the bridge's address, usually the bridge's
//! fingerprint (40 hexadecimal digits) and the transport's `key=value` arguments. Users paste these lines,
//! and they end up in the configuration file ZTBrowser writes for Tor, so a line is accepted only when every
//! part has exactly that shape: a line break, `#` (a comment in Tor's file) or a backslash (a line
//! continuation) could otherwise smuggle options of its own into Tor's configuration.

use crate::language::message_catalogue::MessageCatalogue;
use std::fmt;
use std::net::SocketAddr;

use crate::tor::pluggable_transport::PluggableTransport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeLineProblem {
    UnknownTransport(String),
    MissingAddress,
    InvalidAddress(String),
    InvalidArgument(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidBridgeLine {
    pub line: String,
    pub problem: BridgeLineProblem,
}

impl InvalidBridgeLine {
    /// What is wrong with the line, in the interface's language.
    #[must_use]
    pub fn describe(&self, catalogue: &MessageCatalogue) -> String {
        match &self.problem {
            BridgeLineProblem::UnknownTransport(name) => {
                catalogue.text_with("Unknown bridge transport: {name}", &[("name", name)])
            }
            BridgeLineProblem::MissingAddress => {
                catalogue.text_with("The bridge line has no address: {line}", &[("line", &self.line)])
            }
            BridgeLineProblem::InvalidAddress(text) => {
                catalogue.text_with("{text} is not an IP address and port", &[("text", text)])
            }
            BridgeLineProblem::InvalidArgument(text) => {
                catalogue.text_with("{text} is not a bridge argument", &[("text", text)])
            }
        }
    }
}

impl fmt::Display for InvalidBridgeLine {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.describe(&MessageCatalogue::untranslated()))
    }
}

impl std::error::Error for InvalidBridgeLine {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeLine {
    transport: Option<PluggableTransport>,
    address: SocketAddr,
    fingerprint: Option<String>,
    arguments: Vec<String>,
}

impl BridgeLine {
    const TORRC_KEYWORD: &'static str = "Bridge";
    const FINGERPRINT_LENGTH: usize = 40;

    /// # Errors
    /// [`InvalidBridgeLine`] when the line is not a bridge line in the shape described above.
    pub fn parse(line: &str) -> Result<Self, InvalidBridgeLine> {
        let invalid = |problem| InvalidBridgeLine { line: line.to_owned(), problem };
        let mut words = line.split_whitespace().peekable();
        words.next_if(|word| word.eq_ignore_ascii_case(Self::TORRC_KEYWORD));
        let first = words.next().ok_or_else(|| invalid(BridgeLineProblem::MissingAddress))?;
        let (transport, address) = if let Ok(address) = first.parse::<SocketAddr>() {
            (None, address)
        } else {
            let transport = PluggableTransport::from_name(first)
                .ok_or_else(|| invalid(BridgeLineProblem::UnknownTransport(first.to_owned())))?;
            let address_text = words.next().ok_or_else(|| invalid(BridgeLineProblem::MissingAddress))?;
            let address = address_text
                .parse::<SocketAddr>()
                .map_err(|_| invalid(BridgeLineProblem::InvalidAddress(address_text.to_owned())))?;
            (Some(transport), address)
        };
        let fingerprint = words.next_if(|word| Self::is_fingerprint(word)).map(str::to_ascii_uppercase);
        let arguments: Vec<String> = words.map(str::to_owned).collect();
        if let Some(bad) = arguments.iter().find(|argument| !Self::is_argument(argument)) {
            return Err(invalid(BridgeLineProblem::InvalidArgument(bad.clone())));
        }
        Ok(Self { transport, address, fingerprint, arguments })
    }

    /// Every bridge in pasted text, one per line; blank lines are skipped.
    ///
    /// # Errors
    /// The first line that is not a valid bridge line.
    pub fn parse_all(text: &str) -> Result<Vec<Self>, InvalidBridgeLine> {
        text.lines().filter(|line| !line.trim().is_empty()).map(Self::parse).collect()
    }

    /// The transport the bridge needs; `None` for a plain bridge.
    #[must_use]
    pub fn transport(&self) -> Option<PluggableTransport> {
        self.transport
    }

    /// The line in its canonical form, as written after `Bridge` in Tor's configuration and as stored.
    #[must_use]
    pub fn to_line(&self) -> String {
        let parts = self.transport.map(|transport| transport.name().to_owned()).into_iter();
        let parts = parts.chain([self.address.to_string()]).chain(self.fingerprint.clone());
        parts.chain(self.arguments.iter().cloned()).collect::<Vec<String>>().join(" ")
    }

    fn is_fingerprint(word: &str) -> bool {
        word.len() == Self::FINGERPRINT_LENGTH && word.bytes().all(|byte| byte.is_ascii_hexdigit())
    }

    fn is_argument(word: &str) -> bool {
        let Some((key, _)) = word.split_once('=') else { return false };
        !key.is_empty() && word.bytes().all(|byte| byte.is_ascii_graphic() && !matches!(byte, b'#' | b'\\' | b'"'))
    }
}
