//! The bar that offers to make ZTBrowser the default browser: shown once the desktop names another browser, never
//! when the person does not want to be asked, and nothing changes until the person picks "Make Default".
//!
//! The test runs `qmltestrunner` on the real `DefaultBrowserBar.qml` with a stand-in for the Rust bridge, so the
//! desktop's real default browser is never read or changed.

use std::fs;
use std::path::Path;
use std::process::Command;

struct DefaultBrowserBarScenario {
    directory: tempfile::TempDir,
}

impl DefaultBrowserBarScenario {
    const TEST_RUNNER: &'static str = "/usr/lib/qt6/bin/qmltestrunner";

    /// Given: the bar's QML files next to the test that drives them.
    fn given_the_bars_qml_files() -> Self {
        let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let directory = tempfile::TempDir::new().expect("scratch directory");
        for (from, name) in [
            ("qml/DefaultBrowserBar.qml", "DefaultBrowserBar.qml"),
            ("qml/PillButton.qml", "PillButton.qml"),
            ("qml/IconButton.qml", "IconButton.qml"),
            ("qml/TintedIcon.qml", "TintedIcon.qml"),
            ("tests/qml/tst_default_browser_bar.qml", "tst_default_browser_bar.qml"),
        ] {
            fs::copy(crate_root.join(from), directory.path().join(name)).expect("a QML file");
        }
        Self { directory }
    }

    /// When: the test runs offscreen. `None` when this system has no test runner.
    fn when_run(&self) -> Option<(bool, String)> {
        if !Path::new(Self::TEST_RUNNER).exists() {
            return None;
        }
        let output = Command::new(Self::TEST_RUNNER)
            .arg("-input")
            .arg(self.directory.path().join("tst_default_browser_bar.qml"))
            .env("QT_QPA_PLATFORM", "offscreen")
            .env("QT_QUICK_CONTROLS_STYLE", "Fusion")
            .output()
            .expect("the test runner starts");
        let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        Some((output.status.success(), text))
    }
}

#[test]
fn should_offer_the_change_only_as_the_person_wishes() {
    let scenario = DefaultBrowserBarScenario::given_the_bars_qml_files();
    if let Some((passed, report)) = scenario.when_run() {
        assert!(passed, "{report}");
    }
}
