//! ZTBrowser was called InDuckGo up to version 1.2.4. A person who used InDuckGo keeps profiles, installed web apps
//! and settings: a ZTBrowser start moves InDuckGo's folders (`induckgo` in the XDG data and cache folders) to
//! ZTBrowser's and points the application-menu entries InDuckGo wrote at ZTBrowser.
//!
//! A process keeps using the paths it opened, and Chromium's profile files must never be open in two processes, so
//! the folders move only while no process runs on them. An InDuckGo left open across the upgrade still does; so
//! does a ZTBrowser started beside it, which keeps using InDuckGo's folders and listens beside InDuckGo, so the move
//! waits for it too. Meanwhile a launch of a window InDuckGo already shows (the same profile or web app) is handed
//! to that InDuckGo, as every second launch is handed to the first. Probing for such a process connects to its
//! socket, which brings its window forward.
//!
//! The folders are renamed, not copied: each stays in the same parent folder, so the move is instant. A person who
//! already has ZTBrowser folders keeps them, and InDuckGo's stay untouched beside them.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use ztbrowser_core::identity::application_id::ApplicationId;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::storage::owned_json_file::OwnedJsonFile;
use ztbrowser_core::web_app::desktop_entry::DesktopEntry;
use ztbrowser_core::web_app::web_app_catalog::WebAppCatalog;

use crate::installation_directories::InstallationDirectories;
use crate::single_instance::SingleInstance;

pub struct InDuckGoMigration {
    previous: InstallationDirectories,
    current: InstallationDirectories,
    /// Qt's own cache of compiled QML and shaders, named after the application name.
    previous_qt_cache: PathBuf,
}

impl InDuckGoMigration {
    const PREVIOUS_DIRECTORY: &'static str = "induckgo";
    const PREVIOUS_QT_CACHE: &'static str = "InDuckGo";
    const PREVIOUS_APPLICATION_ID: &'static str = "io.github.induckgo.InDuckGo";
    /// The line that marks the menu entry InDuckGo's AppImage wrote for itself.
    const PREVIOUS_APPIMAGE_MARKER: &'static str = "X-InDuckGo-AppImage=";

    #[must_use]
    pub fn new(data_home: &Path, cache_home: &Path, runtime_home: Option<&Path>) -> Self {
        Self {
            previous: InstallationDirectories::named(Self::PREVIOUS_DIRECTORY, data_home, cache_home, runtime_home),
            current: InstallationDirectories::named(
                InstallationDirectories::DIRECTORY,
                data_home,
                cache_home,
                runtime_home,
            ),
            previous_qt_cache: cache_home.join(Self::PREVIOUS_QT_CACHE),
        }
    }

    #[must_use]
    pub fn from_environment() -> Self {
        Self::new(
            &InstallationDirectories::data_home(),
            &InstallationDirectories::cache_home(),
            InstallationDirectories::runtime_home().as_deref(),
        )
    }

    /// Whether InDuckGo's folders still hold the person's data: they exist and ZTBrowser has no folder of its own.
    #[must_use]
    pub fn is_pending(&self) -> bool {
        self.previous.data_root().is_dir() && !self.current.data_root().exists()
    }

    /// The folders this process uses: InDuckGo's while they still hold the person's data, else ZTBrowser's.
    #[must_use]
    pub fn directories(&self) -> InstallationDirectories {
        if self.is_pending() { self.previous.clone() } else { self.current.clone() }
    }

    /// Moves InDuckGo's folders to ZTBrowser's, unless a process still runs on them. The cache moves first, so the
    /// data folder's move is the step that completes the migration; Qt's InDuckGo cache is deleted after it.
    ///
    /// # Errors
    /// A folder could not be renamed. InDuckGo's data then stays where it is, is used from there, and the next
    /// start tries again.
    pub fn move_folders(&self) -> io::Result<()> {
        if !self.is_pending() || SingleInstance::any_running(&self.previous.runtime_directory()) {
            return Ok(());
        }
        if self.previous.cache_root().exists() && !self.current.cache_root().exists() {
            fs::rename(self.previous.cache_root(), self.current.cache_root())?;
        }
        fs::rename(self.previous.data_root(), self.current.data_root())?;
        Self::ignoring_missing(fs::remove_dir_all(&self.previous_qt_cache))
    }

    /// The identity InDuckGo announced for the window `identity` names: the same profile or web app, old name.
    #[must_use]
    pub fn previous_identity(identity: &str) -> Option<String> {
        let current = ApplicationId::browser();
        identity.strip_prefix(current.as_str()).map(|rest| format!("{}{rest}", Self::PREVIOUS_APPLICATION_ID))
    }

    /// Hands `addresses` to an InDuckGo that shows the window `identity` names, while InDuckGo's folders are in
    /// use; `true` when it took them and this process is not needed.
    #[must_use]
    pub fn hand_over_to_running_induckgo(&self, identity: &str, addresses: &[String]) -> bool {
        self.is_pending()
            && Self::previous_identity(identity).is_some_and(|previous| {
                SingleInstance::forward(&self.previous.runtime_directory(), &previous, addresses).unwrap_or(false)
            })
    }

    /// Once the data is ZTBrowser's, points the menu entries InDuckGo wrote at ZTBrowser: each web app gets
    /// ZTBrowser's entry (starting `executable`, showing the icon from its new folder) in place of InDuckGo's, and
    /// the entry InDuckGo's AppImage wrote for itself is withdrawn. An entry the person wrote stays.
    ///
    /// # Errors
    /// An entry could not be written or removed.
    pub fn repoint_menu_entries(&self, executable: &Path, catalogue: &MessageCatalogue) -> io::Result<()> {
        if self.is_pending() {
            return Ok(());
        }
        let applications = self.current.applications_directory();
        let catalog = OwnedJsonFile::new(self.current.web_app_catalog())
            .read()
            .map(|document| WebAppCatalog::from_json(&document))
            .unwrap_or_default();
        for app in catalog.web_apps() {
            let file_name = DesktopEntry::file_name_for(&app.id);
            let Some(previous) = Self::previous_identity(&file_name).map(|name| applications.join(name)) else {
                continue;
            };
            if !previous.is_file() {
                continue;
            }
            let icon =
                self.current.web_app_icon(&app.id).unwrap_or_else(|| self.current.web_app_icon_directory(&app.id));
            fs::write(applications.join(file_name), DesktopEntry::new(app, executable, &icon, catalogue).render())?;
            fs::remove_file(previous)?;
        }
        self.withdraw_appimage_entry()
    }

    fn withdraw_appimage_entry(&self) -> io::Result<()> {
        let entry = self.current.applications_directory().join(format!("{}.desktop", Self::PREVIOUS_APPLICATION_ID));
        let is_marked = fs::read_to_string(&entry)
            .is_ok_and(|text| text.lines().any(|line| line.starts_with(Self::PREVIOUS_APPIMAGE_MARKER)));
        if !is_marked {
            return Ok(());
        }
        fs::remove_file(entry)?;
        let icon = self.current.application_icons_directory().join(format!("{}.svg", Self::PREVIOUS_APPLICATION_ID));
        Self::ignoring_missing(fs::remove_file(icon))
    }

    fn ignoring_missing(result: io::Result<()>) -> io::Result<()> {
        match result {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }
}
