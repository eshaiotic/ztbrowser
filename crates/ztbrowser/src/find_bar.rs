//! Find in page (Ctrl+F): a search bar driving WebKit's find controller of the current page.

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::EditableExt;
use gtk::prelude::WidgetExt;
use webkit6::FindOptions;
use webkit6::WebView;
use webkit6::prelude::WebViewExt;

pub struct FindBar {
    bar: gtk::SearchBar,
    entry: gtk::SearchEntry,
    target: RefCell<Option<WebView>>,
}

impl FindBar {
    const MAX_MATCHES: u32 = 1000;

    pub fn new() -> Rc<Self> {
        let entry = gtk::SearchEntry::builder().placeholder_text(tr!("Find in page")).width_chars(32).build();
        let bar = gtk::SearchBar::builder().child(&entry).show_close_button(true).build();
        bar.connect_entry(&entry);
        let find_bar = Rc::new(Self { bar, entry, target: RefCell::new(None) });
        find_bar.connect_signals();
        find_bar
    }

    pub fn widget(&self) -> &gtk::SearchBar {
        &self.bar
    }

    pub fn show_for(&self, web_view: &WebView) {
        self.finish();
        self.target.replace(Some(web_view.clone()));
        self.bar.set_search_mode(true);
        self.entry.grab_focus();
        self.search();
    }

    fn connect_signals(self: &Rc<Self>) {
        let find_bar = Rc::downgrade(self);
        self.entry.connect_search_changed(move |_| find_bar.upgrade().iter().for_each(|bar| bar.search()));
        let find_bar = Rc::downgrade(self);
        self.entry.connect_activate(move |_| find_bar.upgrade().iter().for_each(|bar| bar.step(true)));
        let find_bar = Rc::downgrade(self);
        self.entry.connect_next_match(move |_| find_bar.upgrade().iter().for_each(|bar| bar.step(true)));
        let find_bar = Rc::downgrade(self);
        self.entry.connect_previous_match(move |_| find_bar.upgrade().iter().for_each(|bar| bar.step(false)));
        let find_bar = Rc::downgrade(self);
        self.bar.connect_search_mode_enabled_notify(move |bar| {
            if !bar.is_search_mode()
                && let Some(find_bar) = find_bar.upgrade()
            {
                find_bar.finish();
            }
        });
    }

    fn search(&self) {
        let Some(controller) = self.target.borrow().as_ref().and_then(WebViewExt::find_controller) else { return };
        let text = self.entry.text();
        if text.is_empty() {
            controller.search_finish();
        } else {
            let options = (FindOptions::CASE_INSENSITIVE | FindOptions::WRAP_AROUND).bits();
            controller.search(&text, options, Self::MAX_MATCHES);
        }
    }

    fn step(&self, forward: bool) {
        if let Some(controller) = self.target.borrow().as_ref().and_then(WebViewExt::find_controller) {
            if forward { controller.search_next() } else { controller.search_previous() }
        }
    }

    fn finish(&self) {
        if let Some(controller) = self.target.borrow().as_ref().and_then(WebViewExt::find_controller) {
            controller.search_finish();
        }
    }
}
