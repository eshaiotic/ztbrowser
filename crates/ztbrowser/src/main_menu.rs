//! The three-dots menu: what each window offers, every entry with an icon on the left and its shortcut on
//! the right, so the eye finds an entry without reading.
//!
//! GTK draws icons only for the zoom buttons of a menu model, so each entry is a custom row: a flat button
//! that runs the entry's action and closes the menu. The model still orders the rows, sections and headings.
//!
//! The Tor window gets a shorter menu: no profiles, web apps or search engines, which would take browsing
//! out of Tor.

use gtk::prelude::ActionableExt;
use gtk::prelude::BoxExt;
use gtk::prelude::ButtonExt;
use gtk::prelude::Cast;
use gtk::prelude::CastNone;
use gtk::prelude::GtkApplicationExt;
use gtk::prelude::IsA;
use gtk::prelude::PopoverExt;
use gtk::prelude::StaticType;
use gtk::prelude::ToVariant;
use gtk::prelude::WidgetExt;
use ztbrowser_core::navigation::search_engine::SearchEngine;
use ztbrowser_core::page_zoom::zoom_level::ZoomLevel;

use crate::app_context::AppContext;
use crate::app_icons::AppIcons;
use crate::tor_connection_page::TorConnectionPage;
use crate::zoom_controls::ZoomControls;

pub struct MainMenu {
    /// The custom rows, by the id the model's items name them with.
    rows: Vec<(String, gtk::Widget)>,
}

impl MainMenu {
    pub fn popover(context: &AppContext, zoom: ZoomLevel) -> gtk::PopoverMenu {
        let mut menu = Self { rows: Vec::new() };
        let model = menu.model(context, zoom);
        let popover = gtk::PopoverMenu::from_model(Some(&model));
        for (id, row) in &menu.rows {
            popover.add_child(row, id);
        }
        popover
    }

    fn model(&mut self, context: &AppContext, zoom: ZoomLevel) -> gio::Menu {
        let menu = gio::Menu::new();
        let tabs = gio::Menu::new();
        tabs.append_item(&self.entry(tr!("New Tab"), "win.new-tab", "tab-new-symbolic"));
        if context.tor.is_none() {
            tabs.append_item(&self.entry(tr!("New Window"), "app.new-window", "window-new-symbolic"));
        }
        tabs.append_item(&self.entry(tr!("New Tor Window"), "app.new-tor-window", AppIcons::TOR_WINDOW));
        menu.append_section(None, &tabs);
        let page = gio::Menu::new();
        page.append_item(&self.entry(tr!("Downloads"), "win.downloads", "folder-download-symbolic"));
        page.append_item(&self.entry(tr!("Find in Page…"), "win.find", "edit-find-symbolic"));
        menu.append_section(None, &page);
        menu.append_item(&ZoomControls::menu_row(zoom));
        if context.tor.is_some() {
            let tor = gio::Menu::new();
            tor.append_item(&self.entry(tr!("New Identity"), "win.fire", AppIcons::FIRE));
            tor.append_item(&self.entry(
                tr!("Tor Connection…"),
                TorConnectionPage::SETTINGS_ACTION,
                "network-workgroup-symbolic",
            ));
            menu.append_section(None, &tor);
            let about = gio::Menu::new();
            about.append_item(&self.entry(tr!("About ZTBrowser"), "app.about", "help-about-symbolic"));
            menu.append_section(None, &about);
            return menu;
        }
        let apps = gio::Menu::new();
        apps.append_item(&self.entry(tr!("Install Site as App…"), "win.install-web-app", AppIcons::INSTALL_APP));
        apps.append_item(&self.entry(tr!("Installed Apps…"), "win.manage-web-apps", "view-app-grid-symbolic"));
        menu.append_section(None, &apps);
        let profiles = gio::Menu::new();
        for profile in context.catalogs.profiles().profiles() {
            let target = profile.id.as_str().to_variant();
            profiles.append_item(&self.targeted_entry(
                &profile.name,
                "app.open-profile",
                "avatar-default-symbolic",
                &target,
            ));
        }
        profiles.append_item(&self.entry(tr!("Manage Profiles…"), "win.manage-profiles", "system-users-symbolic"));
        menu.append_section(Some(tr!("Profiles")), &profiles);
        let about = gio::Menu::new();
        about.append_item(&self.search_engine_entry());
        about.append_item(&self.entry(tr!("Preferences…"), "win.preferences", "preferences-system-symbolic"));
        about.append_item(&self.entry(tr!("About ZTBrowser"), "app.about", "help-about-symbolic"));
        menu.append_section(None, &about);
        menu
    }

