//! Following Tor's connection progress for the Tor window's connection screen.

use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::tor::bootstrap_status::BootstrapStatus;

struct BootstrapReportScenario {
    report: String,
}

impl BootstrapReportScenario {
    fn given_report(report: &str) -> Self {
        Self { report: report.to_owned() }
    }

    fn when_read(&self) -> BootstrapOutcome {
        BootstrapOutcome { status: BootstrapStatus::from_report(&self.report) }
    }
}

struct BootstrapOutcome {
    status: Option<BootstrapStatus>,
}

impl BootstrapOutcome {
    fn status(&self) -> &BootstrapStatus {
        self.status.as_ref().expect("a bootstrap report")
    }

    fn then_shows(&self, progress: u8, summary: &str) {
        let catalogue = MessageCatalogue::untranslated();
        assert_eq!((self.status().progress(), self.status().summary(&catalogue)), (progress, summary));
    }

    fn then_reports_problem(&self, problem: Option<&str>) {
        assert_eq!(self.status().problem(&MessageCatalogue::untranslated()), problem);
    }

    fn then_shows_in(&self, catalogue: &MessageCatalogue, summary: &str) {
        assert_eq!(self.status().summary(catalogue), summary);
    }

    fn then_is_connected(&self) {
        assert!(self.status().is_complete());
    }

    fn then_is_not_a_bootstrap_report(&self) {
        assert_eq!(self.status, None);
    }
}

#[test]
fn should_show_progress_and_what_tor_is_doing() {
    BootstrapReportScenario::given_report(
        "NOTICE BOOTSTRAP PROGRESS=50 TAG=loading_descriptors SUMMARY=\"Loading relay descriptors\"",
    )
    .when_read()
    .then_shows(50, "Loading relay descriptors");
}

#[test]
fn should_unescape_quoted_summary() {
    BootstrapReportScenario::given_report("NOTICE BOOTSTRAP PROGRESS=5 SUMMARY=\"Connecting to \\\"bridge\\\"\"")
        .when_read()
        .then_shows(5, "Connecting to \"bridge\"");
}

#[test]
fn should_report_the_problem_tor_warns_about() {
    BootstrapReportScenario::given_report(
        "WARN BOOTSTRAP PROGRESS=10 TAG=conn_done SUMMARY=\"Connected\" WARNING=\"Connection refused\" REASON=CONNECTREFUSED RECOMMENDATION=warn",
    )
    .when_read()
    .then_reports_problem(Some("Connection refused"));
}

#[test]
fn should_not_alarm_about_problems_tor_recommends_ignoring() {
    BootstrapReportScenario::given_report("WARN BOOTSTRAP PROGRESS=10 WARNING=\"timeout\" RECOMMENDATION=ignore")
        .when_read()
        .then_reports_problem(None);
}

#[test]
fn should_know_tor_is_connected_at_one_hundred_percent() {
    BootstrapReportScenario::given_report("NOTICE BOOTSTRAP PROGRESS=100 TAG=done SUMMARY=\"Done\"")
        .when_read()
        .then_is_connected();
}

#[test]
fn should_ignore_other_client_status_reports() {
    BootstrapReportScenario::given_report("NOTICE CIRCUIT_ESTABLISHED").when_read().then_is_not_a_bootstrap_report();
}

#[test]
fn should_ignore_progress_beyond_one_hundred_percent() {
    BootstrapReportScenario::given_report("NOTICE BOOTSTRAP PROGRESS=250").when_read().then_is_not_a_bootstrap_report();
}

#[test]
fn should_show_a_known_step_in_the_interface_language_instead_of_tors_english() {
    let catalogue =
        MessageCatalogue::parse("msgid \"Loading relay descriptors\"\nmsgstr \"Ładowanie opisów przekaźników\"\n");

    BootstrapReportScenario::given_report(
        "NOTICE BOOTSTRAP PROGRESS=50 TAG=loading_descriptors SUMMARY=\"Loading relay descriptors\"",
    )
    .when_read()
    .then_shows_in(&catalogue, "Ładowanie opisów przekaźników");
}

#[test]
fn should_fall_back_to_tors_own_words_for_a_step_it_added_later() {
    let catalogue = MessageCatalogue::parse("msgid \"Done\"\nmsgstr \"Gotowe\"\n");

    BootstrapReportScenario::given_report(
        "NOTICE BOOTSTRAP PROGRESS=42 TAG=brand_new_step SUMMARY=\"Doing a new thing\"",
    )
    .when_read()
    .then_shows_in(&catalogue, "Doing a new thing");
}

#[test]
fn should_describe_a_warning_without_text_in_the_interface_language() {
    BootstrapReportScenario::given_report("WARN BOOTSTRAP PROGRESS=10 RECOMMENDATION=warn")
        .when_read()
        .then_reports_problem(Some("Tor reported a problem"));
}
