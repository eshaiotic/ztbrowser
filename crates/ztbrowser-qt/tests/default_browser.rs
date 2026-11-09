//! Whether ZTBrowser opens the web links of other programs, making it do so, and asking the person about it.
//!
//! The desktop's own tool, `xdg-settings`, reads and changes the setting. Every scenario puts a stand-in for that tool
//! in a temporary folder: it answers from a file and records how it was called, so no test ever reads or changes the
//! default browser of the machine that runs it.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use serde_json::Value;
use serde_json::json;
use tempfile::TempDir;
use ztbrowser_qt::default_browser::DefaultBrowser;
use ztbrowser_qt::default_browser::DefaultBrowserChoice;
use ztbrowser_qt::default_browser::DefaultBrowserState;

use ztbrowser_qt as _;

struct DefaultBrowserScenario {
    roots: TempDir,
    tool: PathBuf,
}

impl DefaultBrowserScenario {
    const ENTRY: &'static str = "io.github.ztbrowser.ZTBrowser.desktop";
    const OTHER_ENTRY: &'static str = "brave-browser.desktop";
    /// Answers `get` from the file `current`, stores what `set` names there, refuses `set` while `refuses` exists,
    /// and appends every call to `calls`.
    const TOOL: &'static str = r#"#!/bin/sh
here=$(dirname "$0")
echo "$*" >> "$here/calls"
case "$1" in
  get) cat "$here/current" 2>/dev/null ;;
  set)
    if [ -e "$here/refuses" ]; then
      echo "xdg-settings: cannot write the configuration" >&2
      exit 4
    fi
    printf '%s\n' "$3" > "$here/current"
    ;;
esac
"#;

    fn given_an_installed_ztbrowser_while_another_browser_opens_links() -> Self {
        let roots = TempDir::new().expect("roots");
        let tool = roots.path().join("tool/xdg-settings");
        fs::create_dir_all(tool.parent().expect("tool folder")).expect("tool folder");
        fs::write(&tool, Self::TOOL).expect("tool");
        fs::set_permissions(&tool, fs::Permissions::from_mode(0o755)).expect("executable tool");
        let scenario = Self { roots, tool };
        Self::write_entry(&scenario.system_applications(), Self::ENTRY, "[Desktop Entry]\nName=ZTBrowser\n");
        Self::write_entry(
            &scenario.system_applications(),
            Self::OTHER_ENTRY,
            "[Desktop Entry]\nName=Brave\nExec=brave %U\n\n[Desktop Action new-window]\nName=New Window\n",
        );
        scenario.with_the_desktop_naming(Self::OTHER_ENTRY)
    }

    fn given_a_development_build_without_a_menu_entry() -> Self {
        let scenario = Self::given_an_installed_ztbrowser_while_another_browser_opens_links();
        fs::remove_file(scenario.system_applications().join(Self::ENTRY)).expect("entry removed");
        scenario
    }

    /// The menu entry an AppImage writes for itself, in the person's own applications folder.
    fn given_an_appimage_in_the_menu_while_another_browser_opens_links() -> Self {
        let scenario = Self::given_a_development_build_without_a_menu_entry();
        Self::write_entry(&scenario.person_applications(), Self::ENTRY, "[Desktop Entry]\nName=ZTBrowser\n");
        scenario
    }

    fn with_the_desktop_naming(self, entry: &str) -> Self {
        fs::write(self.roots.path().join("tool/current"), format!("{entry}\n")).expect("current browser");
        self
    }

    fn with_the_other_browser_missing_from_the_menu(self) -> Self {
        fs::remove_file(self.system_applications().join(Self::OTHER_ENTRY)).expect("other entry removed");
        self
    }

    fn with_the_desktop_tool_missing(mut self) -> Self {
        self.tool = self.roots.path().join("tool/missing");
        self
    }

    fn with_the_desktop_refusing_changes(self) -> Self {
        fs::write(self.roots.path().join("tool/refuses"), "").expect("refusal");
        self
    }

    fn person_applications(&self) -> PathBuf {
        self.roots.path().join("home/.local/share/applications")
    }

    fn system_applications(&self) -> PathBuf {
        self.roots.path().join("usr/share/applications")
    }

    fn question_file(&self) -> PathBuf {
        self.roots.path().join("home/.local/share/ztbrowser/default-browser.json")
    }

    fn write_entry(applications: &std::path::Path, name: &str, text: &str) {
        fs::create_dir_all(applications).expect("applications folder");
        fs::write(applications.join(name), text).expect("entry");
    }

    fn browser(&self) -> DefaultBrowser {
        DefaultBrowser::new(self.tool.clone(), vec![self.person_applications(), self.system_applications()])
    }

    /// A process of ZTBrowser: what it asks the desktop, and the person's stored wish to be asked.
    fn choice(&self) -> DefaultBrowserChoice {
        DefaultBrowserChoice::open(self.browser(), &self.question_file())
    }

    fn calls(&self) -> String {
        fs::read_to_string(self.roots.path().join("tool/calls")).unwrap_or_default()
    }

    fn when_the_state_is_read(&self) -> StateOutcome {
        StateOutcome { state: self.browser().state() }
    }

    fn when_ztbrowser_is_made_the_default(self) -> ChangeOutcome {
        let result = self.browser().make_default().map_err(|error| error.to_string());
        ChangeOutcome { result, scenario: self }
    }

    /// The process checks once the window is up, and the window then asks whether to offer the change.
    fn when_windows_ask_whether_to_offer_the_change(&self, windows: usize) -> OfferOutcome {
        let choice = self.choice();
        choice.check().join().expect("check");
        OfferOutcome { offers: (0..windows).map(|_| choice.takes_offer()).collect() }
    }

    fn when_the_person_says_not_to_ask_again_and_ztbrowser_starts_again(&self) -> OfferOutcome {
        self.choice().set_asks_at_start(false);
        self.when_windows_ask_whether_to_offer_the_change(1)
    }

    fn when_the_person_wants_to_be_asked_again_after_saying_no(&self) -> OfferOutcome {
        self.choice().set_asks_at_start(false);
        self.choice().set_asks_at_start(true);
        self.when_windows_ask_whether_to_offer_the_change(1)
    }

    fn when_a_window_asks_before_and_after_the_desktop_answered(&self) -> OfferOutcome {
        let choice = self.choice();
        let before = choice.takes_offer();
        choice.check().join().expect("check");
        OfferOutcome { offers: vec![before, choice.takes_offer()] }
    }

    fn when_the_answer_is_read_after_a_check(&self) -> AnswerOutcome {
        let choice = self.choice();
        choice.check().join().expect("check");
        AnswerOutcome { answer: choice.answer_json() }
    }

    fn when_the_answer_is_read_after_making_ztbrowser_the_default(&self) -> AnswerOutcome {
        let choice = self.choice();
        choice.make_default().join().expect("change");
        AnswerOutcome { answer: choice.answer_json() }
    }

    fn when_the_answer_is_read_before_any_check(&self) -> AnswerOutcome {
        AnswerOutcome { answer: self.choice().answer_json() }
    }
}

