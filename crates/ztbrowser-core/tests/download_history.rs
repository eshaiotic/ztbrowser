//! The most recent downloads a profile shows in its downloads list.

use std::path::Path;

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::download::download_history::DownloadEntry;
use ztbrowser_core::download::download_history::DownloadHistory;
use ztbrowser_core::download::download_history::DownloadId;
use ztbrowser_core::download::download_history::DownloadState;

struct DownloadHistoryScenario {
    downloads: DownloadHistory,
    last: Option<DownloadId>,
}

impl DownloadHistoryScenario {
    fn given_nothing_downloaded() -> Self {
        Self { downloads: DownloadHistory::new(), last: None }
    }

    fn given_stored(document: &Value) -> Self {
        Self { downloads: DownloadHistory::from_json(document), last: None }
    }

    fn when_started(mut self, path: &str) -> Self {
        self.last = Some(self.downloads.begin(Path::new(path).to_path_buf()));
        self
    }

    fn when_finished(mut self) -> Self {
        self.downloads.finish(self.last.expect("a download was started"));
        self
    }

    fn when_failed(mut self) -> Self {
        self.downloads.fail(self.last.expect("a download was started"));
        self
    }

    fn when_cancelled(mut self) -> Self {
        self.downloads.discard(self.last.expect("a download was started"));
        self
    }

    fn when_received(mut self, received: u64, total: Option<u64>) -> Self {
        self.downloads.progress(self.last.expect("a download was started"), received, total);
        self
    }

    fn when_browsing_data_is_burned(mut self) -> Self {
        self.downloads.forget_all();
        self
    }

    fn after_saving_and_loading(self) -> Self {
        Self { downloads: DownloadHistory::from_json(&self.downloads.to_json()), last: None }
    }

    fn names(&self) -> Vec<String> {
        self.downloads.entries().iter().map(|entry| entry.name().to_owned()).collect()
    }

    fn newest_id(&self) -> Option<DownloadId> {
        self.downloads.entries().first().map(DownloadEntry::id)
    }

    fn when_removed(mut self, name: &str) -> Self {
        let id =
            self.downloads.entries().iter().find(|entry| entry.name() == name).map(DownloadEntry::id).expect("listed");
        self.downloads.discard(id);
        self
    }

    fn then_window_shows(&self, cleared_through: Option<DownloadId>, count: usize, expected: &[&str]) {
        let shown: Vec<&str> =
            self.downloads.latest_after(cleared_through, count).into_iter().map(DownloadEntry::name).collect();
        assert_eq!(shown, expected);
    }

    fn then_lists(&self, expected: &[&str]) {
        assert_eq!(self.names(), expected);
    }

    fn then_newest_is(&self, expected: &DownloadState) {
        assert_eq!(self.downloads.entries().first().map(DownloadEntry::state), Some(expected));
    }
}

#[test]
fn should_list_the_newest_download_first() {
    DownloadHistoryScenario::given_nothing_downloaded()
        .when_started("/d/a.zip")
        .when_started("/d/b.zip")
        .then_lists(&["b.zip", "a.zip"]);
}

#[test]
fn should_keep_only_the_newest_downloads_up_to_the_capacity() {
    let mut scenario = DownloadHistoryScenario::given_nothing_downloaded();
    for number in 0..DownloadHistory::CAPACITY + 2 {
        scenario = scenario.when_started(&format!("/d/{number}.bin"));
    }
    assert_eq!(
        (scenario.names().len(), scenario.names().first().cloned(), scenario.names().last().cloned()),
        (DownloadHistory::CAPACITY, Some("501.bin".to_owned()), Some("2.bin".to_owned()))
    );
}

#[test]
fn should_show_only_the_latest_few_downloads_in_a_window() {
    let mut scenario = DownloadHistoryScenario::given_nothing_downloaded();
    for name in ["1", "2", "3", "4", "5", "6", "7"] {
        scenario = scenario.when_started(&format!("/d/{name}.bin"));
    }
    scenario.then_window_shows(None, 5, &["7.bin", "6.bin", "5.bin", "4.bin", "3.bin"]);
}

