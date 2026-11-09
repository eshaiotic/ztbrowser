//! Showing a download in the system file manager: the file is selected in its folder, and a folder is shown as is.

use std::path::Path;

use tempfile::TempDir;
use ztbrowser_qt::file_manager_request::FileManagerRequest;

use ztbrowser_qt as _;

fn given_a_downloaded_file(directory: &TempDir) -> std::path::PathBuf {
    let path = directory.path().join("my report.pdf");
    std::fs::write(&path, b"pdf").expect("write");
    path
}

#[test]
fn should_select_the_file_through_the_file_manager_service() {
    let directory = TempDir::new().expect("tempdir");
    let path = given_a_downloaded_file(&directory);

    let request = FileManagerRequest::showing_file(&path).expect("file exists");

    let arguments = request.gdbus_arguments();
    assert!(arguments.contains(&"org.freedesktop.FileManager1.ShowItems".to_owned()));
    assert!(arguments.iter().any(|argument| argument.ends_with("/my%20report.pdf']")));
}

#[test]
fn should_show_a_folder_through_the_file_manager_service() {
    let directory = TempDir::new().expect("tempdir");

    let request = FileManagerRequest::showing_folder(directory.path()).expect("folder exists");

    assert!(request.gdbus_arguments().contains(&"org.freedesktop.FileManager1.ShowFolders".to_owned()));
}

#[test]
fn should_fall_back_to_the_folder_that_holds_the_file() {
    let directory = TempDir::new().expect("tempdir");
    let path = given_a_downloaded_file(&directory);

    let request = FileManagerRequest::showing_file(&path).expect("file exists");

    assert_eq!(request.fallback_folder(), directory.path());
}

#[test]
fn should_refuse_a_file_that_is_gone() {
    assert!(FileManagerRequest::showing_file(Path::new("/nonexistent/gone.pdf")).is_none());
}

#[test]
fn should_refuse_a_relative_path() {
    assert!(FileManagerRequest::showing_folder(Path::new("downloads")).is_none());
}

#[test]
fn should_pass_the_startup_id_as_a_quoted_empty_string() {
    let directory = TempDir::new().expect("tempdir");

    let request = FileManagerRequest::showing_folder(directory.path()).expect("folder exists");

    assert_eq!(request.gdbus_arguments().last().map(String::as_str), Some("''"));
}

#[test]
fn should_not_let_a_file_name_close_the_quoted_uri_list() {
    let directory = TempDir::new().expect("tempdir");
    let path = directory.path().join("it's'], 'x\\.pdf");
    std::fs::write(&path, b"pdf").expect("write");

    let request = FileManagerRequest::showing_file(&path).expect("file exists");

    let list = &request.gdbus_arguments()[10];
    assert_eq!(list.matches('\'').count(), 2, "only the two delimiters may be quotes: {list}");
    assert!(!list.contains('\\'));
}
