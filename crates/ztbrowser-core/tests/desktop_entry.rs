//! The `.desktop` file that makes an installed web app appear in the desktop's application menu.
//!
//! Page titles come from websites and are hostile input: they end up in a file the desktop executes.

use std::path::Path;

use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::web_app::desktop_entry::DesktopEntry;
use ztbrowser_core::web_app::web_app_catalog::WebAppCatalog;

struct DesktopEntryScenario {
    catalog: WebAppCatalog,
    executable: String,
}

impl DesktopEntryScenario {
    fn given_installed_web_app(name: &str) -> Self {
        let mut catalog = WebAppCatalog::default();
        catalog.install(name, "https://online.mbank.pl/").expect("installable");
        Self { catalog, executable: "/usr/bin/ztbrowser".to_owned() }
    }

    fn with_executable_at(mut self, path: &str) -> Self {
        path.clone_into(&mut self.executable);
        self
    }

    fn when_desktop_entry_is_rendered(&self) -> RenderedDesktopEntry {
        let app = self.catalog.web_apps().next().expect("one web app");
        let catalogue = MessageCatalogue::untranslated();
        let entry = DesktopEntry::new(
            app,
            Path::new(&self.executable),
            Path::new("/home/me/.local/share/ztbrowser/web-apps/online_mbank_pl/icon.png"),
            &catalogue,
        );
        RenderedDesktopEntry { file_name: entry.file_name(), text: entry.render() }
    }
}

struct RenderedDesktopEntry {
    file_name: String,
    text: String,
}

impl RenderedDesktopEntry {
    fn line_starting_with(&self, key: &str) -> &str {
        self.text.lines().find(|line| line.starts_with(key)).unwrap_or_default()
    }

    fn then_reads(&self, expected: &str) {
        assert_eq!(self.text, expected);
    }

    fn then_has_exactly_one_line_starting_with(&self, key: &str) {
        assert_eq!(self.text.lines().filter(|line| line.starts_with(key)).count(), 1, "{}", self.text);
    }
}

#[test]
fn should_describe_web_app_for_the_application_menu() {
    DesktopEntryScenario::given_installed_web_app("mBank").when_desktop_entry_is_rendered().then_reads(
        "[Desktop Entry]\n\
         Type=Application\n\
         Version=1.5\n\
         Name=mBank\n\
         Comment=online.mbank.pl — ZTBrowser web app\n\
         Exec=/usr/bin/ztbrowser --web-app online_mbank_pl\n\
         Icon=/home/me/.local/share/ztbrowser/web-apps/online_mbank_pl/icon.png\n\
         Terminal=false\n\
         Categories=Network;\n\
         StartupNotify=true\n\
         StartupWMClass=io.github.ztbrowser.ZTBrowser.WebApp.online_mbank_pl\n",
    );
}

#[test]
fn should_name_file_after_application_identifier_so_desktop_groups_its_windows() {
    let entry = DesktopEntryScenario::given_installed_web_app("mBank").when_desktop_entry_is_rendered();

    assert_eq!(entry.file_name, "io.github.ztbrowser.ZTBrowser.WebApp.online_mbank_pl.desktop");
}

#[test]
fn should_keep_hostile_title_on_a_single_line() {
    let entry =
        DesktopEntryScenario::given_installed_web_app("Bank\nExec=rm -rf ~\r\n").when_desktop_entry_is_rendered();

    entry.then_has_exactly_one_line_starting_with("Exec=");
}

#[test]
fn should_escape_control_characters_and_backslashes_in_title() {
    let entry = DesktopEntryScenario::given_installed_web_app("Bank\nNext\\Tab\tEnd").when_desktop_entry_is_rendered();

    assert_eq!(entry.line_starting_with("Name="), "Name=Bank\\nNext\\\\Tab\\tEnd");
}

#[test]
fn should_quote_executable_path_containing_spaces_and_shell_characters() {
    let entry = DesktopEntryScenario::given_installed_web_app("mBank")
        .with_executable_at("/home/me/My Apps/$HOME/ztbrowser")
        .when_desktop_entry_is_rendered();

    assert_eq!(
        entry.line_starting_with("Exec="),
        r#"Exec="/home/me/My Apps/\\$HOME/ztbrowser" --web-app online_mbank_pl"#
    );
}
