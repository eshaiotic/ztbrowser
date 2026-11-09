//! The profile's choices the preferences window edits: which toolbar buttons show and whether the mouse wheel
//! scrolls smoothly. Each is kept in its own file of the profile, as in the GTK edition.

use std::path::Path;

use ztbrowser_core::profile::address_bar_preferences::AddressBarPreferences;
use ztbrowser_core::profile::tab_placement::TabPlacement;
use ztbrowser_core::profile::tab_placement::TabPlacementPreferences;
use ztbrowser_core::profile::toolbar_layout::ToolbarButton;
use ztbrowser_core::profile::toolbar_layout::ToolbarLayout;
use ztbrowser_core::profile::window_controls_style::WindowControlsPreferences;
use ztbrowser_core::profile::window_controls_style::WindowControlsStyle;
use ztbrowser_core::scrolling::scrolling_preferences::ScrollingPreferences;
use ztbrowser_core::storage::owned_json_file::OwnedJsonFile;

pub struct ProfilePreferences {
    toolbar: ToolbarLayout,
    toolbar_file: OwnedJsonFile,
    scrolling: ScrollingPreferences,
    scrolling_file: OwnedJsonFile,
    address_bar: AddressBarPreferences,
    address_bar_file: OwnedJsonFile,
    tab_placement: TabPlacementPreferences,
    tab_placement_file: OwnedJsonFile,
    window_controls: WindowControlsPreferences,
    window_controls_file: OwnedJsonFile,
}

impl ProfilePreferences {
    #[must_use]
    pub fn open(directory: &Path) -> Self {
        let toolbar_file = OwnedJsonFile::new(directory.join("toolbar.json"));
        let scrolling_file = OwnedJsonFile::new(directory.join("scrolling.json"));
        let address_bar_file = OwnedJsonFile::new(directory.join("address-bar.json"));
        let tab_placement_file = OwnedJsonFile::new(directory.join("tab-placement.json"));
        let window_controls_file = OwnedJsonFile::new(directory.join("window-controls.json"));
        Self {
            toolbar: toolbar_file.read().map(|document| ToolbarLayout::from_json(&document)).unwrap_or_default(),
            scrolling: scrolling_file
                .read()
                .map(|document| ScrollingPreferences::from_json(&document))
                .unwrap_or_default(),
            address_bar: address_bar_file
                .read()
                .map(|document| AddressBarPreferences::from_json(&document))
                .unwrap_or_default(),
            tab_placement: tab_placement_file
                .read()
                .map(|document| TabPlacementPreferences::from_json(&document))
                .unwrap_or_default(),
            window_controls: window_controls_file
                .read()
                .map(|document| WindowControlsPreferences::from_json(&document))
                .unwrap_or_default(),
            toolbar_file,
            scrolling_file,
            address_bar_file,
            tab_placement_file,
            window_controls_file,
        }
    }

    /// Whether the toolbar shows the button stored as `id` (`reload`); a name nobody offers counts as shown.
    #[must_use]
    pub fn shows_toolbar_button(&self, id: &str) -> bool {
        Self::button_named(id).is_none_or(|button| self.toolbar.shows(button))
    }

    pub fn set_toolbar_button_shown(&mut self, id: &str, is_shown: bool) {
        let Some(button) = Self::button_named(id) else { return };
        self.toolbar.set_shown(button, is_shown);
        // A choice that cannot be written still applies until the browser closes.
        let _ = self.toolbar_file.write(&self.toolbar.to_json());
    }

    #[must_use]
    pub fn smooth_wheel(&self) -> bool {
        self.scrolling.smooth_wheel()
    }

    pub fn set_smooth_wheel(&mut self, is_enabled: bool) {
        self.scrolling.set_smooth_wheel(is_enabled);
        let _ = self.scrolling_file.write(&self.scrolling.to_json());
    }

    /// Whether the address bar takes all the room the toolbar leaves it, not a short centred bar.
    #[must_use]
    pub fn wide_address_bar(&self) -> bool {
        self.address_bar.is_wide()
    }

    pub fn set_wide_address_bar(&mut self, is_wide: bool) {
        self.address_bar.set_wide(is_wide);
        let _ = self.address_bar_file.write(&self.address_bar.to_json());
    }

    /// Where the tabs sit, as the identifier the interface uses (`top`, `bottom`, `left`, `right`).
    #[must_use]
    pub fn tab_placement(&self) -> &'static str {
        self.tab_placement.placement().id()
    }

    /// A name nobody offers changes nothing.
    pub fn set_tab_placement(&mut self, id: &str) {
        let Some(placement) = TabPlacement::from_id(id) else { return };
        self.tab_placement.set_placement(placement);
        let _ = self.tab_placement_file.write(&self.tab_placement.to_json());
    }

    /// How the window buttons look, as the identifier the interface uses (`default`, `macos`).
    #[must_use]
    pub fn window_controls_style(&self) -> &'static str {
        self.window_controls.style().id()
    }

    /// A name nobody offers changes nothing.
    pub fn set_window_controls_style(&mut self, id: &str) {
        let Some(style) = WindowControlsStyle::from_id(id) else { return };
        self.window_controls.set_style(style);
        let _ = self.window_controls_file.write(&self.window_controls.to_json());
    }

    fn button_named(id: &str) -> Option<ToolbarButton> {
        ToolbarButton::ALL.into_iter().find(|button| button.id() == id)
    }
}
