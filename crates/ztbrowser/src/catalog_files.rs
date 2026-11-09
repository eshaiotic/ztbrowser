//! Loading and saving the profile and web app catalogs.
//!
//! The browser process writes them; web app and profile processes only read them at start-up. A missing or
//! corrupt file reads as an empty catalog (which still contains the default profile).

use std::io;

use serde_json::Value;
use ztbrowser_core::profile::profile_catalog::ProfileCatalog;
use ztbrowser_core::web_app::web_app_catalog::WebAppCatalog;

use crate::installation_paths::InstallationPaths;
use crate::stored_file::StoredFile;

pub struct CatalogFiles {
    profiles: StoredFile,
    web_apps: StoredFile,
}

impl CatalogFiles {
    pub fn new(paths: &InstallationPaths) -> Self {
        Self { profiles: StoredFile::new(paths.profile_catalog()), web_apps: StoredFile::new(paths.web_app_catalog()) }
    }

    pub fn profiles(&self) -> ProfileCatalog {
        ProfileCatalog::from_json(&self.profiles.read_json().unwrap_or(Value::Null))
    }

    pub fn save_profiles(&self, catalog: &ProfileCatalog) -> io::Result<()> {
        self.profiles.write_json(&catalog.to_json())
    }

    pub fn web_apps(&self) -> WebAppCatalog {
        WebAppCatalog::from_json(&self.web_apps.read_json().unwrap_or(Value::Null))
    }

    pub fn save_web_apps(&self, catalog: &WebAppCatalog) -> io::Result<()> {
        self.web_apps.write_json(&catalog.to_json())
    }
}
