//! A request to the system file manager to show a file or a folder, however the desktop has configured folders to open.
//!
//! `xdg-open` hands a folder to whatever application is the default for `inode/directory`, which on some desktops is an
//! editor, not a file manager. The `org.freedesktop.FileManager1` service is what "Show in Folder" means: it selects the
//! file in a file manager window. Desktops without the service get the plain folder open instead.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use url::Url;

const SERVICE: &str = "org.freedesktop.FileManager1";

#[derive(Debug, Clone)]
pub struct FileManagerRequest {
    method: &'static str,
    uri: String,
    fallback_folder: PathBuf,
}

impl FileManagerRequest {
    /// Select `path` in a file manager window; `None` when the file is not there or the path is relative.
    #[must_use]
    pub fn showing_file(path: &Path) -> Option<Self> {
        if !path.is_absolute() || !path.exists() {
            return None;
        }
        Some(Self {
            method: "ShowItems",
            uri: Url::from_file_path(path).ok()?.into(),
            fallback_folder: path.parent()?.to_owned(),
        })
    }

    /// Open `folder` in a file manager window; `None` when it is not a folder or the path is relative.
    #[must_use]
    pub fn showing_folder(folder: &Path) -> Option<Self> {
        if !folder.is_absolute() || !folder.is_dir() {
            return None;
        }
        Some(Self {
            method: "ShowFolders",
            uri: Url::from_directory_path(folder).ok()?.into(),
            fallback_folder: folder.to_owned(),
        })
    }

    #[must_use]
    pub fn fallback_folder(&self) -> &Path {
        &self.fallback_folder
    }

    /// The URI with the two characters that mean something inside a quoted `GVariant` text string written as percent
    /// escapes. `url` leaves `'` alone in a path, and a file name chosen by a web server must not close the string.
    fn gvariant_safe_uri(&self) -> String {
        self.uri.replace('\\', "%5C").replace('\'', "%27")
    }

    /// The arguments of `gdbus call` that make the file manager show the item.
    #[must_use]
    pub fn gdbus_arguments(&self) -> Vec<String> {
        [
            "call",
            "--session",
            "--timeout",
            "10",
            "--dest",
            SERVICE,
            "--object-path",
            "/org/freedesktop/FileManager1",
            "--method",
        ]
        .iter()
        .map(|argument| (*argument).to_owned())
        .chain([format!("{SERVICE}.{}", self.method), format!("['{}']", self.gvariant_safe_uri()), "''".to_owned()])
        .collect()
    }

    /// Ask the file manager on a thread of its own (starting one can take a while); open the folder the plain way
    /// when the desktop has no file-manager service.
    pub fn carry_out(self) {
        std::thread::spawn(move || {
            let shown = Command::new("gdbus")
                .args(self.gdbus_arguments())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success());
            if !shown {
                let _ = Command::new("xdg-open").arg(&self.fallback_folder).spawn().and_then(|mut child| child.wait());
            }
        });
    }
}
