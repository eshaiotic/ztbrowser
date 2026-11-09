//! The command line of one ZTBrowser process.
//!
//! `ztbrowser [--profile <id>] [address...]` opens a browser window in a profile; `ztbrowser --web-app <id>`
//! starts one installed web app (that is what its `.desktop` file runs); `ztbrowser --tor [address...]` opens a
//! Tor window, whose session lives in memory only and reaches the network through ZTBrowser's own Tor. The
//! target decides the GTK application id, and with it the process: a second launch with the same target is
//! handed to the process that is already running instead of starting another one.

use std::fmt;

use crate::identity::application_id::ApplicationId;
use crate::profile::profile_id::ProfileId;
use crate::web_app::desktop_entry::DesktopEntry;
use crate::web_app::web_app_id::WebAppId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchRequestError {
    UnknownSwitch(String),
    MissingValue(String),
    InvalidIdentifier(String),
    /// `app_*` profiles belong to installed web apps and only open through `--web-app`.
    ReservedProfile(String),
    /// A web app opens only its own start address, in its own profile; a Tor window has no profile.
    ConflictingTargets,
}

impl fmt::Display for LaunchRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSwitch(switch) => write!(formatter, "unknown option {switch}"),
            Self::MissingValue(switch) => write!(formatter, "option {switch} needs a value"),
            Self::InvalidIdentifier(text) => write!(formatter, "{text:?} is not a valid profile or web app identifier"),
            Self::ReservedProfile(text) => write!(formatter, "profile {text} belongs to a web app; use --web-app"),
            Self::ConflictingTargets => formatter
                .write_str("--web-app takes no other option or address, and --tor cannot be combined with --profile"),
        }
    }
}

impl std::error::Error for LaunchRequestError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LaunchTarget {
    Browser(ProfileId),
    WebApp(WebAppId),
    TorWindow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchRequest {
    target: LaunchTarget,
    addresses: Vec<String>,
}

impl LaunchRequest {
    pub const PROFILE_SWITCH: &'static str = "--profile";
    pub const TOR_SWITCH: &'static str = "--tor";

    /// Parses the arguments after the program name.
    ///
    /// # Errors
    /// [`LaunchRequestError`] for unknown options, missing or unsafe identifiers and conflicting targets.
    pub fn parse(arguments: &[String]) -> Result<Self, LaunchRequestError> {
        let mut profile = None;
        let mut web_app = None;
        let mut tor_window = false;
        let mut addresses = Vec::new();
        let mut remaining = arguments.iter();
        while let Some(argument) = remaining.next() {
            match argument.as_str() {
                Self::PROFILE_SWITCH => profile = Some(Self::value_of(argument, remaining.next())?),
                DesktopEntry::WEB_APP_SWITCH => web_app = Some(Self::value_of(argument, remaining.next())?),
                Self::TOR_SWITCH => tor_window = true,
                switch if switch.starts_with('-') => return Err(LaunchRequestError::UnknownSwitch(switch.to_owned())),
                address => addresses.push(address.to_owned()),
            }
        }
        let target = match (web_app, profile, tor_window) {
            (Some(web_app), None, false) if addresses.is_empty() => {
                LaunchTarget::WebApp(Self::parsed_web_app_id(web_app)?)
            }
            (Some(_), _, _) | (None, Some(_), true) => return Err(LaunchRequestError::ConflictingTargets),
            (None, None, true) => LaunchTarget::TorWindow,
            (None, profile, false) => LaunchTarget::Browser(Self::browser_profile_id(profile)?),
        };
        Ok(Self { target, addresses })
    }

    #[must_use]
    pub fn application_id(&self) -> ApplicationId {
        match &self.target {
            LaunchTarget::Browser(profile) => profile.application_id(),
            LaunchTarget::WebApp(web_app) => web_app.application_id(),
            LaunchTarget::TorWindow => ApplicationId::tor_window(),
        }
    }

    /// The profile whose data the process uses; `None` for the Tor window, which keeps nothing on disk.
    #[must_use]
    pub fn profile_id(&self) -> Option<ProfileId> {
        match &self.target {
            LaunchTarget::Browser(profile) => Some(profile.clone()),
            LaunchTarget::WebApp(web_app) => Some(web_app.profile_id()),
            LaunchTarget::TorWindow => None,
        }
    }

    #[must_use]
    pub fn web_app_id(&self) -> Option<&WebAppId> {
        match &self.target {
            LaunchTarget::WebApp(web_app) => Some(web_app),
            LaunchTarget::Browser(_) | LaunchTarget::TorWindow => None,
        }
    }

    #[must_use]
    pub fn is_tor_window(&self) -> bool {
        self.target == LaunchTarget::TorWindow
    }

    /// Addresses as given; the browser window interprets each like text typed into the address bar.
    #[must_use]
    pub fn addresses(&self) -> &[String] {
        &self.addresses
    }

    fn value_of<'a>(switch: &str, value: Option<&'a String>) -> Result<&'a str, LaunchRequestError> {
        value.map(String::as_str).ok_or_else(|| LaunchRequestError::MissingValue(switch.to_owned()))
    }

    fn parsed_web_app_id(text: &str) -> Result<WebAppId, LaunchRequestError> {
        WebAppId::parse(text).map_err(|_| LaunchRequestError::InvalidIdentifier(text.to_owned()))
    }

    fn browser_profile_id(text: Option<&str>) -> Result<ProfileId, LaunchRequestError> {
        let Some(text) = text else { return Ok(ProfileId::default_profile()) };
        let profile = ProfileId::parse(text).map_err(|_| LaunchRequestError::InvalidIdentifier(text.to_owned()))?;
        if profile.belongs_to_web_app() {
            return Err(LaunchRequestError::ReservedProfile(text.to_owned()));
        }
        Ok(profile)
    }
}