struct StateOutcome {
    state: DefaultBrowserState,
}

impl StateOutcome {
    fn then_is(&self, expected: &DefaultBrowserState) {
        assert_eq!(&self.state, expected);
    }
}

struct ChangeOutcome {
    result: Result<(), String>,
    scenario: DefaultBrowserScenario,
}

impl ChangeOutcome {
    fn then_the_desktop_was_asked_to_choose_ztbrowser(&self) {
        let expected = format!("set default-web-browser {}", DefaultBrowserScenario::ENTRY);
        assert!(self.scenario.calls().lines().any(|call| call == expected), "calls: {}", self.scenario.calls());
    }

    fn then_ztbrowser_opens_links(&self) {
        self.scenario.when_the_state_is_read().then_is(&DefaultBrowserState::ThisBrowser);
    }

    fn then_fails_with_the_desktop_message(&self) {
        assert_eq!(self.result, Err("xdg-settings: cannot write the configuration".to_owned()));
    }

    fn then_the_desktop_was_never_asked(&self) {
        assert_eq!(self.scenario.calls(), "");
    }
}

struct OfferOutcome {
    offers: Vec<bool>,
}

impl OfferOutcome {
    fn then_offers(&self, expected: &[bool]) {
        assert_eq!(self.offers, expected);
    }
}

struct AnswerOutcome {
    answer: Value,
}

impl AnswerOutcome {
    fn then_is(&self, expected: &Value) {
        assert_eq!(&self.answer, expected);
    }
}

#[test]
fn should_name_the_other_browser_by_its_menu_name() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .when_the_state_is_read()
        .then_is(&DefaultBrowserState::OtherBrowser(Some("Brave".to_owned())));
}

#[test]
fn should_report_another_browser_without_a_name_when_its_menu_entry_is_missing() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .with_the_other_browser_missing_from_the_menu()
        .when_the_state_is_read()
        .then_is(&DefaultBrowserState::OtherBrowser(None));
}

#[test]
fn should_report_another_browser_without_a_name_when_the_desktop_names_none() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .with_the_desktop_naming("")
        .when_the_state_is_read()
        .then_is(&DefaultBrowserState::OtherBrowser(None));
}

#[test]
fn should_report_ztbrowser_when_the_desktop_names_its_entry() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .with_the_desktop_naming(DefaultBrowserScenario::ENTRY)
        .when_the_state_is_read()
        .then_is(&DefaultBrowserState::ThisBrowser);
}

