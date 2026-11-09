//! The application-menu entry an AppImage writes for itself.
//!
//! A package installs `io.github.ztbrowser.ZTBrowser.desktop`; an AppImage installs nothing. A Wayland desktop takes a
//! window's icon from the `.desktop` file named like the window's `app_id`, so without an entry the window shows a
//! generic icon. The entry is the packaged one with its program and icon pointed at the files of this machine.

use std::path::Path;
use std::path::PathBuf;

use crate::web_app::desktop_entry::DesktopEntry;

pub struct AppImageMenuEntry<'entry> {
    packaged_entry: &'entry str,
    appimage: &'entry Path,
    icon: &'entry Path,
}

impl<'entry> AppImageMenuEntry<'entry> {
    /// The key of the line that tells the entry apart from one the person wrote, and names the AppImage file: only
    /// marked entries are ever rewritten or removed, and the file they name is the one that must stay trustworthy.
    const MARKER_KEY: &'static str = "X-ZTBrowser-AppImage";
    const GROUP_HEADER: &'static str = "[Desktop Entry]";
    const PACKAGED_PROGRAM: &'static str = "ztbrowser";

    #[must_use]
    pub fn new(packaged_entry: &'entry str, appimage: &'entry Path, icon: &'entry Path) -> Self {
        Self { packaged_entry, appimage, icon }
    }

    /// The AppImage file a rendered entry names, or nothing for an entry that was not rendered here.
    #[must_use]
    pub fn file_named_by(entry: &str) -> Option<PathBuf> {
        let prefix = format!("{}=", Self::MARKER_KEY);
        let value = entry.lines().find_map(|line| line.strip_prefix(prefix.as_str()))?;
        Some(PathBuf::from(Self::unescaped(value)))
    }

    /// Reverses `DesktopEntry::escaped_value` (the desktop entry specification's escapes).
    fn unescaped(value: &str) -> String {
        let mut text = String::with_capacity(value.len());
        let mut characters = value.chars();
        while let Some(character) = characters.next() {
            if character != '\\' {
                text.push(character);
                continue;
            }
            match characters.next() {
                Some('n') => text.push('\n'),
                Some('t') => text.push('\t'),
                Some('r') => text.push('\r'),
                Some('s') => text.push(' '),
                Some(other) => text.push(other),
                None => text.push('\\'),
            }
        }
        text
    }

    /// The packaged entry with `Exec` starting the AppImage file and `Icon` naming the icon file.
    #[must_use]
    pub fn render(&self) -> String {
        let mut text: String =
            self.packaged_entry.lines().map(|line| self.adapted(line)).collect::<Vec<_>>().join("\n");
        text.push('\n');
        text
    }

    fn adapted(&self, line: &str) -> String {
        if line == Self::GROUP_HEADER {
            let named = DesktopEntry::escaped_value(&self.appimage.to_string_lossy());
            return format!("{line}\n{}={named}", Self::MARKER_KEY);
        }
        if let Some(arguments) = line.strip_prefix("Exec=").and_then(|exec| exec.strip_prefix(Self::PACKAGED_PROGRAM)) {
            let program = DesktopEntry::exec_argument(&self.appimage.to_string_lossy());
            return format!("Exec={program}{arguments}");
        }
        if line.starts_with("Icon=") {
            return format!("Icon={}", DesktopEntry::escaped_value(&self.icon.to_string_lossy()));
        }
        line.to_owned()
    }
}
