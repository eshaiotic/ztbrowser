//! What this process was asked to be: the browser in a profile, or one installed web app.
//!
//! `ztbrowser-qt [--profile <id>] [address...]` opens the browser, `ztbrowser-qt --web-app <id>` one installed web
//! app (what its `.desktop` file runs). A web app has a profile of its own and an identity of its own: the
//! desktop matches its window to the app's `.desktop` file and icon by that identity, so the window shows the
//! app's icon, not a generic one.

use std::fmt;
use std::path::PathBuf;
use std::sync::OnceLock;

use ztbrowser_core::launch::launch_request::LaunchRequest;
use ztbrowser_core::launch::launch_request::LaunchRequestError;
use ztbrowser_core::profile::profile_id::ProfileId;
use ztbrowser_core::storage::owned_json_file::OwnedJsonFile;
use ztbrowser_core::web_app::installed_web_app::InstalledWebApp;
use ztbrowser_core::web_app::web_app_catalog::WebAppCatalog;

use crate::installation_directories::InstallationDirectories;
use crate::single_instance::PrimaryInstance;

#[derive(Debug)]
pub enum LaunchError {
    Request(LaunchRequestError),
    /// The identifier is well formed, but no installed app has it.
    UnknownWebApp(String),
    TorWindowUnavailable,
}

impl fmt::Display for LaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(error) => error.fmt(formatter),
            Self::UnknownWebApp(id) => write!(formatter, "no web app {id} is installed"),
            Self::TorWindowUnavailable => formatter.write_str("the Tor window is not available in this edition yet"),
        }
    }
}

impl std::error::Error for LaunchError {}

pub struct LaunchContext {
    request: LaunchRequest,
    web_app: Option<InstalledWebApp>,
    directories: InstallationDirectories,
}

static CURRENT: OnceLock<LaunchContext> = OnceLock::new();
static PRIMARY: OnceLock<PrimaryInstance> = OnceLock::new();

impl LaunchContext {
    /// # Errors
    /// [`LaunchError`] for options ZTBrowser does not know, an app that is not installed, or a Tor window.
    pub fn resolve(arguments: &[String], directories: &InstallationDirectories) -> Result<Self, LaunchError> {
        let request = LaunchRequest::parse(arguments).map_err(LaunchError::Request)?;
        if request.is_tor_window() {
            return Err(LaunchError::TorWindowUnavailable);
        }
        let web_app = match request.web_app_id() {
            Some(id) => {
                let catalog = OwnedJsonFile::new(directories.web_app_catalog())
                    .read()
                    .map(|document| WebAppCatalog::from_json(&document))
                    .unwrap_or_default();
                Some(catalog.web_app(id).cloned().ok_or_else(|| LaunchError::UnknownWebApp(id.to_string()))?)
            }
            None => None,
        };
        Ok(Self { request, web_app, directories: directories.clone() })
    }

    /// Makes this the launch of the process; a second call is ignored.
    pub fn install(self) {
        let _ = CURRENT.set(self);
    }

    #[must_use]
    pub fn current() -> Option<&'static Self> {
        CURRENT.get()
    }

    /// Keeps the socket that receives later launches open for as long as the process runs.
    pub fn hold(primary: PrimaryInstance) {
        let _ = PRIMARY.set(primary);
    }

    /// What later launches asked since the last call; see [`PrimaryInstance::take_forwarded`].
    #[must_use]
    pub fn take_forwarded() -> Vec<Vec<String>> {
        PRIMARY.get().map(PrimaryInstance::take_forwarded).unwrap_or_default()
    }

    #[must_use]
    pub fn web_app(&self) -> Option<&InstalledWebApp> {
        self.web_app.as_ref()
    }

    /// The addresses to open, as typed on the command line.
    #[must_use]
    pub fn addresses(&self) -> &[String] {
        self.request.addresses()
    }

    #[must_use]
    pub fn directories(&self) -> &InstallationDirectories {
        &self.directories
    }

    /// The profile whose data the process uses.
    #[must_use]
    pub fn profile_id(&self) -> ProfileId {
        self.request.profile_id().unwrap_or_else(ProfileId::default_profile)
    }

    /// The identity the window announces to the desktop (Wayland's `app_id`, X11's window class), which names
    /// the `.desktop` file that carries the icon.
    #[must_use]
    pub fn application_id(&self) -> String {
        self.request.application_id().to_string()
    }

    /// The app's own icon file, when the process runs a web app and the installer left one.
    #[must_use]
    pub fn window_icon(&self) -> Option<PathBuf> {
        self.directories.web_app_icon(&self.web_app.as_ref()?.id)
    }
}
