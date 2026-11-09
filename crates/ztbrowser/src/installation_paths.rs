//! Where ZTBrowser keeps its files, following the XDG base directory layout.
//!
//! Data that must survive (`~/.local/share/ztbrowser`): profile and web app catalogs, each profile's cookies
//! and site storage, web app icons, the Tor window's Tor state and connection settings. Data that can be
//! rebuilt (`~/.cache/ztbrowser`): HTTP caches, the downloaded tracker lists and compiled WebKit rules. Data
//! of the running Tor window (`$XDG_RUNTIME_DIR/ztbrowser`, memory that the system empties at logout): its
//! compiled WebKit rules, which name sites it visited.

use std::path::Path;
use std::path::PathBuf;

use ztbrowser_core::profile::profile_directories::ProfileDirectories;
use ztbrowser_core::profile::profile_directories::ProfileLocation;
use ztbrowser_core::profile::profile_id::ProfileId;
use ztbrowser_core::web_app::web_app_id::WebAppId;

pub struct InstallationPaths {
    data_root: PathBuf,
    cache_root: PathBuf,
    runtime_root: PathBuf,
    applications: PathBuf,
    downloads: PathBuf,
}

impl InstallationPaths {
    const DIRECTORY: &'static str = "ztbrowser";

    pub fn from_environment() -> Self {
        let data_home = glib::user_data_dir();
        Self {
            data_root: data_home.join(Self::DIRECTORY),
            cache_root: glib::user_cache_dir().join(Self::DIRECTORY),
            runtime_root: glib::user_runtime_dir().join(Self::DIRECTORY),
            applications: data_home.join("applications"),
            downloads: glib::user_special_dir(glib::UserDirectory::Downloads)
                .unwrap_or_else(|| glib::home_dir().join("Downloads")),
        }
    }

    pub fn profile(&self, id: &ProfileId) -> ProfileLocation {
        ProfileDirectories::new(&self.data_root, &self.cache_root).for_profile(id)
    }

    pub fn profile_catalog(&self) -> PathBuf {
        self.data_root.join("profiles.json")
    }

    /// The interface language the person chose, shared by every profile and web app.
    pub fn interface_language(&self) -> PathBuf {
        self.data_root.join("interface.json")
    }

    pub fn web_app_catalog(&self) -> PathBuf {
        self.data_root.join("web-apps.json")
    }

    pub fn web_app_icon(&self, id: &WebAppId) -> PathBuf {
        self.data_root.join("web-apps").join(id.as_str()).join("icon.png")
    }

    pub fn desktop_entry(&self, file_name: &str) -> PathBuf {
        self.applications.join(file_name)
    }

    /// Downloaded Tracker Data Set and privacy configuration, shared by every profile.
    pub fn protection_feed(&self) -> PathBuf {
        self.cache_root.join("protections")
    }

    /// Icon theme directory holding ZTBrowser's own icon.
    pub fn icon_theme(&self) -> PathBuf {
        self.data_root.join("icons")
    }

    /// Tor's own data directory (`DataDirectory`): its entry guards and cached relay directory.
    pub fn tor_data(&self) -> PathBuf {
        self.data_root.join("tor")
    }

    /// How the Tor window reaches Tor: directly or through which bridges.
    pub fn tor_connection_settings(&self) -> PathBuf {
        self.data_root.join("tor-connection.json")
    }

    /// What the Tor window keeps while it runs (its compiled WebKit rules and its own copy of the protection
    /// data), in memory-backed storage; deleted when the Tor window starts and when it quits.
    pub fn tor_session(&self) -> PathBuf {
        self.runtime_root.join("tor-session")
    }

    pub fn downloads(&self) -> &Path {
        &self.downloads
    }
}
