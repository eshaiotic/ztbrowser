//! The folder the desktop calls "Downloads".

use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub struct SystemDownloadsFolder;

impl SystemDownloadsFolder {
    /// `XDG_DOWNLOAD_DIR` of `~/.config/user-dirs.dirs` (or under `$XDG_CONFIG_HOME`), else `~/Downloads`.
    #[must_use]
    pub fn locate(home: &Path, config_home: &Path) -> PathBuf {
        fs::read_to_string(config_home.join("user-dirs.dirs"))
            .ok()
            .and_then(|document| Self::from_document(&document, home))
            .unwrap_or_else(|| home.join("Downloads"))
    }

    /// Reads the line `XDG_DOWNLOAD_DIR="$HOME/Downloads"`; a path outside the home folder or a relative one is ignored.
    fn from_document(document: &str, home: &Path) -> Option<PathBuf> {
        let value =
            document.lines().find_map(|line| line.trim().strip_prefix("XDG_DOWNLOAD_DIR="))?.trim().trim_matches('"');
        let path = match value.strip_prefix("$HOME") {
            Some(rest) => home.join(rest.trim_start_matches('/')),
            None => PathBuf::from(value),
        };
        path.is_absolute().then_some(path)
    }
}
