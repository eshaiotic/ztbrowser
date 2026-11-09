//! "Install Site as App": turns the open page into a separate application with its own profile.
//!
//! The page's own metadata and the site's manifest arrive as JSON (the page cannot tamper with the script that
//! collects them), the icons as PNG files the interface already downloaded without cookies. This type decides
//! what the app is called and which icon to try first, and writes what the desktop needs: a catalogue entry, the
//! icon, and a `.desktop` file in `~/.local/share/applications` that puts the app into the application menu and
//! lets the compositor match the app's window to that icon.

use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

use serde_json::Value;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::storage::owned_json_file::OwnedJsonFile;
use ztbrowser_core::web_app::desktop_entry::DesktopEntry;
use ztbrowser_core::web_app::installed_web_app::InstalledWebApp;
use ztbrowser_core::web_app::page_metadata::PageMetadata;
use ztbrowser_core::web_app::png_header::PngHeader;
use ztbrowser_core::web_app::site_icon_choice::SiteIconChoice;
use ztbrowser_core::web_app::web_app_catalog::WebAppCatalog;
use ztbrowser_core::web_app::web_app_catalog::WebAppInstallError;
use ztbrowser_core::web_app::web_app_draft::WebAppDraft;
use ztbrowser_core::web_app::web_app_manifest::WebAppManifest;

use crate::installation_directories::InstallationDirectories;

/// A PNG file whose declared dimensions are sane: a few kilobytes on the wire must not become gigabytes once
/// decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconPng {
    bytes: Vec<u8>,
}

impl IconPng {
    /// `None` unless `bytes` start like a PNG of a reasonable size.
    #[must_use]
    pub fn accept(bytes: &[u8]) -> Option<Self> {
        PngHeader::read(bytes).filter(PngHeader::is_reasonable_icon).map(|_| Self { bytes: bytes.to_vec() })
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// What the person is offered: a name to confirm and the addresses of the icons to try, best first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationDraft {
    pub name: String,
    pub start_url: String,
    pub icon_addresses: Vec<String>,
}

#[derive(Debug)]
pub enum InstallError {
    NotAWebAddress,
    Storage(io::Error),
}

impl fmt::Display for InstallError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotAWebAddress => WebAppInstallError::NotAWebAddress.fmt(formatter),
            Self::Storage(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for InstallError {}

impl From<io::Error> for InstallError {
    fn from(error: io::Error) -> Self {
        Self::Storage(error)
    }
}

pub struct WebAppInstaller<'context> {
    directories: &'context InstallationDirectories,
    executable: PathBuf,
    catalogue: &'context MessageCatalogue,
}

impl<'context> WebAppInstaller<'context> {
    /// `executable` is the file that starts ZTBrowser (the AppImage when running from one).
    #[must_use]
    pub fn new(
        directories: &'context InstallationDirectories,
        executable: PathBuf,
        catalogue: &'context MessageCatalogue,
    ) -> Self {
        Self { directories, executable, catalogue }
    }

    /// The manifest the page links to, which the interface downloads before asking for [`Self::draft`].
    #[must_use]
    pub fn manifest_address(metadata: &Value) -> Option<String> {
        PageMetadata::from_json(metadata).manifest_url
    }

    /// The manifest wins where it has an opinion, the page fills the gaps; the icons come sharpest first.
    #[must_use]
    pub fn draft(page_url: &str, metadata: &Value, manifest: Option<&Value>) -> InstallationDraft {
        let page = PageMetadata::from_json(metadata);
        let manifest = manifest
            .zip(page.manifest_url.as_deref())
            .map(|(document, address)| WebAppManifest::from_json(document, address));
        let draft = WebAppDraft::new(page_url, &page, manifest.as_ref());
        let mut remaining = draft.icons;
        let mut icon_addresses = Vec::new();
        while let Some(best) = SiteIconChoice::new(&remaining).best().cloned() {
            remaining.retain(|icon| icon != &best);
            icon_addresses.push(best.url);
        }
        InstallationDraft { name: draft.name, start_url: draft.start_url, icon_addresses }
    }

    /// Registers the app and writes its icon (`fallback_icon` as an SVG when the site gave no usable PNG) and its
    /// desktop entry. The catalogue is saved last, so a failure half-way leaves no app that cannot start.
    ///
    /// # Errors
    /// [`InstallError::NotAWebAddress`] for anything but an `http(s)` page; [`InstallError::Storage`] when a file
    /// cannot be written.
    pub fn install(
        &self,
        name: &str,
        start_url: &str,
        icon: Option<&IconPng>,
        fallback_icon: &str,
    ) -> Result<InstalledWebApp, InstallError> {
        let catalog_file = OwnedJsonFile::new(self.directories.web_app_catalog());
        let mut catalog = catalog_file.read().map(|document| WebAppCatalog::from_json(&document)).unwrap_or_default();
        let app = catalog.install(name, start_url).map_err(|_| InstallError::NotAWebAddress)?.clone();
        let icon_path = self.write_icon(&app, icon, fallback_icon)?;
        let entry = DesktopEntry::new(&app, &self.executable, &icon_path, self.catalogue);
        let applications = self.directories.applications_directory();
        fs::create_dir_all(applications)?;
        fs::write(applications.join(entry.file_name()), entry.render())?;
        catalog_file.write(&catalog.to_json())?;
        Ok(app)
    }

    fn write_icon(&self, app: &InstalledWebApp, icon: Option<&IconPng>, fallback_icon: &str) -> io::Result<PathBuf> {
        let directory = self.directories.web_app_icon_directory(&app.id);
        fs::create_dir_all(&directory)?;
        let (file_name, bytes) = match icon {
            Some(png) => ("icon.png", png.as_bytes()),
            None => ("icon.svg", fallback_icon.as_bytes()),
        };
        let path = directory.join(file_name);
        fs::write(&path, bytes)?;
        Ok(path)
    }
}
