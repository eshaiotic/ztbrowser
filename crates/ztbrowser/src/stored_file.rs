//! A file ZTBrowser owns: written atomically and readable only by the user.
//!
//! Writes go to `<name>.part` and are renamed over the target, so a crash never leaves half a catalog
//! behind. Directories are created `0700` and files `0600`: profile data includes session cookies.
//!
//! The Tor window keeps its settings in memory only, so its files are [`StoredFile::discarded`]: they read as
//! missing and drop what is written to them.

use std::fs;
use std::fs::DirBuilder;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;

pub struct StoredFile {
    /// `None` for a discarded file.
    path: Option<PathBuf>,
}

impl StoredFile {
    const LOG_DOMAIN: &'static str = "ztbrowser";

    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: Some(path.into()) }
    }

    /// A file that is never written: reading finds nothing, writing keeps nothing.
    pub fn discarded() -> Self {
        Self { path: None }
    }

    pub fn exists(&self) -> bool {
        self.path.as_deref().is_some_and(Path::exists)
    }

    /// The parsed document, or `None` when missing or corrupt (corruption is logged, then treated as empty).
    pub fn read_json(&self) -> Option<Value> {
        let path = self.path.as_deref()?;
        let bytes = fs::read(path).ok()?;
        serde_json::from_slice(&bytes)
            .inspect_err(|error| glib::g_warning!(Self::LOG_DOMAIN, "ignoring {}: {error}", path.display()))
            .ok()
    }

    pub fn read_bytes(&self) -> Option<Vec<u8>> {
        fs::read(self.path.as_deref()?).ok()
    }

    pub fn read_text(&self) -> Option<String> {
        fs::read_to_string(self.path.as_deref()?).ok()
    }

    pub fn write_json(&self, document: &Value) -> io::Result<()> {
        self.write(&serde_json::to_vec_pretty(document)?)
    }

    pub fn write(&self, bytes: &[u8]) -> io::Result<()> {
        let Some(path) = self.path.as_deref() else { return Ok(()) };
        if let Some(directory) = path.parent() {
            DirBuilder::new().recursive(true).mode(0o700).create(directory)?;
        }
        let mut partial_name = path.file_name().unwrap_or_default().to_owned();
        partial_name.push(".part");
        let partial = path.with_file_name(partial_name);
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&partial)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&partial, path)
    }

    pub fn remove(&self) -> io::Result<()> {
        let Some(path) = self.path.as_deref() else { return Ok(()) };
        match fs::remove_file(path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            result => result,
        }
    }
}
