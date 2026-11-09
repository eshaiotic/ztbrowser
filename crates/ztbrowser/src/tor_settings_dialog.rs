//! How the Tor window reaches Tor: directly, through Tor Browser's built-in Snowflake or obfs4 bridges, or
//! through bridges the user pastes. Connecting with a new choice saves it and restarts Tor.

use std::rc::Rc;

use adw::prelude::ActionRowExt;
use adw::prelude::AdwDialogExt;
use adw::prelude::PreferencesDialogExt;
use adw::prelude::PreferencesGroupExt;
use adw::prelude::PreferencesPageExt;
use gtk::prelude::ButtonExt;
use gtk::prelude::CheckButtonExt;
use gtk::prelude::IsA;
use gtk::prelude::TextBufferExt;
use gtk::prelude::TextViewExt;
use gtk::prelude::WidgetExt;
use ztbrowser_core::tor::bridge_choice::BridgeChoice;
use ztbrowser_core::tor::bridge_line::BridgeLine;
use ztbrowser_core::tor::pluggable_transport::PluggableTransport;

use crate::interface_messages::InterfaceMessages;
use crate::tor_connection::TorConnection;

pub struct TorSettingsDialog {
    dialog: adw::PreferencesDialog,
    direct: gtk::CheckButton,
    snowflake: gtk::CheckButton,
    obfs4: gtk::CheckButton,
    custom: gtk::CheckButton,
    custom_lines: gtk::TextView,
    problem: gtk::Label,
    tor: Rc<TorConnection>,
}

impl TorSettingsDialog {
    pub fn present(tor: &Rc<TorConnection>, parent: &impl IsA<gtk::Widget>) {
        let direct = gtk::CheckButton::new();
        let [snowflake, obfs4, custom] = [(); 3].map(|()| gtk::CheckButton::builder().group(&direct).build());
        let custom_lines = gtk::TextView::builder()
            .monospace(true)
            .wrap_mode(gtk::WrapMode::Char)
            .top_margin(8)
            .bottom_margin(8)
            .left_margin(8)
            .right_margin(8)
            .height_request(120)
            .build();
        let settings = Rc::new(Self {
            dialog: adw::PreferencesDialog::builder().title(tr!("Tor Connection")).build(),
            direct,
            snowflake,
            obfs4,
            custom,
            custom_lines,
            problem: gtk::Label::builder().wrap(true).xalign(0.0).css_classes(["error"]).visible(false).build(),
            tor: Rc::clone(tor),
        });
        settings.build();
        settings.show_choice(&tor.bridges());
        settings.dialog.present(Some(parent));
    }

    fn build(self: &Rc<Self>) {
        let programs = TorConnection::programs();
        let is_missing = |transport: PluggableTransport| {
            programs.as_ref().is_none_or(|programs| programs.program_for(transport).is_none())
        };
        let choices = adw::PreferencesGroup::builder()
            .title(tr!("How to Reach Tor"))
            .description(tr!("Bridges are unlisted Tor relays for networks that block Tor."))
            .build();
        let rows = [
            (&self.direct, tr!("Connect directly"), tr!("Best wherever Tor is not blocked.").to_owned()),
            (
                &self.snowflake,
                "Snowflake",
                if is_missing(PluggableTransport::Snowflake) {
                    tr!(
                        "Through volunteers' browsers; works on most censored networks. Needs {package}.",
                        package = PluggableTransport::Snowflake.package_hint(InterfaceMessages::catalogue())
                    )
                } else {
                    tr!("Through volunteers' browsers; works on most censored networks.").to_owned()
                },
            ),
            (
                &self.obfs4,
                "obfs4",
                if is_missing(PluggableTransport::Obfs4) {
                    tr!(
                        "Disguised connections to Tor Browser's built-in bridges. Needs {package}.",
                        package = PluggableTransport::Obfs4.package_hint(InterfaceMessages::catalogue())
                    )
                } else {
                    tr!("Disguised connections to Tor Browser's built-in bridges.").to_owned()
                },
            ),
            (
                &self.custom,
                tr!("Bridges I provide"),
                tr!("From bridges.torproject.org, by e-mail or from a friend.").to_owned(),
            ),
        ];
        for (check, title, subtitle) in rows {
            let row = adw::ActionRow::builder().title(title).subtitle(subtitle).activatable_widget(check).build();
            row.add_prefix(check);
            choices.add(&row);
        }
        let own = adw::PreferencesGroup::builder()
            .title(tr!("Your Bridges"))
            .description(tr!("One bridge per line, for example: obfs4 192.0.2.1:443 FINGERPRINT cert=… iat-mode=0"))
            .build();
        own.add(&gtk::Frame::builder().child(&self.custom_lines).build());
        own.add(&self.problem);
        let settings = Rc::downgrade(self);
        self.custom.connect_toggled(move |custom| {
            if let Some(settings) = settings.upgrade() {
                settings.custom_lines.set_sensitive(custom.is_active());
            }
        });
        let connect = gtk::Button::builder()
            .label(tr!("Connect"))
            .halign(gtk::Align::Center)
            .margin_top(12)
            .css_classes(["pill", "suggested-action"])
            .build();
        let settings = Rc::downgrade(self);
        connect.connect_clicked(move |_| {
            if let Some(settings) = settings.upgrade() {
                settings.connect();
            }
        });
        own.add(&connect);
        let page = adw::PreferencesPage::new();
        page.add(&choices);
        page.add(&own);
        self.dialog.add(&page);
        let owner = std::cell::RefCell::new(Some(Rc::clone(self)));
        self.dialog.connect_closed(move |_| drop(owner.take()));
    }

    fn show_choice(&self, choice: &BridgeChoice) {
        let check = match choice {
            BridgeChoice::Direct => &self.direct,
            BridgeChoice::Snowflake => &self.snowflake,
            BridgeChoice::Obfs4 => &self.obfs4,
            BridgeChoice::Custom(bridges) => {
                let lines: Vec<String> = bridges.iter().map(BridgeLine::to_line).collect();
                self.custom_lines.buffer().set_text(&lines.join("\n"));
                &self.custom
            }
        };
        check.set_active(true);
        self.custom_lines.set_sensitive(self.custom.is_active());
    }

    fn connect(&self) {
        let choice = if self.snowflake.is_active() {
            BridgeChoice::Snowflake
        } else if self.obfs4.is_active() {
            BridgeChoice::Obfs4
        } else if self.custom.is_active() {
            let buffer = self.custom_lines.buffer();
            match BridgeLine::parse_all(&buffer.text(&buffer.start_iter(), &buffer.end_iter(), false)) {
                Ok(bridges) if !bridges.is_empty() => BridgeChoice::Custom(bridges),
                Ok(_) => return self.show_problem(tr!("Paste at least one bridge.")),
                Err(invalid) => return self.show_problem(&invalid.describe(InterfaceMessages::catalogue())),
            }
        } else {
            BridgeChoice::Direct
        };
        self.tor.use_bridges(choice);
        self.dialog.close();
    }

    fn show_problem(&self, problem: &str) {
        self.problem.set_text(problem);
        self.problem.set_visible(true);
    }
}
