//! What a Tor window shows until Tor is connected: how far Tor got, what it is doing, what went wrong, and
//! the way to connect differently. Nothing leaves the window meanwhile: pages wait for the connection.

use adw::prelude::BoxExt;
use adw::prelude::WidgetExt;
use gtk::prelude::Cast;
use ztbrowser_core::tor::bootstrap_status::BootstrapStatus;

use crate::app_icons::AppIcons;
use crate::interface_messages::InterfaceMessages;
use crate::tor_connection::TorState;

pub struct TorConnectionPage {
    status: adw::StatusPage,
    progress: gtk::ProgressBar,
    problem: gtk::Label,
    retry: gtk::Button,
}

impl TorConnectionPage {
    pub const RETRY_ACTION: &'static str = "win.tor-retry";
    pub const SETTINGS_ACTION: &'static str = "win.tor-settings";

    pub fn new() -> Self {
        let progress = gtk::ProgressBar::builder().show_text(true).build();
        let problem = gtk::Label::builder()
            .wrap(true)
            .selectable(true)
            .justify(gtk::Justification::Center)
            .css_classes(["error"])
            .visible(false)
            .build();
        let retry = gtk::Button::builder()
            .label(tr!("Try Again"))
            .action_name(Self::RETRY_ACTION)
            .halign(gtk::Align::Center)
            .css_classes(["pill", "suggested-action"])
            .visible(false)
            .build();
        let settings = gtk::Button::builder()
            .label(tr!("Connection Settings…"))
            .action_name(Self::SETTINGS_ACTION)
            .halign(gtk::Align::Center)
            .css_classes(["pill"])
            .build();
        let column = gtk::Box::new(gtk::Orientation::Vertical, 18);
        for widget in
            [progress.upcast_ref::<gtk::Widget>(), problem.upcast_ref(), retry.upcast_ref(), settings.upcast_ref()]
        {
            column.append(widget);
        }
        let status = adw::StatusPage::builder()
            .icon_name(AppIcons::APPLICATION)
            .child(&adw::Clamp::builder().maximum_size(460).child(&column).build())
            .build();
        Self { status, progress, problem, retry }
    }

    pub fn widget(&self) -> &adw::StatusPage {
        &self.status
    }

    pub fn show(&self, state: &TorState) {
        match state {
            TorState::Starting => self.show_progress(0.0, tr!("Starting Tor…"), None),
            TorState::Connecting(status) => self.show_bootstrap(status),
            TorState::Connected { .. } => self.show_progress(1.0, tr!("Connected"), None),
            TorState::Failed(problem) => {
                self.status.set_title(tr!("Cannot Connect to Tor"));
                self.status.set_description(Some(tr!("Pages stay unopened until the Tor window is connected.")));
                self.progress.set_visible(false);
                self.show_problem(Some(problem));
                self.retry.set_visible(true);
            }
        }
    }

    fn show_bootstrap(&self, status: &BootstrapStatus) {
        let catalogue = InterfaceMessages::catalogue();
        self.show_progress(f64::from(status.progress()) / 100.0, status.summary(catalogue), status.problem(catalogue));
    }

    fn show_progress(&self, fraction: f64, doing: &str, problem: Option<&str>) {
        self.status.set_title(tr!("Connecting to Tor"));
        self.status.set_description(Some(tr!(
            "ZTBrowser's own Tor is connecting. Nothing leaves this window until it is connected."
        )));
        self.progress.set_visible(true);
        self.progress.set_fraction(fraction);
        self.progress.set_text(Some(doing));
        self.show_problem(problem);
        self.retry.set_visible(problem.is_some());
    }

    fn show_problem(&self, problem: Option<&str>) {
        self.problem.set_text(problem.unwrap_or_default());
        self.problem.set_visible(problem.is_some());
    }
}
