//! Where downloads go and whether the person is asked first.

use std::path::Path;
use std::path::PathBuf;

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::download::download_settings::DownloadSettings;

struct DownloadSettingsScenario {
    settings: DownloadSettings,
}

impl DownloadSettingsScenario {
    fn given_stored(document: &Value) -> Self {
        Self { settings: DownloadSettings::from_json(document) }
    }

    fn when_asking_is(mut self, asks: bool) -> Self {
        self.settings = self.settings.with_asking(asks);
        self
    }

    fn when_folder_is(mut self, folder: &str) -> Self {
        self.settings = self.settings.with_folder(PathBuf::from(folder));
        self
    }

    fn after_saving_and_loading(self) -> Self {
        Self { settings: DownloadSettings::from_json(&self.settings.to_json()) }
    }

    fn then_asks_where_to_save(&self, expected: bool) {
        assert_eq!(self.settings.asks_where_to_save(), expected);
    }

    fn then_saves_into(&self, expected: &str) {
        let default = Path::new("/home/someone/Downloads");
        assert_eq!(self.settings.folder_or(default), Path::new(expected));
    }
}

#[test]
fn should_ask_where_to_save_until_the_person_says_otherwise() {
    DownloadSettingsScenario::given_stored(&Value::Null).then_asks_where_to_save(true);
}

#[test]
fn should_save_into_the_default_folder_until_the_person_picks_one() {
    DownloadSettingsScenario::given_stored(&Value::Null).then_saves_into("/home/someone/Downloads");
}

#[test]
fn should_keep_the_choice_not_to_ask_across_restarts() {
    DownloadSettingsScenario::given_stored(&Value::Null)
        .when_asking_is(false)
        .after_saving_and_loading()
        .then_asks_where_to_save(false);
}

#[test]
fn should_keep_the_chosen_folder_across_restarts() {
    DownloadSettingsScenario::given_stored(&Value::Null)
        .when_folder_is("/data/files")
        .after_saving_and_loading()
        .then_saves_into("/data/files");
}

#[test]
fn should_ignore_a_stored_folder_that_is_not_an_absolute_path() {
    DownloadSettingsScenario::given_stored(&json!({"folder": "relative/dir"}))
        .then_saves_into("/home/someone/Downloads");
}

#[test]
fn should_ignore_a_stored_value_of_the_wrong_kind() {
    DownloadSettingsScenario::given_stored(&json!({"asksWhereToSave": "no", "folder": 7}))
        .then_asks_where_to_save(true);
}
