//! The Fire button: what the next start deletes.

use std::fs;

use tempfile::TempDir;
use ztbrowser_core::profile::profile_directories::ProfileDirectories;
use ztbrowser_core::profile::profile_directories::ProfileLocation;
use ztbrowser_core::profile::profile_id::ProfileId;
use ztbrowser_qt::burn_restart::BurnRestart;

use ztbrowser_qt as _;

struct BurnScenario {
    roots: TempDir,
    location: ProfileLocation,
}

impl BurnScenario {
    fn given_a_profile_with_browsing_data_and_settings() -> Self {
        let roots = TempDir::new().expect("roots");
        let location = ProfileDirectories::new(&roots.path().join("data"), &roots.path().join("cache"))
            .for_profile(&ProfileId::default_profile());
        fs::create_dir_all(location.data.join("engine/Cookies")).expect("engine data");
        fs::create_dir_all(location.cache.join("engine")).expect("cache");
        fs::write(location.data.join("unprotected-sites.json"), b"{}").expect("settings");
        Self { roots, location }
    }

    fn burn(&self) -> BurnRestart {
        BurnRestart::new(self.location.clone())
    }
}

#[test]
fn should_keep_everything_until_a_burn_is_requested() {
    let scenario = BurnScenario::given_a_profile_with_browsing_data_and_settings();
    assert!(!scenario.burn().complete_pending().expect("complete"));
    assert!(scenario.location.data.join("engine").exists());
}

#[test]
fn should_delete_the_engine_directory_and_the_cache_after_a_request() {
    let scenario = BurnScenario::given_a_profile_with_browsing_data_and_settings();
    scenario.burn().request().expect("request");
    assert!(scenario.burn().complete_pending().expect("complete"));
    assert!(!scenario.location.data.join("engine").exists() && !scenario.location.cache.exists());
}

#[test]
fn should_keep_the_persons_settings() {
    let scenario = BurnScenario::given_a_profile_with_browsing_data_and_settings();
    scenario.burn().request().expect("request");
    scenario.burn().complete_pending().expect("complete");
    assert!(scenario.location.data.join("unprotected-sites.json").exists());
}

#[test]
fn should_delete_the_download_list_and_the_site_zoom_but_not_the_download_choices() {
    let scenario = BurnScenario::given_a_profile_with_browsing_data_and_settings();
    for file in ["downloads.json", "site-zoom-levels.json", "download-settings.json"] {
        fs::write(scenario.location.data.join(file), b"{}").expect("file");
    }
    scenario.burn().request().expect("request");
    scenario.burn().complete_pending().expect("complete");
    let remaining: Vec<bool> = ["downloads.json", "site-zoom-levels.json", "download-settings.json"]
        .iter()
        .map(|file| scenario.location.data.join(file).exists())
        .collect();
    assert_eq!(remaining, [false, false, true]);
}

#[test]
fn should_burn_only_once_per_request() {
    let scenario = BurnScenario::given_a_profile_with_browsing_data_and_settings();
    scenario.burn().request().expect("request");
    scenario.burn().complete_pending().expect("first");
    fs::create_dir_all(scenario.location.data.join("engine")).expect("new data");
    assert!(!scenario.burn().complete_pending().expect("second"));
    assert!(scenario.location.data.join("engine").exists());
    drop(scenario.roots);
}

/// A program that records the arguments it was started with.
fn recording_program(directory: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let record = directory.join("arguments.txt");
    let program = directory.join("program.sh");
    fs::write(&program, format!("#!/bin/sh\nprintf '%s|' \"$@\" > '{}'\n", record.display())).expect("program");
    fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).expect("executable");
    (program, record)
}

#[test]
fn should_start_the_program_with_its_arguments_once_the_old_process_is_gone() {
    let directory = TempDir::new().expect("directory");
    let (program, record) = recording_program(directory.path());
    let mut finished = std::process::Command::new("true").spawn().expect("true");
    finished.wait().expect("wait");
    let status = BurnRestart::restart_command(finished.id(), 5, &program, &["--flag".to_owned(), "x y".to_owned()])
        .status()
        .expect("script");
    assert!(status.success() && fs::read_to_string(record).expect("record") == "--flag|x y|");
}

#[test]
fn should_start_nothing_while_the_old_process_is_still_running() {
    let directory = TempDir::new().expect("directory");
    let (program, record) = recording_program(directory.path());
    let mut running = std::process::Command::new("sleep").arg("30").spawn().expect("sleep");
    let status = BurnRestart::restart_command(running.id(), 2, &program, &[]).status().expect("script");
    running.kill().expect("kill");
    running.wait().expect("wait");
    assert!(!status.success() && !record.exists());
}
