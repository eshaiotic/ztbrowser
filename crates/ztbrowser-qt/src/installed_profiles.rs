//! The browsing profiles the person created: listing them as tiles, adding one, and deleting one with
//! everything it owns.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::profile::profile_catalog::ProfileCatalog;
use ztbrowser_core::profile::profile_id::ProfileId;
use ztbrowser_core::profile::profile_tile::ProfileTile;
use ztbrowser_core::profile::profile_tile::ProfileTiles;
use ztbrowser_core::storage::owned_json_file::OwnedJsonFile;

use crate::installation_directories::InstallationDirectories;
use crate::single_instance::SingleInstance;

/// The longest name a profile may be given; the rest is cut off.
const NAME_MAX_CHARACTERS: usize = 40;

#[derive(Debug)]
pub enum DeleteError {
    /// The default profile cannot be deleted.
    IsDefault,
    /// The profile's window is open somewhere, so its data must stay until it is closed.
    StillRunning(String),
    /// Some of the profile's files could not be removed.
    Incomplete(io::Error),
}

pub struct InstalledProfiles<'a> {
    directories: &'a InstallationDirectories,
    runtime_directory: PathBuf,
}

impl<'a> InstalledProfiles<'a> {
    #[must_use]
    pub fn new(directories: &'a InstallationDirectories, runtime_directory: PathBuf) -> Self {
        Self { directories, runtime_directory }
    }

    #[must_use]
    pub fn catalog(&self) -> ProfileCatalog {
        OwnedJsonFile::new(self.directories.profile_catalog())
            .read()
            .map(|document| ProfileCatalog::from_json(&document))
            .unwrap_or_default()
    }

    #[must_use]
    pub fn tiles(&self, current: &ProfileId, catalogue: &MessageCatalogue) -> Vec<ProfileTile> {
        ProfileTiles::of(&self.catalog(), current, catalogue)
    }

    /// Adds a profile called `name` (blank names and over-long ones are tidied) and returns its identifier.
    ///
    /// # Errors
    /// The catalogue file could not be written.
    pub fn create(&self, name: &str) -> io::Result<Option<ProfileId>> {
        let name: String = name.trim().chars().take(NAME_MAX_CHARACTERS).collect();
        if name.is_empty() {
            return Ok(None);
        }
        let mut catalog = self.catalog();
        let id = catalog.create_profile(&name);
        OwnedJsonFile::new(self.directories.profile_catalog()).write(&catalog.to_json())?;
        Ok(Some(id))
    }

    /// Forgets the profile and removes its cookies, storage and cache.
    ///
    /// # Errors
    /// [`DeleteError::IsDefault`] for the default profile; [`DeleteError::StillRunning`] while it is open;
    /// [`DeleteError::Incomplete`] when a file stays.
    pub fn delete(&self, id: &ProfileId) -> Result<(), DeleteError> {
        if id.is_default() {
            return Err(DeleteError::IsDefault);
        }
        let mut catalog = self.catalog();
        let Some(profile) = catalog.profile(id).cloned() else { return Ok(()) };
        if SingleInstance::is_running(&self.runtime_directory, &id.application_id().to_string()) {
            return Err(DeleteError::StillRunning(profile.name));
        }
        catalog.remove_profile(id);
        let location = self.directories.profile(id);
        OwnedJsonFile::new(self.directories.profile_catalog())
            .write(&catalog.to_json())
            .and_then(|()| Self::remove_directory(&location.data))
            .and_then(|()| Self::remove_directory(&location.cache))
            .map_err(DeleteError::Incomplete)
    }

    fn remove_directory(path: &Path) -> io::Result<()> {
        match fs::remove_dir_all(path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            other => other,
        }
    }
}
