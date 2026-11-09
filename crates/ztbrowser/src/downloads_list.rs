//! The list the downloads button and the main menu's "Downloads" entry open: the latest downloads of this
//! window, a way to clear that list, a way into the whole history, and a last row to the download folder.
//!
//! Clearing empties only this window's view (a downloads list that starts clean); the history, the files and
//! the other windows' lists are untouched. The list follows the [`DownloadCenter`] while it is visible.

use std::cell::Cell;
use std::rc::Rc;

use glib::prelude::Cast;
use glib::prelude::CastNone;
use glib::prelude::ObjectExt;
use gtk::prelude::BoxExt;
use gtk::prelude::ButtonExt;
use gtk::prelude::PopoverExt;
use gtk::prelude::StaticType;
use gtk::prelude::WidgetExt;
use ztbrowser_core::download::download_history::DownloadEntry;
use ztbrowser_core::download::download_history::DownloadId;

use crate::download_center::DownloadCenter;
use crate::download_row::DownloadRow;
use crate::downloads_manager::DownloadsManager;

pub struct DownloadsList {
    root: gtk::Box,
    center: Rc<DownloadCenter>,
    /// `false` in the Tor window: a file opened in another application would connect outside Tor.
    opens_files: bool,
    /// The newest download this window's person cleared away; this list shows only later ones.
    cleared_through: Cell<Option<DownloadId>>,
}

impl DownloadsList {
    const WIDTH: i32 = 360;
    /// How many downloads the list shows.
    const ROWS: usize = 5;

    pub fn new(center: &Rc<DownloadCenter>, opens_files: bool) -> Rc<Self> {
        let list = Rc::new(Self {
            root: gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(2)
                .width_request(Self::WIDTH)
                .build(),
            center: Rc::clone(center),
            opens_files,
            cleared_through: Cell::new(None),
        });
        list.rebuild();
        let followed = list.root.downgrade();
        let owner = Rc::downgrade(&list);
        center.observe(move || match (followed.upgrade(), owner.upgrade()) {
            (Some(root), Some(list)) => {
                if root.is_mapped() {
                    list.rebuild();
                }
                glib::ControlFlow::Continue
            }
            _ => glib::ControlFlow::Break,
        });
        let owner = Rc::clone(&list);
        list.root.connect_map(move |_| owner.rebuild());
        list
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.root
    }

    fn rebuild(self: &Rc<Self>) {
        while let Some(child) = self.root.first_child() {
            self.root.remove(&child);
        }
        let entries = self.center.latest_after(self.cleared_through.get(), Self::ROWS);
        if entries.is_empty() {
            let empty = gtk::Label::builder()
                .label(tr!("No downloads yet"))
                .css_classes(["dim-label"])
                .margin_top(12)
                .margin_bottom(12)
                .build();
            self.root.append(&empty);
        }
        for entry in &entries {
            self.root.append(&DownloadRow::build(entry, self.opens_files, None));
        }
        self.root.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        self.root.append(&self.show_all_button());
        self.root.append(&self.clear_button(!entries.is_empty()));
        self.root.append(&self.folder_button());
    }

    fn show_all_button(self: &Rc<Self>) -> gtk::Button {
        let button = Self::text_button(tr!("Show All Downloads"));
        let list = Rc::clone(self);
        button.connect_clicked(move |button| {
            let window = Self::window_of(button);
            Self::close_popover(button);
            if let Some(window) = window {
                DownloadsManager::present(&list.center, list.opens_files, &window);
            }
        });
        button
    }

    fn clear_button(self: &Rc<Self>, has_rows: bool) -> gtk::Button {
        let button = Self::text_button(tr!("Clear List"));
        button.set_sensitive(has_rows);
        let list = Rc::clone(self);
        button.connect_clicked(move |_| {
            let newest = list.center.entries().first().map(DownloadEntry::id);
            list.cleared_through.set(newest);
            list.rebuild();
        });
        button
    }

    fn folder_button(&self) -> gtk::Button {
        let button = Self::text_button(tr!("Go to Download Folder"));
        let folder = self.center.folder();
        button.connect_clicked(move |button| {
            gtk::FileLauncher::new(Some(&gio::File::for_path(&folder))).launch(
                Self::window_of(button).as_ref(),
                None::<&gio::Cancellable>,
                |_| {},
            );
        });
        button
    }

    fn text_button(label: &str) -> gtk::Button {
        let text = gtk::Label::builder().label(label).css_classes(["body"]).build();
        gtk::Button::builder().child(&text).css_classes(["flat"]).build()
    }

    fn close_popover(button: &gtk::Button) {
        if let Some(popover) = button.ancestor(gtk::Popover::static_type()).and_downcast::<gtk::Popover>() {
            popover.popdown();
        }
    }

    fn window_of(button: &gtk::Button) -> Option<gtk::Window> {
        button.root().and_then(|root| root.downcast::<gtk::Window>().ok())
    }
}