#[test]
fn should_count_the_menu_entry_an_appimage_wrote_for_itself() {
    DefaultBrowserScenario::given_an_appimage_in_the_menu_while_another_browser_opens_links()
        .when_the_state_is_read()
        .then_is(&DefaultBrowserState::OtherBrowser(Some("Brave".to_owned())));
}

#[test]
fn should_report_a_development_build_as_unregistered() {
    DefaultBrowserScenario::given_a_development_build_without_a_menu_entry()
        .when_the_state_is_read()
        .then_is(&DefaultBrowserState::Unregistered);
}

#[test]
fn should_report_an_unknown_state_when_the_desktop_tool_is_missing() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .with_the_desktop_tool_missing()
        .when_the_state_is_read()
        .then_is(&DefaultBrowserState::Unknown);
}

#[test]
fn should_ask_the_desktop_to_choose_ztbrowser_s_menu_entry() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .when_ztbrowser_is_made_the_default()
        .then_the_desktop_was_asked_to_choose_ztbrowser();
}

#[test]
fn should_open_links_once_made_the_default() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .when_ztbrowser_is_made_the_default()
        .then_ztbrowser_opens_links();
}

#[test]
fn should_fail_with_the_desktop_message_when_the_desktop_refuses() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .with_the_desktop_refusing_changes()
        .when_ztbrowser_is_made_the_default()
        .then_fails_with_the_desktop_message();
}

#[test]
fn should_never_ask_the_desktop_to_choose_a_build_without_a_menu_entry() {
    DefaultBrowserScenario::given_a_development_build_without_a_menu_entry()
        .when_ztbrowser_is_made_the_default()
        .then_the_desktop_was_never_asked();
}

#[test]
fn should_offer_the_change_in_the_first_window_only() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .when_windows_ask_whether_to_offer_the_change(2)
        .then_offers(&[true, false]);
}

#[test]
fn should_not_offer_the_change_when_ztbrowser_already_opens_links() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .with_the_desktop_naming(DefaultBrowserScenario::ENTRY)
        .when_windows_ask_whether_to_offer_the_change(1)
        .then_offers(&[false]);
}

#[test]
fn should_not_offer_the_change_when_the_desktop_cannot_say() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .with_the_desktop_tool_missing()
        .when_windows_ask_whether_to_offer_the_change(1)
        .then_offers(&[false]);
}

#[test]
fn should_not_offer_the_change_to_a_development_build() {
    DefaultBrowserScenario::given_a_development_build_without_a_menu_entry()
        .when_windows_ask_whether_to_offer_the_change(1)
        .then_offers(&[false]);
}

#[test]
fn should_never_offer_the_change_again_once_the_person_said_so() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .when_the_person_says_not_to_ask_again_and_ztbrowser_starts_again()
        .then_offers(&[false]);
}

#[test]
fn should_offer_the_change_again_once_the_person_wants_to_be_asked_again() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .when_the_person_wants_to_be_asked_again_after_saying_no()
        .then_offers(&[true]);
}

#[test]
fn should_keep_the_offer_for_the_window_that_asks_after_the_desktop_answered() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .when_a_window_asks_before_and_after_the_desktop_answered()
        .then_offers(&[false, true]);
}

#[test]
fn should_answer_that_it_is_checking_before_the_desktop_answered() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .when_the_answer_is_read_before_any_check()
        .then_is(&json!({ "state": "checking", "browser": "", "error": "" }));
}

#[test]
fn should_answer_with_the_other_browser_s_name() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .when_the_answer_is_read_after_a_check()
        .then_is(&json!({ "state": "other", "browser": "Brave", "error": "" }));
}

#[test]
fn should_answer_that_a_development_build_cannot_be_chosen() {
    DefaultBrowserScenario::given_a_development_build_without_a_menu_entry()
        .when_the_answer_is_read_after_a_check()
        .then_is(&json!({ "state": "unregistered", "browser": "", "error": "" }));
}

#[test]
fn should_answer_that_the_desktop_cannot_say_when_its_tool_is_missing() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .with_the_desktop_tool_missing()
        .when_the_answer_is_read_after_a_check()
        .then_is(&json!({ "state": "unknown", "browser": "", "error": "" }));
}

#[test]
fn should_answer_that_ztbrowser_opens_links_after_the_change() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .when_the_answer_is_read_after_making_ztbrowser_the_default()
        .then_is(&json!({ "state": "this", "browser": "", "error": "" }));
}

#[test]
fn should_answer_with_the_desktop_message_when_the_change_fails() {
    DefaultBrowserScenario::given_an_installed_ztbrowser_while_another_browser_opens_links()
        .with_the_desktop_refusing_changes()
        .when_the_answer_is_read_after_making_ztbrowser_the_default()
        .then_is(&json!({
            "state": "other",
            "browser": "Brave",
            "error": "xdg-settings: cannot write the configuration"
        }));
}
