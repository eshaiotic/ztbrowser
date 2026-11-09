//! Reading the command line: which profile or web app to start, and which addresses to open.

use ztbrowser_core::launch::launch_request::LaunchRequest;
use ztbrowser_core::launch::launch_request::LaunchRequestError;

struct CommandLineScenario {
    arguments: Vec<String>,
}

impl CommandLineScenario {
    fn given_arguments(arguments: &[&str]) -> Self {
        Self { arguments: arguments.iter().map(|argument| (*argument).to_owned()).collect() }
    }

    fn when_parsed(&self) -> LaunchOutcome {
        LaunchOutcome { parsed: LaunchRequest::parse(&self.arguments) }
    }
}

struct LaunchOutcome {
    parsed: Result<LaunchRequest, LaunchRequestError>,
}

impl LaunchOutcome {
    fn request(&self) -> &LaunchRequest {
        self.parsed.as_ref().expect("valid command line")
    }

    fn then_runs_as(&self, application_id: &str, profile_id: &str) {
        let request = self.request();
        let profile = request.profile_id().map(|profile| profile.as_str().to_owned());
        assert_eq!((request.application_id().as_str(), profile.as_deref()), (application_id, Some(profile_id)));
    }

    fn then_runs_tor_window_without_profile(&self) {
        let request = self.request();
        assert_eq!(
            (request.application_id().as_str(), request.profile_id(), request.is_tor_window()),
            ("io.github.ztbrowser.ZTBrowser.Tor", None, true)
        );
    }

    fn then_opens(&self, addresses: &[&str]) {
        assert_eq!(self.request().addresses(), addresses);
    }

    fn then_starts_web_app(&self, web_app_id: &str) {
        assert_eq!(self.request().web_app_id().map(|id| id.as_str().to_owned()), Some(web_app_id.to_owned()));
    }

    fn then_is_rejected_with(&self, error: &LaunchRequestError) {
        assert_eq!(self.parsed.as_ref().err(), Some(error));
    }
}

#[test]
fn should_start_default_profile_browser_without_arguments() {
    CommandLineScenario::given_arguments(&[]).when_parsed().then_runs_as("io.github.ztbrowser.ZTBrowser", "default");
}

#[test]
fn should_open_every_address_given_on_the_command_line() {
    CommandLineScenario::given_arguments(&["https://duckduckgo.com/", "mbank.pl"])
        .when_parsed()
        .then_opens(&["https://duckduckgo.com/", "mbank.pl"]);
}

#[test]
fn should_run_each_profile_as_its_own_application() {
    CommandLineScenario::given_arguments(&["--profile", "praca", "https://jira.example.com/"])
        .when_parsed()
        .then_runs_as("io.github.ztbrowser.ZTBrowser.Profile.praca", "praca");
}

#[test]
fn should_start_installed_web_app_in_its_own_profile() {
    let outcome = CommandLineScenario::given_arguments(&["--web-app", "online_mbank_pl"]).when_parsed();

    outcome.then_starts_web_app("online_mbank_pl");
    outcome.then_runs_as("io.github.ztbrowser.ZTBrowser.WebApp.online_mbank_pl", "app_online_mbank_pl");
}

#[test]
fn should_reject_switch_without_value() {
    CommandLineScenario::given_arguments(&["--profile"])
        .when_parsed()
        .then_is_rejected_with(&LaunchRequestError::MissingValue("--profile".to_owned()));
}

#[test]
fn should_reject_unsafe_identifiers() {
    CommandLineScenario::given_arguments(&["--web-app", "../../.ssh"])
        .when_parsed()
        .then_is_rejected_with(&LaunchRequestError::InvalidIdentifier("../../.ssh".to_owned()));
}

#[test]
fn should_reject_opening_a_web_app_profile_as_a_browser() {
    CommandLineScenario::given_arguments(&["--profile", "app_online_mbank_pl"])
        .when_parsed()
        .then_is_rejected_with(&LaunchRequestError::ReservedProfile("app_online_mbank_pl".to_owned()));
}

#[test]
fn should_reject_web_app_combined_with_profile_or_addresses() {
    CommandLineScenario::given_arguments(&["--web-app", "gmail", "https://evil.com/"])
        .when_parsed()
        .then_is_rejected_with(&LaunchRequestError::ConflictingTargets);
}

#[test]
fn should_reject_unknown_switches() {
    CommandLineScenario::given_arguments(&["--remote-debugging-port=9222"])
        .when_parsed()
        .then_is_rejected_with(&LaunchRequestError::UnknownSwitch("--remote-debugging-port=9222".to_owned()));
}

#[test]
fn should_start_the_tor_window_in_its_own_process_without_a_profile() {
    CommandLineScenario::given_arguments(&["--tor"]).when_parsed().then_runs_tor_window_without_profile();
}

#[test]
fn should_open_addresses_in_the_tor_window() {
    CommandLineScenario::given_arguments(&["--tor", "http://abcdefgh.onion/"])
        .when_parsed()
        .then_opens(&["http://abcdefgh.onion/"]);
}

#[test]
fn should_reject_tor_window_combined_with_a_profile_or_a_web_app() {
    for arguments in [["--tor", "--profile", "work"].as_slice(), ["--tor", "--web-app", "gmail"].as_slice()] {
        CommandLineScenario::given_arguments(arguments)
            .when_parsed()
            .then_is_rejected_with(&LaunchRequestError::ConflictingTargets);
    }
}