#[test]
fn should_show_a_clean_list_after_a_window_cleared_it_and_only_later_downloads_after_that() {
    let cleared = DownloadHistoryScenario::given_nothing_downloaded()
        .when_started("/d/old-1.bin")
        .when_finished()
        .when_started("/d/old-2.bin")
        .when_finished();
    let cutoff = cleared.newest_id();
    let scenario = cleared.when_started("/d/new.bin");
    scenario.then_window_shows(cutoff, 5, &["new.bin"]);
    scenario.then_lists(&["new.bin", "old-2.bin", "old-1.bin"]);
}

#[test]
fn should_remove_one_download_from_the_history_and_keep_the_others() {
    DownloadHistoryScenario::given_nothing_downloaded()
        .when_started("/d/a.bin")
        .when_started("/d/b.bin")
        .when_removed("a.bin")
        .then_lists(&["b.bin"]);
}

#[test]
fn should_track_how_much_of_a_download_has_arrived() {
    DownloadHistoryScenario::given_nothing_downloaded()
        .when_started("/d/a.iso")
        .when_received(512, Some(2048))
        .then_newest_is(&DownloadState::InProgress { received: 512, total: Some(2048) });
}

#[test]
fn should_mark_a_download_finished() {
    DownloadHistoryScenario::given_nothing_downloaded()
        .when_started("/d/a.iso")
        .when_finished()
        .then_newest_is(&DownloadState::Finished);
}

#[test]
fn should_mark_a_download_failed() {
    DownloadHistoryScenario::given_nothing_downloaded()
        .when_started("/d/a.iso")
        .when_failed()
        .then_newest_is(&DownloadState::Failed);
}

#[test]
fn should_drop_a_download_the_person_cancelled() {
    DownloadHistoryScenario::given_nothing_downloaded().when_started("/d/a.iso").when_cancelled().then_lists(&[]);
}

#[test]
fn should_keep_only_finished_downloads_across_restarts() {
    DownloadHistoryScenario::given_nothing_downloaded()
        .when_started("/d/done.pdf")
        .when_finished()
        .when_started("/d/broken.pdf")
        .when_failed()
        .when_started("/d/halfway.pdf")
        .after_saving_and_loading()
        .then_lists(&["done.pdf"]);
}

#[test]
fn should_restore_stored_downloads_as_finished_and_in_the_same_order() {
    DownloadHistoryScenario::given_stored(&json!({"downloads": [{"path": "/d/new.txt"}, {"path": "/d/old.txt"}]}))
        .then_lists(&["new.txt", "old.txt"]);
}

#[test]
fn should_ignore_stored_entries_without_an_absolute_path() {
    DownloadHistoryScenario::given_stored(
        &json!({"downloads": [{"path": "rel/a.txt"}, {"name": "x"}, 5, {"path": "/d/ok.txt"}]}),
    )
    .then_lists(&["ok.txt"]);
}

#[test]
fn should_forget_every_download_when_browsing_data_is_burned() {
    DownloadHistoryScenario::given_nothing_downloaded()
        .when_started("/home/me/Downloads/done.pdf")
        .when_finished()
        .when_started("/home/me/Downloads/broken.zip")
        .when_failed()
        .when_browsing_data_is_burned()
        .then_lists(&[]);
}

#[test]
fn should_forget_a_running_download_too_and_not_list_it_when_it_finishes() {
    DownloadHistoryScenario::given_nothing_downloaded()
        .when_started("/home/me/Downloads/big.iso")
        .when_browsing_data_is_burned()
        .when_finished()
        .then_lists(&[]);
}

#[test]
fn should_keep_showing_a_download_that_is_still_running_after_the_list_was_cleared() {
    let cleared = DownloadHistoryScenario::given_nothing_downloaded().when_started("/d/big.iso");
    let cutoff = cleared.newest_id();
    cleared.then_window_shows(cutoff, 5, &["big.iso"]);
}
