//! The configuration file ZTBrowser writes for its private `tor`.
//!
//! Tor is started as `tor -f <data directory>/torrc --defaults-torrc /dev/null`, so neither the system's
//! `/etc/tor/torrc` nor its defaults file can change what the Tor window's Tor does. It runs as a client only,
//! picks free loopback ports for SOCKS and control itself (`auto`), writes the control port to a file
//! ZTBrowser waits for, protects the control port with a cookie file, and exits when ZTBrowser does
//! (`__OwningControllerProcess`). The data directory persists between runs: it keeps Tor's entry guards
//! (choosing new ones every run would make it likelier to meet a malicious one) and the cached directory of
//! relays, which makes later connections much faster.
//!
//! Tor's file format reads `"..."` with C escapes as one value, so paths are written quoted. Transport
//! program lines are split at spaces by Tor even inside quotes, so a program path with a space, a quote, a
//! backslash or `#` cannot be written at all and is refused.

use std::ffi::OsString;
use std::fmt;
use std::fmt::Write;
use std::path::Path;
use std::path::PathBuf;

use crate::language::message_catalogue::MessageCatalogue;
use crate::tor::bridge_choice::BridgeChoice;
use crate::tor::bridge_line::BridgeLine;
use crate::tor::pluggable_transport::PluggableTransport;
use crate::tor::tor_programs::TorPrograms;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TorConfigurationProblem {
    /// The bridges need a transport program that is not installed.
    MissingTransport(PluggableTransport),
    /// A path Tor's configuration format cannot carry.
    UnusablePath(PathBuf),
}

impl TorConfigurationProblem {
    /// What is wrong, in the interface's language.
    #[must_use]
    pub fn describe(&self, catalogue: &MessageCatalogue) -> String {
        match self {
            Self::MissingTransport(transport) => catalogue.text_with(
                "The {transport} bridges need a program that is not installed; install {package}",
                &[("transport", transport.name()), ("package", transport.package_hint(catalogue))],
            ),
            Self::UnusablePath(path) => {
                catalogue.text_with("Tor cannot use the path {path}", &[("path", &path.display().to_string())])
            }
        }
    }
}

impl fmt::Display for TorConfigurationProblem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.describe(&MessageCatalogue::untranslated()))
    }
}

impl std::error::Error for TorConfigurationProblem {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorConfiguration {
    data_directory: PathBuf,
    owner_process: u32,
    bridges: Vec<BridgeLine>,
    transport_programs: Vec<(PluggableTransport, PathBuf)>,
}

impl TorConfiguration {
    pub const TORRC_FILE: &'static str = "torrc";
    pub const CONTROL_PORT_FILE: &'static str = "control-port";
    pub const COOKIE_FILE: &'static str = "control-auth-cookie";
    const NO_DEFAULTS: &'static str = "/dev/null";

    /// A configuration for the Tor of process `owner_process`, keeping its state in `data_directory`.
    ///
    /// # Errors
    /// [`TorConfigurationProblem::MissingTransport`] when a chosen bridge needs a transport `programs` lacks;
    /// [`TorConfigurationProblem::UnusablePath`] when a transport program's path cannot be written for Tor.
    pub fn new(
        data_directory: PathBuf,
        owner_process: u32,
        bridges: &BridgeChoice,
        programs: &TorPrograms,
    ) -> Result<Self, TorConfigurationProblem> {
        let transport_programs = bridges
            .needed_transports()
            .into_iter()
            .map(|transport| {
                let program =
                    programs.program_for(transport).ok_or(TorConfigurationProblem::MissingTransport(transport))?;
                if !Self::is_bare_word(program) {
                    return Err(TorConfigurationProblem::UnusablePath(program.to_path_buf()));
                }
                Ok((transport, program.to_path_buf()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { data_directory, owner_process, bridges: bridges.bridges(), transport_programs })
    }

    #[must_use]
    pub fn data_directory(&self) -> &Path {
        &self.data_directory
    }

    #[must_use]
    pub fn torrc_file(&self) -> PathBuf {
        self.data_directory.join(Self::TORRC_FILE)
    }

    /// Where Tor writes `PORT=127.0.0.1:<port>` once its control port listens.
    #[must_use]
    pub fn control_port_file(&self) -> PathBuf {
        self.data_directory.join(Self::CONTROL_PORT_FILE)
    }

    /// The secret ZTBrowser proves it may control Tor with.
    #[must_use]
    pub fn cookie_file(&self) -> PathBuf {
        self.data_directory.join(Self::COOKIE_FILE)
    }

    /// The arguments `tor` is started with.
    #[must_use]
    pub fn command_line(&self) -> Vec<OsString> {
        vec!["-f".into(), self.torrc_file().into(), "--defaults-torrc".into(), Self::NO_DEFAULTS.into()]
    }

    /// The contents of the configuration file.
    ///
    /// # Errors
    /// [`TorConfigurationProblem::UnusablePath`] when the data directory is not valid UTF-8.
    pub fn to_torrc(&self) -> Result<String, TorConfigurationProblem> {
        let mut torrc = String::new();
        let quoted_paths = [
            ("DataDirectory", self.data_directory.clone()),
            ("ControlPortWriteToFile", self.control_port_file()),
            ("CookieAuthFile", self.cookie_file()),
        ];
        for (option, path) in quoted_paths {
            let _ = writeln!(torrc, "{option} {}", Self::quoted(&path)?);
        }
        let fixed = [
            "SocksPort auto".to_owned(),
            "ControlPort auto".to_owned(),
            "CookieAuthentication 1".to_owned(),
            format!("__OwningControllerProcess {}", self.owner_process),
            "ClientOnly 1".to_owned(),
            "AvoidDiskWrites 1".to_owned(),
            "Log notice stdout".to_owned(),
        ];
        for line in fixed {
            let _ = writeln!(torrc, "{line}");
        }
        if !self.bridges.is_empty() {
            let _ = writeln!(torrc, "UseBridges 1");
        }
        for (transport, program) in &self.transport_programs {
            let _ = writeln!(torrc, "ClientTransportPlugin {} exec {}", transport.name(), program.display());
        }
        for bridge in &self.bridges {
            let _ = writeln!(torrc, "Bridge {}", bridge.to_line());
        }
        Ok(torrc)
    }

    fn quoted(path: &Path) -> Result<String, TorConfigurationProblem> {
        let text = path.to_str().ok_or_else(|| TorConfigurationProblem::UnusablePath(path.to_path_buf()))?;
        let mut quoted = String::with_capacity(text.len() + 2);
        quoted.push('"');
        for character in text.chars() {
            match character {
                '"' => quoted.push_str("\\\""),
                '\\' => quoted.push_str("\\\\"),
                '\n' => quoted.push_str("\\n"),
                '\r' => quoted.push_str("\\r"),
                '\t' => quoted.push_str("\\t"),
                control if control.is_control() => {
                    return Err(TorConfigurationProblem::UnusablePath(path.to_path_buf()));
                }
                other => quoted.push(other),
            }
        }
        quoted.push('"');
        Ok(quoted)
    }

    fn is_bare_word(path: &Path) -> bool {
        path.to_str().is_some_and(|text| {
            !text.is_empty()
                && text.chars().all(|character| {
                    !character.is_whitespace() && !character.is_control() && !matches!(character, '"' | '\\' | '#')
                })
        })
    }
}
