//! The icons of the header bar's buttons stay visible on a dark bar, whichever Qt Quick Controls style draws them.
//!
//! Plasma 6.8's Breeze style paints a button's icon with `Kirigami.Icon`, which applies `icon.color` to mask icons
//! only, so ZTBrowser's plain SVG icons stayed dark on the dark bar. `IconButton` therefore draws the icon itself, in
//! the theme's text colour, and gives the style nothing to paint. The test runs `qmltestrunner` on the real
//! `IconButton.qml` under each style that is installed (the Breeze style on Plasma systems).

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

struct IconButtonScenario {
    directory: tempfile::TempDir,
}

impl IconButtonScenario {
    const TEST_RUNNER: &'static str = "/usr/lib/qt6/bin/qmltestrunner";
    const BREEZE_STYLE_DIRECTORY: &'static str = "/usr/lib/qt6/qml/org/kde/breeze";

    /// Given: the button's QML files next to the test that looks at them.
    fn given_the_buttons_qml_files() -> Self {
        let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let directory = tempfile::TempDir::new().expect("scratch directory");
        for (from, name) in [
            ("qml/IconButton.qml", "IconButton.qml"),
            ("qml/TintedIcon.qml", "TintedIcon.qml"),
            ("tests/qml/tst_icon_button.qml", "tst_icon_button.qml"),
        ] {
            fs::copy(crate_root.join(from), directory.path().join(name)).expect("a QML file");
        }
        Self { directory }
    }

    /// When: the test runs, offscreen, with the given Controls style. `None` when this system has no test runner.
    fn when_run_with_style(&self, style: &str) -> Option<(bool, String)> {
        if !PathBuf::from(Self::TEST_RUNNER).exists() {
            return None;
        }
        let output = Command::new(Self::TEST_RUNNER)
            .arg("-input")
            .arg(self.directory.path().join("tst_icon_button.qml"))
            .env("QT_QPA_PLATFORM", "offscreen")
            .env("QT_QUICK_CONTROLS_STYLE", style)
            .output()
            .expect("the test runner starts");
        let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        Some((output.status.success(), text))
    }
}

#[test]
fn should_draw_the_icon_itself_on_the_fusion_style() {
    let scenario = IconButtonScenario::given_the_buttons_qml_files();
    if let Some((passed, report)) = scenario.when_run_with_style("Fusion") {
        assert!(passed, "{report}");
    }
}

#[test]
fn should_draw_the_icon_itself_on_the_basic_style() {
    let scenario = IconButtonScenario::given_the_buttons_qml_files();
    if let Some((passed, report)) = scenario.when_run_with_style("Basic") {
        assert!(passed, "{report}");
    }
}

#[test]
fn should_draw_the_icon_itself_on_the_breeze_style() {
    if !Path::new(IconButtonScenario::BREEZE_STYLE_DIRECTORY).is_dir() {
        return;
    }
    let scenario = IconButtonScenario::given_the_buttons_qml_files();
    if let Some((passed, report)) = scenario.when_run_with_style("org.kde.breeze") {
        assert!(passed, "{report}");
    }
}
