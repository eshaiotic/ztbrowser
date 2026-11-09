//! The folder the desktop calls "Downloads".

use std::fs;
use std::path::Path;

use tempfile::TempDir;
use ztbrowser_qt::system_downloads_folder::SystemDownloadsFolder;

use ztbrowser_qt as _;

fn folder_for(document: Option<&str>) -> std::path::PathBuf {
    let config = TempDir::new().expect("config");
    if let Some(document) = document {
        fs::write(config.path().join("user-dirs.dirs"), document).expect("document");
    }
    SystemDownloadsFolder::locate(Path::new("/home/ann"), config.path())
}

#[test]
fn should_use_the_downloads_folder_of_the_home_when_the_desktop_says_nothing() {
    assert_eq!(folder_for(None), Path::new("/home/ann/Downloads"));
}

#[test]
fn should_follow_the_desktops_choice_relative_to_home() {
    assert_eq!(folder_for(Some("XDG_DOWNLOAD_DIR=\"$HOME/Pobrane\"\n")), Path::new("/home/ann/Pobrane"));
}

#[test]
fn should_follow_an_absolute_choice() {
    assert_eq!(folder_for(Some("XDG_DOWNLOAD_DIR=\"/data/dl\"\n")), Path::new("/data/dl"));
}

#[test]
fn should_ignore_a_relative_choice() {
    assert_eq!(folder_for(Some("XDG_DOWNLOAD_DIR=\"dl\"\n")), Path::new("/home/ann/Downloads"));
}
