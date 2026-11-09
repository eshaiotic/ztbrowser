//! Where downloads go, what the list shows, and what survives a restart.

use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;
use tempfile::TempDir;
use ztbrowser_qt::download_center::DownloadCenter;

// The bridge's symbols must be linked into every test binary of this crate.
use ztbrowser_qt as _;

struct DownloadsScenario {
    profile: TempDir,
    downloads_folder: TempDir,
}

impl DownloadsScenario {
    fn given_a_profile_with_an_empty_downloads_folder() -> Self {
        Self { profile: TempDir::new().expect("profile"), downloads_folder: TempDir::new().expect("folder") }
    }

    fn center(&self) -> DownloadCenter {
        DownloadCenter::open(self.profile.path(), self.downloads_folder.path().to_path_buf())
    }

    fn file_in_downloads(&self, name: &str) -> PathBuf {
        let path = self.downloads_folder.path().join(name);
        fs::write(&path, b"data").expect("file");
        path
    }
}

fn names(rows: &Value) -> Vec<&str> {
    rows.as_array().expect("rows").iter().filter_map(|row| row["name"].as_str()).collect()
}

#[test]
fn should_propose_the_download_folder_and_a_cleaned_name() {
    let scenario = DownloadsScenario::given_a_profile_with_an_empty_downloads_folder();
    let path = scenario.center().proposed_path("../../evil.sh");
    assert_eq!(path, scenario.downloads_folder.path().join("evil.sh"));
}

#[test]
fn should_number_the_name_when_a_file_with_it_exists() {
    let scenario = DownloadsScenario::given_a_profile_with_an_empty_downloads_folder();
    scenario.file_in_downloads("report.pdf");
    let path = scenario.center().proposed_path("report.pdf");
    assert_eq!(path, scenario.downloads_folder.path().join("report (2).pdf"));
}

#[test]
fn should_ask_where_to_save_until_the_person_says_otherwise() {
    let scenario = DownloadsScenario::given_a_profile_with_an_empty_downloads_folder();
    let mut center = scenario.center();
    assert!(center.asks_where_to_save());
    center.set_asks_where_to_save(false);
    assert!(!scenario.center().asks_where_to_save());
}

#[test]
fn should_remember_the_folder_the_person_chose() {
    let scenario = DownloadsScenario::given_a_profile_with_an_empty_downloads_folder();
    let chosen = scenario.profile.path().join("elsewhere");
    scenario.center().set_folder(chosen.clone());
    assert_eq!(scenario.center().folder(), chosen);
}

#[test]
fn should_list_a_finished_download_after_the_browser_restarts() {
    let scenario = DownloadsScenario::given_a_profile_with_an_empty_downloads_folder();
    let mut center = scenario.center();
    let path = scenario.file_in_downloads("a.zip");
    let id = center.begin(path);
    center.finish(id);
    assert_eq!(names(&scenario.center().rows_json(None, DownloadCenter::LIST_LENGTH)), ["a.zip"]);
}

#[test]
fn should_not_keep_a_download_that_was_still_running() {
    let scenario = DownloadsScenario::given_a_profile_with_an_empty_downloads_folder();
    let mut center = scenario.center();
    let id = center.begin(scenario.downloads_folder.path().join("big.iso"));
    center.progress(id, 10, Some(100));
    center.remove(id);
    assert_eq!(names(&scenario.center().rows_json(None, DownloadCenter::LIST_LENGTH)), Vec::<&str>::new());
}

#[test]
fn should_report_a_finished_file_that_was_deleted_as_missing() {
    let scenario = DownloadsScenario::given_a_profile_with_an_empty_downloads_folder();
    let mut center = scenario.center();
    let path = scenario.file_in_downloads("gone.txt");
    let id = center.begin(path.clone());
    center.finish(id);
    fs::remove_file(&path).expect("delete");
    assert_eq!(center.rows_json(None, 1)[0]["missing"], Value::Bool(true));
}

#[test]
fn should_show_only_the_latest_five_rows() {
    let scenario = DownloadsScenario::given_a_profile_with_an_empty_downloads_folder();
    let mut center = scenario.center();
    for number in 0..7 {
        let id = center.begin(Path::new("/downloads").join(format!("{number}.bin")));
        center.finish(id);
    }
    assert_eq!(
        names(&center.rows_json(None, DownloadCenter::LIST_LENGTH)),
        ["6.bin", "5.bin", "4.bin", "3.bin", "2.bin"]
    );
}

#[test]
fn should_forget_everything_when_the_history_is_cleared() {
    let scenario = DownloadsScenario::given_a_profile_with_an_empty_downloads_folder();
    let mut center = scenario.center();
    let id = center.begin(scenario.file_in_downloads("x.bin"));
    center.finish(id);
    center.clear_history();
    assert_eq!(names(&scenario.center().rows_json(None, 10)), Vec::<&str>::new());
}
