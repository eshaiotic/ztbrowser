//! The downloads manager: the whole history of downloaded files, searchable by name, with a way to take one
//! entry out of the list or to clear the whole history. Files on disk are never touched.

use std::rc::Rc;

use adw::prelude::AdwDialogExt;
use glib::prelude::ObjectExt;
use gtk::prelude::BoxExt;
use gtk::prelude::ButtonExt;
use gtk::prelude::EditableExt;

use gtk::prelude::WidgetExt;
use ztbrowser_core::download::download_history::DownloadId;

use crate::download_center::DownloadCenter;
use crate::download_row::DownloadRow;

pub struct DownloadsManager {
    list: gtk::ListBox,
    center: Rc<DownloadCenter>,
    opens_files: bool,
}

impl DownloadsManager {
    const LOG_DOMAIN: &'static str = "ztbrowser";
    const WIDTH: i32 = 560;
    const HEIGHT: i32 = 560;

    pub fn present(center: &Rc<DownloadCenter>, opens_files: bool, parent: &gtk::Window) {
        let manager = Rc::new(Self {
            list: gtk::ListBox::builder().selection_mode(gtk::SelectionMode::None).css_classes(["boxed-list"]).build(),
            center: Rc::clone(center),
            opens_files,
        });
        let search = gtk::SearchEntry::builder().placeholder_text(tr!("Search Downloads")).hexpand(true).build();
        let list = manager.list.clone();
        manager.list.set_filter_func({
            let search = search.clone();
            move |row| row.widget_name().contains(search.text().to_lowercase().as_str())
        });
        search.connect_search_changed(move |_| list.invalidate_filter());
        let scroller = gtk::ScrolledWindow::builder()
            .child(&manager.list)
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .build();
        let clear = gtk::Button::builder().label(tr!("Clear History")).css_classes(["destructive-action"]).build();
        let owner = Rc::clone(&manager);
        clear.connect_clicked(move |_| {
            if let Err(error) = owner.center.forget_all() {
                glib::g_warning!(Self::LOG_DOMAIN, "cannot clear the downloads history: {error}");
            }
        });
        let header = adw::HeaderBar::new();
        header.pack_end(&clear);
        let body = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_bottom(12)
            .build();
        body.append(&search);
        body.append(&scroller);
        let content = adw::ToolbarView::new();
        content.add_top_bar(&header);
        content.set_content(Some(&body));
        let dialog = adw::Dialog::builder()
            .title(tr!("Downloads"))
            .content_width(Self::WIDTH)
            .content_height(Self::HEIGHT)
            .child(&content)
            .build();
        manager.rebuild();
        let followed = manager.list.downgrade();
        let owner = Rc::downgrade(&manager);
        center.observe(move || match (followed.upgrade(), owner.upgrade()) {
            (Some(_), Some(manager)) => {
                manager.rebuild();
                glib::ControlFlow::Continue
            }
            _ => glib::ControlFlow::Break,
        });
        // The manager lives as long as its dialog does.
        dialog.connect_closed(move |_| drop(Rc::clone(&manager)));
        dialog.present(Some(parent));
    }

    fn rebuild(self: &Rc<Self>) {
        self.list.remove_all();
        let center = Rc::clone(&self.center);
        let remover: Rc<dyn Fn(DownloadId)> = Rc::new(move |id| center.remove(id));
        for entry in self.center.entries() {
            let row = gtk::ListBoxRow::builder()
                .activatable(false)
                .child(&DownloadRow::build(&entry, self.opens_files, Some(Rc::clone(&remover))))
                .build();
            row.set_widget_name(&entry.name().to_lowercase());
            self.list.append(&row);
        }
        if self.list.first_child().is_none() {
            let empty = gtk::Label::builder()
                .label(tr!("No downloads yet"))
                .css_classes(["dim-label"])
                .margin_top(24)
                .margin_bottom(24)
                .build();
            self.list.append(&empty);
        }
    }
}
