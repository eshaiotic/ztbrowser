//! The browser windows open in this process, so that one window's action can reach the others (installing a
//! site as an app closes that site's tabs in every window).
//!
//! Only weak references are kept: every window owns the shared `AppContext` holding this list, so strong ones
//! would keep closed windows alive forever.

use std::cell::RefCell;
use std::rc::Rc;
use std::rc::Weak;

use gtk::prelude::Cast;
use gtk::prelude::GtkApplicationExt;

use crate::browser_window::BrowserWindow;

#[derive(Default)]
pub struct BrowserWindows {
    windows: RefCell<Vec<Weak<BrowserWindow>>>,
}

impl BrowserWindows {
    pub fn register(&self, browser: &Rc<BrowserWindow>) {
        let mut windows = self.windows.borrow_mut();
        windows.retain(|window| window.strong_count() > 0);
        windows.push(Rc::downgrade(browser));
    }

    pub fn open(&self) -> Vec<Rc<BrowserWindow>> {
        self.windows.borrow().iter().filter_map(Weak::upgrade).collect()
    }

    /// The browser window the user is working in, if the application's active window is one.
    pub fn active(&self, application: &adw::Application) -> Option<Rc<BrowserWindow>> {
        let active = application.active_window()?;
        self.open().into_iter().find(|browser| browser.window().upcast_ref::<gtk::Window>() == &active)
    }
}
