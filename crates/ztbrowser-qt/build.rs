use cxx_qt_build::CxxQtBuilder;
use cxx_qt_build::QmlModule;

fn main() {
    CxxQtBuilder::new_qml_module(QmlModule::new("io.ztbrowser.browser").qml_files([
        "qml/Main.qml",
        "qml/AboutDialog.qml",
        "qml/ConfirmDialog.qml",
        "qml/DefaultBrowserBar.qml",
        "qml/AdwaitaSwitch.qml",
        "qml/BrowserTab.qml",
        "qml/DownloadRow.qml",
        "qml/DownloadsHub.qml",
        "qml/DownloadsManager.qml",
        "qml/DownloadsPopover.qml",
        "qml/FindBar.qml",
        "qml/InstallAppDialog.qml",
        "qml/IconButton.qml",
        "qml/MainMenu.qml",
        "qml/PillButton.qml",
        "qml/PreferencesDialog.qml",
        "qml/PreferencesGroup.qml",
        "qml/PreferencesRow.qml",
        "qml/ProfilesPopup.qml",
        "qml/SegmentedChoice.qml",
        "qml/TabStrip.qml",
        "qml/TintedIcon.qml",
        "qml/WebAppInstallation.qml",
        "qml/WindowControls.qml",
    ]))
    .qrc("icons.qrc")
    .files(["src/browser_bridge.rs", "src/guard_bridge.rs", "src/profile_bridge.rs", "src/web_app_bridge.rs"])
    .qt_module("Quick")
    .qt_module("Gui")
    .build();
}
