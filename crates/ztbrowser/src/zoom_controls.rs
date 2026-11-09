//! The zoom controls of a window showing a page: a menu row (zoom out, the size, zoom in) and a badge in the
//! header that names any size other than 100 % and puts the page back to 100 % when clicked.
//!
//! The window provides the actions `win.zoom-in`, `win.zoom-out` and `win.zoom-reset` for its page on screen.

use gio::prelude::ActionMapExt;
use gtk::prelude::ButtonExt;
use gtk::prelude::Cast;
use gtk::prelude::ToVariant;
use gtk::prelude::WidgetExt;
use ztbrowser_core::page_zoom::zoom_level::ZoomLevel;

pub struct ZoomControls {
    badge: gtk::Button,
}

impl ZoomControls {
    pub fn new() -> Self {
        Self {
            badge: gtk::Button::builder()
                .action_name("win.zoom-reset")
                .tooltip_text(tr!("Reset Zoom"))
                .valign(gtk::Align::Center)
                .visible(false)
                .build(),
        }
    }

    pub fn badge(&self) -> &gtk::Button {
        &self.badge
    }

    /// The menu row for a page drawn at `level`: icon buttons around the size, which resets it.
    pub fn menu_row(level: ZoomLevel) -> gio::MenuItem {
        let row = gio::Menu::new();
        for (label, action, icon) in [
            (tr!("Zoom Out"), "win.zoom-out", Some("zoom-out-symbolic")),
            (level.to_string().as_str(), "win.zoom-reset", None),
            (tr!("Zoom In"), "win.zoom-in", Some("zoom-in-symbolic")),
        ] {
            let item = gio::MenuItem::new(Some(label), Some(action));
            if let Some(icon) = icon {
                item.set_attribute_value("verb-icon", Some(&icon.to_variant()));
            }
            row.append_item(&item);
        }
        let section = gio::MenuItem::new_section(None, &row);
        section.set_attribute_value("display-hint", Some(&"horizontal-buttons".to_variant()));
        section
    }

    /// Shows that the page on screen is drawn at `level`, and lets zooming go only as far as it can.
    pub fn show(&self, window: &adw::ApplicationWindow, level: ZoomLevel) {
        self.badge.set_label(&level.to_string());
        self.badge.set_visible(!level.is_actual_size());
        for (action, enabled) in [("zoom-in", level.can_zoom_in()), ("zoom-out", level.can_zoom_out())] {
            if let Some(action) =
                window.lookup_action(action).and_then(|action| action.downcast::<gio::SimpleAction>().ok())
            {
                action.set_enabled(enabled);
            }
        }
    }
}