    fn entry(&mut self, label: &str, action: &str, icon: &str) -> gio::MenuItem {
        let button = Self::button(label, icon, Some(&Self::shortcut_of(action)));
        button.set_action_name(Some(action));
        self.register(button.upcast())
    }

    /// An entry whose action takes `target` (which profile to open).
    fn targeted_entry(&mut self, label: &str, action: &str, icon: &str, target: &glib::Variant) -> gio::MenuItem {
        let button = Self::button(label, icon, None);
        button.set_action_name(Some(action));
        button.set_action_target_value(Some(target));
        self.register(button.upcast())
    }

    /// "Search Engine ›": a row that opens the list of engines to the side.
    fn search_engine_entry(&mut self) -> gio::MenuItem {
        let engines = gio::Menu::new();
        for engine in SearchEngine::ALL {
            let item = gio::MenuItem::new(Some(engine.name()), None);
            item.set_action_and_target_value(Some("app.search-engine"), Some(&engine.id().to_variant()));
            engines.append_item(&item);
        }
        let arrow = gtk::Image::from_icon_name("go-next-symbolic");
        let row = gtk::MenuButton::builder()
            .child(&Self::row_content(tr!("Search Engine"), "system-search-symbolic", Some(&arrow)))
            .menu_model(&engines)
            .direction(gtk::ArrowType::Right)
            .always_show_arrow(false)
            .css_classes(["flat"])
            .build();
        self.register(row.upcast())
    }

    fn register(&mut self, row: gtk::Widget) -> gio::MenuItem {
        let id = format!("row-{}", self.rows.len());
        let item = gio::MenuItem::new(None, None);
        item.set_attribute_value("custom", Some(&id.to_variant()));
        self.rows.push((id, row));
        item
    }

    fn button(label: &str, icon: &str, shortcut: Option<&str>) -> gtk::Button {
        let trailing = shortcut
            .filter(|text| !text.is_empty())
            .map(|text| gtk::Label::builder().label(text).css_classes(["dim-label"]).build());
        let button = gtk::Button::builder()
            .child(&Self::row_content(label, icon, trailing.as_ref()))
            .css_classes(["flat"])
            .build();
        button.connect_clicked(|button| {
            if let Some(popover) = button.ancestor(gtk::Popover::static_type()).and_downcast::<gtk::Popover>() {
                popover.popdown();
            }
        });
        button
    }

    fn row_content(label: &str, icon: &str, trailing: Option<&impl IsA<gtk::Widget>>) -> gtk::Box {
        let content = gtk::Box::builder().spacing(12).build();
        content.append(&gtk::Image::from_icon_name(icon));
        content.append(&gtk::Label::builder().label(label).xalign(0.0).hexpand(true).css_classes(["body"]).build());
        if let Some(trailing) = trailing {
            content.append(trailing);
        }
        content
    }

    /// The first accelerator of `action` as the user sees it ("Ctrl+T"), empty when it has none.
    fn shortcut_of(action: &str) -> String {
        gio::Application::default()
            .and_then(|application| application.downcast::<gtk::Application>().ok())
            .and_then(|application| application.accels_for_action(action).first().cloned())
            .and_then(|accelerator| gtk::accelerator_parse(&accelerator))
            .map(|(key, modifiers)| gtk::accelerator_get_label(key, modifiers).to_string())
            .unwrap_or_default()
    }
}
