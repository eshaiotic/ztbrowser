//! The installed web apps as the browser's apps page shows them, and removing one with everything it owns.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::storage::owned_json_file::OwnedJsonFile;
use ztbrowser_core::web_app::desktop_entry::DesktopEntry;
use ztbrowser_core::web_app::installed_apps_page::AppTile;
use ztbrowser_core::web_app::installed_apps_page::InstalledAppsPage;
use ztbrowser_core::web_app::installed_web_app::InstalledWebApp;
use ztbrowser_core::web_app::web_app_catalog::WebAppCatalog;
use ztbrowser_core::web_app::web_app_id::WebAppId;

use crate::installation_directories::InstallationDirectories;
use crate::single_instance::SingleInstance;

/// The largest icon file put into the page.
const ICON_MAX_BYTES: u64 = 2 * 1024 * 1024;

#[derive(Debug)]
pub enum UninstallError {
    /// The app's window is open somewhere, so its data must stay until it is closed.
    StillRunning(String),
    /// Some of the app's files could not be removed.
    Incomplete(io::Error),
}

pub struct InstalledWebApps<'a> {
    directories: &'a InstallationDirectories,
    runtime_directory: PathBuf,
}

impl<'a> InstalledWebApps<'a> {
    #[must_use]
    pub fn new(directories: &'a InstallationDirectories, runtime_directory: PathBuf) -> Self {
        Self { directories, runtime_directory }
    }

    #[must_use]
    pub fn catalog(&self) -> WebAppCatalog {
        OwnedJsonFile::new(self.directories.web_app_catalog())
            .read()
            .map(|document| WebAppCatalog::from_json(&document))
            .unwrap_or_default()
    }

    /// The apps page: one tile per installed app, with its icon inlined so the page needs no file access.
    #[must_use]
    pub fn page_html(&self, catalogue: &MessageCatalogue, is_dark: bool) -> String {
        let catalog = self.catalog();
        let icons: Vec<Option<String>> = catalog.web_apps().map(|app| self.icon_address(app)).collect();
        let hosts: Vec<String> = catalog.web_apps().map(InstalledWebApp::host).collect();
        let tiles: Vec<AppTile<'_>> = catalog
            .web_apps()
            .zip(icons.iter().zip(&hosts))
            .map(|(app, (icon, host))| AppTile { id: &app.id, name: &app.name, host, icon: icon.as_deref() })
            .collect();
        InstalledAppsPage::new(catalogue, is_dark).render(&tiles)
    }

    /// Removes the app: its catalogue entry, desktop entry, icon, cookies and storage.
    ///
    /// # Errors
    /// [`UninstallError::StillRunning`] while the app is open; [`UninstallError::Incomplete`] when a file stays.
    pub fn uninstall(&self, id: &WebAppId) -> Result<(), UninstallError> {
        let mut catalog = self.catalog();
        let Some(app) = catalog.web_app(id).cloned() else { return Ok(()) };
        if SingleInstance::is_running(&self.runtime_directory, &id.application_id().to_string()) {
            return Err(UninstallError::StillRunning(app.name));
        }
        catalog.uninstall(id);
        let profile = self.directories.profile(&id.profile_id());
        OwnedJsonFile::new(self.directories.web_app_catalog())
            .write(&catalog.to_json())
            .and_then(|()| {
                Self::remove_file(&self.directories.applications_directory().join(DesktopEntry::file_name_for(id)))
            })
            .and_then(|()| Self::remove_directory(&self.directories.web_app_icon_directory(id)))
            .and_then(|()| Self::remove_directory(&profile.data))
            .and_then(|()| Self::remove_directory(&profile.cache))
            .map_err(UninstallError::Incomplete)
    }

    fn icon_address(&self, app: &InstalledWebApp) -> Option<String> {
        let directory = self.directories.web_app_icon_directory(&app.id);
        [("icon.png", "image/png"), ("icon.svg", "image/svg+xml")].into_iter().find_map(|(name, media_type)| {
            let path = directory.join(name);
            let is_small =
                fs::metadata(&path).is_ok_and(|metadata| metadata.is_file() && metadata.len() <= ICON_MAX_BYTES);
            let bytes = if is_small { fs::read(&path).ok()? } else { return None };
            Some(format!("data:{media_type};base64,{}", STANDARD.encode(bytes)))
        })
    }

    fn remove_file(path: &Path) -> io::Result<()> {
        Self::ignoring_missing(fs::remove_file(path))
    }

    fn remove_directory(path: &Path) -> io::Result<()> {
        Self::ignoring_missing(fs::remove_dir_all(path))
    }

    fn ignoring_missing(result: io::Result<()>) -> io::Result<()> {
        match result {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }
}
