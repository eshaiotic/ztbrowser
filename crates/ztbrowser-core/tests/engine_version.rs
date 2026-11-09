//! Whether the installed engine is recent enough, per kind of window.

use ztbrowser_core::engine::engine_verdict::EngineVerdict;
use ztbrowser_core::engine::engine_version::EngineVersion;
use ztbrowser_core::engine::minimum_engine_version::MinimumEngineVersion;
use ztbrowser_core::engine::window_kind::WindowKind;

struct EngineScenario {
    minimum: MinimumEngineVersion,
}

impl EngineScenario {
    fn given_minimum(version: &str) -> Self {
        Self { minimum: MinimumEngineVersion::new(EngineVersion::parse(version).expect("valid version")) }
    }

    fn when_installed(&self, version: &str, window: WindowKind) -> EngineVerdict {
        self.minimum.verdict(&EngineVersion::parse(version).expect("valid version"), window)
    }
}

#[test]
fn should_run_when_the_engine_is_at_the_minimum() {
    assert_eq!(
        EngineScenario::given_minimum("140.0.7339.0").when_installed("140.0.7339.0", WindowKind::Normal),
        EngineVerdict::Run
    );
}

#[test]
fn should_run_when_the_engine_is_newer() {
    assert_eq!(
        EngineScenario::given_minimum("140.0.7339.0").when_installed("156.0.1.2", WindowKind::Tor),
        EngineVerdict::Run
    );
}

#[test]
fn should_warn_below_the_minimum_in_a_normal_window() {
    assert_eq!(
        EngineScenario::given_minimum("140.0.7339.200").when_installed("140.0.7339.100", WindowKind::Normal),
        EngineVerdict::Warn
    );
}

#[test]
fn should_refuse_below_the_minimum_in_the_tor_window() {
    assert_eq!(
        EngineScenario::given_minimum("140.0.7339.200").when_installed("139.9.9.9", WindowKind::Tor),
        EngineVerdict::Refuse
    );
}

#[test]
fn should_compare_versions_part_by_part_not_as_text() {
    let older = EngineVersion::parse("99.0.0.0").expect("valid");
    let newer = EngineVersion::parse("140.0.0.0").expect("valid");
    assert!(older < newer);
}

#[test]
fn should_treat_missing_trailing_parts_as_zero() {
    assert_eq!(EngineVersion::parse("140"), EngineVersion::parse("140.0.0.0"));
}

#[test]
fn should_reject_text_that_is_not_a_version() {
    assert!(EngineVersion::parse("Chrome/140").is_none());
    assert!(EngineVersion::parse("1.2.3.4.5").is_none());
}

#[test]
fn should_have_a_valid_current_floor() {
    let _ = MinimumEngineVersion::current();
}

#[test]
fn should_print_all_four_parts() {
    assert_eq!(EngineVersion::parse("140.0.7339").expect("valid").to_string(), "140.0.7339.0");
}
