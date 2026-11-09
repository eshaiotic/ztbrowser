//! The interface font the desktop chose (GNOME's `font-name`, such as `'Adwaita Sans 11'`), so the window's
//! text is drawn in the same face and size as the GTK edition drew it.

use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopFont {
    family: Option<String>,
    point_size: u32,
}

impl DesktopFont {
    /// GNOME's default interface size, used when the desktop names none.
    pub const DEFAULT_POINT_SIZE: u32 = 11;
    const SMALLEST_POINT_SIZE: u32 = 6;
    const LARGEST_POINT_SIZE: u32 = 48;

    /// The font from the value of the setting (`'Cantarell 11'`, quotes optional); a missing or odd value gives
    /// the default size in the toolkit's own family.
    #[must_use]
    pub fn parse(setting: &str) -> Self {
        let text = setting.trim().trim_matches('\'').trim();
        let Some((family, size)) = text.rsplit_once(' ') else { return Self::fallback() };
        match size.parse::<u32>() {
            Ok(size) if !family.trim().is_empty() => Self {
                family: Some(family.trim().to_owned()),
                point_size: size.clamp(Self::SMALLEST_POINT_SIZE, Self::LARGEST_POINT_SIZE),
            },
            _ => Self::fallback(),
        }
    }

    /// The font GNOME's settings name; the default size where the setting cannot be read (no GNOME, no
    /// `gsettings`).
    #[must_use]
    pub fn from_desktop() -> Self {
        Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "font-name"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map_or_else(Self::fallback, |output| Self::parse(&String::from_utf8_lossy(&output.stdout)))
    }

    #[must_use]
    pub fn family(&self) -> Option<&str> {
        self.family.as_deref()
    }

    #[must_use]
    pub fn point_size(&self) -> u32 {
        self.point_size
    }

    /// The font as Qt reads it (`QFont::fromString`), in `default_family` when the desktop names none.
    #[must_use]
    pub fn description(&self, default_family: &str) -> String {
        format!("{},{}", self.family.as_deref().unwrap_or(default_family), self.point_size)
    }

    fn fallback() -> Self {
        Self { family: None, point_size: Self::DEFAULT_POINT_SIZE }
    }
}
