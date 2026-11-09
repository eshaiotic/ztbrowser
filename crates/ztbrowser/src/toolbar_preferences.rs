//! The profile's choice of toolbar buttons (`toolbar.json`), shared by every browser window of the process.
//!
//! The Tor window belongs to no profile: its preferences are kept in a discarded file, so it shows every
//! button and stores nothing.

use std::cell::RefCell;

use ztbrowser_core::profile::toolbar_layout::ToolbarButton;
use ztbrowser_core::profile::toolbar_layout::ToolbarLayout;

use crate::stored_file::StoredFile;

pub struct ToolbarPreferences {
    file: StoredFile,
    layout: RefCell<ToolbarLayout>,
}

impl ToolbarPreferences {
    pub const FILE_NAME: &'static str = "toolbar.json";
    const LOG_DOMAIN: &'static str = "ztbrowser";

    pub fn load(file: StoredFile) -> Self {
        let layout = file.read_json().map(|document| ToolbarLayout::from_json(&document)).unwrap_or_default();
        Self { file, layout: RefCell::new(layout) }
    }

    pub fn shows(&self, button: ToolbarButton) -> bool {
        self.layout.borrow().shows(button)
    }

    /// Shows or hides `button` and saves the choice.
    pub fn set_shown(&self, button: ToolbarButton, is_shown: bool) {
        let document = {
            let mut layout = self.layout.borrow_mut();
            layout.set_shown(button, is_shown);
            layout.to_json()
        };
        if let Err(error) = self.file.write_json(&document) {
            glib::g_warning!(Self::LOG_DOMAIN, "cannot save the toolbar buttons: {error}");
        }
    }
}
