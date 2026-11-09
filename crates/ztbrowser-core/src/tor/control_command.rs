//! The commands ZTBrowser sends to its Tor over the control port.
//!
//! Only what the Tor window needs: prove it may control this Tor (`AUTHENTICATE` with the cookie Tor wrote
//! into ZTBrowser's own directory), make Tor quit when ZTBrowser does (`TAKEOWNERSHIP`), follow the connection
//! progress (`SETEVENTS STATUS_CLIENT`, `GETINFO status/bootstrap-phase`), learn the SOCKS port (`GETINFO
//! net/listeners/socks`) and ask for new circuits (`SIGNAL NEWNYM`, the "New Identity" of Tor Browser).

use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControlCommand {
    Authenticate { cookie: Vec<u8> },
    TakeOwnership,
    FollowStatus,
    GetBootstrapPhase,
    GetSocksListeners,
    NewIdentity,
}

impl ControlCommand {
    pub const BOOTSTRAP_PHASE_KEY: &'static str = "status/bootstrap-phase";
    pub const SOCKS_LISTENERS_KEY: &'static str = "net/listeners/socks";
    /// Tor's authentication cookie is always exactly this long.
    pub const COOKIE_LENGTH: usize = 32;

    /// `AUTHENTICATE` with the cookie read from Tor's cookie file; `None` when the file is not a Tor cookie.
    #[must_use]
    pub fn authenticate(cookie: &[u8]) -> Option<Self> {
        (cookie.len() == Self::COOKIE_LENGTH).then(|| Self::Authenticate { cookie: cookie.to_vec() })
    }

    /// The command as sent, with its `CRLF` line ending.
    #[must_use]
    pub fn to_line(&self) -> String {
        match self {
            Self::Authenticate { cookie } => {
                let mut line = String::from("AUTHENTICATE ");
                for byte in cookie {
                    let _ = write!(line, "{byte:02X}");
                }
                line + "\r\n"
            }
            Self::TakeOwnership => "TAKEOWNERSHIP\r\n".to_owned(),
            Self::FollowStatus => "SETEVENTS STATUS_CLIENT\r\n".to_owned(),
            Self::GetBootstrapPhase => format!("GETINFO {}\r\n", Self::BOOTSTRAP_PHASE_KEY),
            Self::GetSocksListeners => format!("GETINFO {}\r\n", Self::SOCKS_LISTENERS_KEY),
            Self::NewIdentity => "SIGNAL NEWNYM\r\n".to_owned(),
        }
    }
}
