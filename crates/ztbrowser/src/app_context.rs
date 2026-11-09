//! What every window of this process shares: file locations, the HTTP client, the live profile, its catalogs
//! and toolbar buttons, the list of open browser windows and, in the Tor window's process, its Tor.

use std::rc::Rc;

use crate::app_icons::AppIcons;
use crate::browser_windows::BrowserWindows;
use crate::catalog_files::CatalogFiles;
use crate::http_client::HttpClient;
use crate::installation_paths::InstallationPaths;
use crate::interface_language_setting::InterfaceLanguageSetting;
use crate::profile_session::ProfileSession;
use crate::toolbar_preferences::ToolbarPreferences;
use crate::tor_connection::TorConnection;

pub struct AppContext {
    pub paths: InstallationPaths,
    pub http: Rc<HttpClient>,
    pub catalogs: CatalogFiles,
    pub icons: AppIcons,
    pub session: Rc<ProfileSession>,
    pub toolbar: ToolbarPreferences,
    pub interface_language: InterfaceLanguageSetting,
    pub browser_windows: BrowserWindows,
    /// `Some` only in the Tor window's process.
    pub tor: Option<Rc<TorConnection>>,
}
