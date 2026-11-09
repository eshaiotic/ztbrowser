//! A JSON document ZTBrowser owns: written atomically and readable only by the user.
//!
//! Writes go to `<name>.part` and are renamed over the target, so a crash never leaves half a document behind.
//! Directories are created `0700` and files `0600`: profile data includes session cookies.

use std::fs;
use std::fs::DirBuilder;
use std::fs::OpenOptions;
use std::io;
use std::io::Write;
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::fs::OpenOptionsExt;
use std::path::PathBuf;

use serde_json::Value;

pub struct OwnedJsonFile {
    path: PathBuf,
}

impl OwnedJsonFile {
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The parsed document; `None` when the file is missing or not JSON.
    #[must_use]
    pub fn read(&self) -> Option<Value> {
        serde_json::from_slice(&fs::read(&self.path).ok()?).ok()
    }

    /// # Errors
    /// Fails when the directory or the file cannot be written.
    pub fn write(&self, document: &Value) -> io::Result<()> {
        if let Some(directory) = self.path.parent() {
            DirBuilder::new().recursive(true).mode(0o700).create(directory)?;
        }
        let mut partial_name = self.path.file_name().unwrap_or_default().to_owned();
        partial_name.push(".part");
        let partial = self.path.with_file_name(partial_name);
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&partial)?;
        file.write_all(&serde_json::to_vec_pretty(document)?)?;
        file.sync_all()?;
        fs::rename(&partial, &self.path)
    }
}
