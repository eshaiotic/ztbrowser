//! The freedesktop.org `.desktop` file that puts an installed web app into the application menu.
//!
//! Spec: <https://specifications.freedesktop.org/desktop-entry-spec/latest/>. The app name comes from a web
//! page and is hostile input: a newline in it would let a site add its own `Exec=` line, which the desktop
//! runs when the user clicks the icon. Every value is therefore escaped as the spec requires, and the
//! executable path is quoted as an `Exec` argument.

use std::path::Path;

use crate::language::message_catalogue::MessageCatalogue;
use crate::web_app::installed_web_app::InstalledWebApp;
use crate::web_app::web_app_id::WebAppId;

pub struct DesktopEntry<'app> {
    app: &'app InstalledWebApp,
    executable: &'app Path,
    icon: &'app Path,
    catalogue: &'app MessageCatalogue,
}

impl<'app> DesktopEntry<'app> {
    /// Command-line switch that starts ZTBrowser as one web app.
    pub const WEB_APP_SWITCH: &'static str = "--web-app";
    /// Characters that force an `Exec` argument to be quoted.
    const RESERVED_IN_EXEC: &'static [char] =
        &[' ', '\t', '\n', '"', '\'', '\\', '>', '<', '~', '|', '&', ';', '$', '*', '?', '#', '(', ')', '`', '='];

    #[must_use]
    pub fn new(
        app: &'app InstalledWebApp,
        executable: &'app Path,
        icon: &'app Path,
        catalogue: &'app MessageCatalogue,
    ) -> Self {
        Self { app, executable, icon, catalogue }
    }

    /// Named after the application id, which lets the desktop match windows to this entry.
    #[must_use]
    pub fn file_name(&self) -> String {
        Self::file_name_for(&self.app.id)
    }

    #[must_use]
    pub fn file_name_for(web_app: &WebAppId) -> String {
        format!("{}.desktop", web_app.application_id())
    }

    #[must_use]
    pub fn render(&self) -> String {
        let lines = [
            "[Desktop Entry]".to_owned(),
            "Type=Application".to_owned(),
            "Version=1.5".to_owned(),
            format!("Name={}", Self::escaped_value(&self.app.name)),
            format!("Comment={}", Self::escaped_value(&self.comment())),
            format!("Exec={}", Self::escaped_value(&self.exec_command())),
            format!("Icon={}", Self::escaped_value(&self.icon.to_string_lossy())),
            "Terminal=false".to_owned(),
            "Categories=Network;".to_owned(),
            "StartupNotify=true".to_owned(),
            format!("StartupWMClass={}", self.app.id.application_id()),
        ];
        let mut text = lines.join("\n");
        text.push('\n');
        text
    }

    /// The menu's tooltip for the app: where it lives and that ZTBrowser runs it.
    fn comment(&self) -> String {
        self.catalogue.text_with("{host} — ZTBrowser web app", &[("host", &self.app.host())])
    }

    /// `<executable> --web-app <id>`; the id is a slug and never needs quoting.
    fn exec_command(&self) -> String {
        let executable = Self::exec_argument(&self.executable.to_string_lossy());
        format!("{executable} {} {}", Self::WEB_APP_SWITCH, self.app.id)
    }

    /// Quotes an `Exec` argument when needed; `%` is doubled because it introduces field codes.
    pub(crate) fn exec_argument(argument: &str) -> String {
        let argument = argument.replace('%', "%%");
        if !argument.contains(Self::RESERVED_IN_EXEC) {
            return argument;
        }
        let mut quoted = String::from('"');
        for character in argument.chars() {
            if matches!(character, '"' | '`' | '$' | '\\') {
                quoted.push('\\');
            }
            quoted.push(character);
        }
        quoted.push('"');
        quoted
    }

    /// Escapes for `string`/`localestring` values; other control characters are dropped.
    pub(crate) fn escaped_value(value: &str) -> String {
        let mut escaped = String::with_capacity(value.len());
        for character in value.chars() {
            match character {
                '\\' => escaped.push_str("\\\\"),
                '\n' => escaped.push_str("\\n"),
                '\t' => escaped.push_str("\\t"),
                '\r' => escaped.push_str("\\r"),
                control if control.is_control() => {}
                other => escaped.push(other),
            }
        }
        escaped
    }
}
