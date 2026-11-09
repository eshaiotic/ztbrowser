//! Where ZTBrowser keeps its files, shared with the GTK edition so profiles, settings and downloaded
//! protection data carry over.

use std::env;
use std::path::Path;
use std::path::PathBuf;

use ztbrowser_core::profile::profile_directories::ProfileDirectories;
use ztbrowser_core::profile::profile_directories::ProfileLocation;
use ztbrowser_core::profile::profile_id::ProfileId;
use ztbrowser_core::web_app::web_app_id::WebAppId;

use crate::system_downloads_folder::SystemDownloadsFolder;

#[derive(Debug, Clone)]
pub struct InstallationDirectories {
    data_root: PathBuf,
    cache_root: PathBuf,
    /// Where the sockets that hand a second launch to the running process live.
    runtime: PathBuf,
    /// Where the desktop looks for `.desktop` files of the person's own applications.
    applications: PathBuf,
    /// Where the desktop looks for the icons of the person's own applications.
    application_icons: PathBuf,
}

impl InstallationDirectories {
    /// The name of ZTBrowser's folder in each XDG base directory.
    pub const DIRECTORY: &'static str = "ztbrowser";
    const DEFAULT_SYSTEM_DATA_DIRECTORIES: &'static str = "/usr/local/share:/usr/share";

    /// Follows the XDG base directory specification: `$XDG_DATA_HOME`, `$XDG_CACHE_HOME` and `$XDG_RUNTIME_DIR`,
    /// else the `~/.local/share` and `~/.cache` defaults.
    #[must_use]
    pub fn from_environment() -> Self {
        Self::under(&Self::data_home(), &Self::cache_home())
    }

    /// The layout below the given XDG data and cache homes; the sockets go to the session's runtime directory.
    #[must_use]
    pub fn under(data_home: &Path, cache_home: &Path) -> Self {
        Self::named(Self::DIRECTORY, data_home, cache_home, Self::runtime_home().as_deref())
    }

    /// The layout of folders called `directory`; the sockets go to `runtime_home`, else to a folder of the cache.
    #[must_use]
    pub fn named(directory: &str, data_home: &Path, cache_home: &Path, runtime_home: Option<&Path>) -> Self {
        let cache_root = cache_home.join(directory);
        Self {
            data_root: data_home.join(directory),
            runtime: runtime_home.map_or_else(|| cache_root.join("run"), |runtime| runtime.join(directory)),
            cache_root,
            applications: data_home.join("applications"),
            application_icons: data_home.join("icons/hicolor/scalable/apps"),
        }
    }

    /// `$XDG_DATA_HOME`, else `~/.local/share`.
    #[must_use]
    pub fn data_home() -> PathBuf {
        Self::directory_from("XDG_DATA_HOME").unwrap_or_else(|| Self::home().join(".local/share"))
    }

    /// `$XDG_CACHE_HOME`, else `~/.cache`.
    #[must_use]
    pub fn cache_home() -> PathBuf {
        Self::directory_from("XDG_CACHE_HOME").unwrap_or_else(|| Self::home().join(".cache"))
    }

    /// The session's runtime directory, `$XDG_RUNTIME_DIR`; the specification has no default for it.
    #[must_use]
    pub fn runtime_home() -> Option<PathBuf> {
        Self::directory_from("XDG_RUNTIME_DIR")
    }

    /// Where packages install their files, menu entries among them: `$XDG_DATA_DIRS`, else the specification's
    /// `/usr/local/share` and `/usr/share`.
    #[must_use]
    pub fn system_data_directories() -> Vec<PathBuf> {
        let configured = env::var_os("XDG_DATA_DIRS").filter(|value| !value.is_empty());
        let listed = configured.unwrap_or_else(|| Self::DEFAULT_SYSTEM_DATA_DIRECTORIES.into());
        env::split_paths(&listed).filter(|path| path.is_absolute()).collect()
    }

    fn home() -> PathBuf {
        env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
    }

    /// An XDG variable counts only when it is an absolute path, as the specification requires.
    fn directory_from(variable: &str) -> Option<PathBuf> {
        env::var_os(variable).map(PathBuf::from).filter(|path| path.is_absolute())
    }

    /// Everything kept for good: profiles, settings, installed web apps.
    #[must_use]
    pub fn data_root(&self) -> &Path {
        &self.data_root
    }

    /// Everything that can be fetched or built again: page caches, downloaded protection data.
    #[must_use]
    pub fn cache_root(&self) -> &Path {
        &self.cache_root
    }

    #[must_use]
    pub fn profile(&self, id: &ProfileId) -> ProfileLocation {
        ProfileDirectories::new(&self.data_root, &self.cache_root).for_profile(id)
    }

    #[must_use]
    pub fn interface_language_file(&self) -> PathBuf {
        self.data_root.join("interface.json")
    }

    /// Whether the person still wants to be asked to make ZTBrowser the default browser, for every profile.
    #[must_use]
    pub fn default_browser_file(&self) -> PathBuf {
        self.data_root.join("default-browser.json")
    }

    /// Where the sockets that hand a second launch to the running process live: the session's runtime
    /// directory, else a folder of the cache.
    #[must_use]
    pub fn runtime_directory(&self) -> PathBuf {
        self.runtime.clone()
    }

    /// The browsing profiles the person created, shared with the GTK edition.
    #[must_use]
    pub fn profile_catalog(&self) -> PathBuf {
        self.data_root.join("profiles.json")
    }

    /// The sites installed as web apps, shared with the GTK edition.
    #[must_use]
    pub fn web_app_catalog(&self) -> PathBuf {
        self.data_root.join("web-apps.json")
    }

    /// The folder holding one installed web app's icon.
    #[must_use]
    pub fn web_app_icon_directory(&self, id: &WebAppId) -> PathBuf {
        self.data_root.join("web-apps").join(id.as_str())
    }

    /// The icon the installer left for a web app, PNG or SVG; nothing when it is missing.
    #[must_use]
    pub fn web_app_icon(&self, id: &WebAppId) -> Option<PathBuf> {
        let directory = self.web_app_icon_directory(id);
        ["icon.png", "icon.svg"].iter().map(|name| directory.join(name)).find(|path| path.is_file())
    }

    /// Where the `.desktop` file of an installed web app goes, so the application menu lists it.
    #[must_use]
    pub fn applications_directory(&self) -> &Path {
        &self.applications
    }

    /// Where the icon of the `.desktop` file of an AppImage goes (the icon theme's scalable application icons).
    #[must_use]
    pub fn application_icons_directory(&self) -> PathBuf {
        self.application_icons.clone()
    }

    /// The folder the desktop calls "Downloads".
    #[must_use]
    pub fn system_downloads_folder(&self) -> PathBuf {
        let home = Self::home();
        let config_home = Self::directory_from("XDG_CONFIG_HOME").unwrap_or_else(|| home.join(".config"));
        SystemDownloadsFolder::locate(&home, &config_home)
    }

    /// Downloaded Tracker Data Set and privacy configuration, shared by every profile.
    #[must_use]
    pub fn protection_cache(&self) -> PathBuf {
        self.cache_root.join("protections")
    }
}
