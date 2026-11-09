//! Hands work to other processes: another ZTBrowser process (browser, profile, web app) or another program.
//!
//! ZTBrowser processes are GTK applications with fixed ids, so starting `ztbrowser <address>` while the browser
//! runs only forwards the address to the running browser over D-Bus; the started process exits at once.

use std::ffi::OsStr;
use std::path::PathBuf;

use url::Url;
use ztbrowser_core::launch::launch_request::LaunchRequest;
use ztbrowser_core::launch::program_location::ProgramLocation;
use ztbrowser_core::profile::profile_id::ProfileId;
use ztbrowser_core::web_app::desktop_entry::DesktopEntry;
use ztbrowser_core::web_app::web_app_id::WebAppId;

pub struct ExternalLauncher;

impl ExternalLauncher {
    const LOG_DOMAIN: &'static str = "ztbrowser";

    /// Opens a web address in the default profile's browser; `false` when nothing was started (not a web
    /// address, or ZTBrowser could not be started, which is logged).
    pub fn open_in_browser(address: &str) -> bool {
        Url::parse(address).is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
            && Self::start_ztbrowser(&[address])
    }

    pub fn start_profile(profile: &ProfileId) {
        if profile.is_default() {
            Self::start_ztbrowser(&[]);
        } else {
            Self::start_ztbrowser(&[LaunchRequest::PROFILE_SWITCH, profile.as_str()]);
        }
    }

    /// Opens a Tor window, in the Tor window's own process.
    pub fn start_tor_window() {
        Self::start_ztbrowser(&[LaunchRequest::TOR_SWITCH]);
    }

    /// `false` when the app's process could not be started (logged).
    pub fn start_web_app(web_app: &WebAppId) -> bool {
        Self::start_ztbrowser(&[DesktopEntry::WEB_APP_SWITCH, web_app.as_str()])
    }

    /// Opens a non-web link (`mailto:`, `tel:`, ...) with the program the desktop chooses for it.
    pub fn open_outside(address: &str, parent: Option<&gtk::Window>) {
        gtk::UriLauncher::new(address).launch(parent, None::<&gio::Cancellable>, |result| {
            if let Err(error) = result {
                glib::g_message!(Self::LOG_DOMAIN, "no application opened the link: {error}");
            }
        });
    }

    /// The file that starts ZTBrowser: the AppImage when running from one, else the running executable.
    ///
    /// # Errors
    /// The running executable cannot be determined.
    pub fn program() -> std::io::Result<PathBuf> {
        let appimage = std::env::var_os(ProgramLocation::APPIMAGE_VARIABLE);
        Ok(ProgramLocation::choose(appimage.as_deref(), std::env::current_exe()?))
    }

    /// `false` when the process could not be started (logged).
    fn start_ztbrowser(arguments: &[&str]) -> bool {
        let executable = match Self::program() {
            Ok(executable) => executable,
            Err(error) => {
                glib::g_warning!(Self::LOG_DOMAIN, "cannot locate the ZTBrowser executable: {error}");
                return false;
            }
        };
        let mut command_line = vec![executable.as_os_str()];
        command_line.extend(arguments.iter().map(OsStr::new));
        match gio::Subprocess::newv(&command_line, gio::SubprocessFlags::NONE) {
            Ok(process) => {
                process.wait_async(None::<&gio::Cancellable>, |_| {});
                true
            }
            Err(error) => {
                glib::g_warning!(Self::LOG_DOMAIN, "cannot start ZTBrowser: {error}");
                false
            }
        }
    }
}
