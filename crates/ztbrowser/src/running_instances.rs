//! Whether another ZTBrowser process (a profile or a web app) is running, asked through D-Bus.
//!
//! Every ZTBrowser process owns its application id on the session bus. Data of a running process must not be
//! deleted under its feet, so when the bus cannot answer, the process counts as running.

use gtk::prelude::ToVariant;
use ztbrowser_core::identity::application_id::ApplicationId;

pub struct RunningInstances;

impl RunningInstances {
    const TIMEOUT_MILLISECONDS: i32 = 1000;

    pub fn may_be_running(application_id: &ApplicationId) -> bool {
        let Ok(bus) = gio::bus_get_sync(gio::BusType::Session, None::<&gio::Cancellable>) else { return true };
        let reply = bus.call_sync(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            "org.freedesktop.DBus",
            "NameHasOwner",
            Some(&(application_id.as_str(),).to_variant()),
            glib::VariantTy::new("(b)").ok(),
            gio::DBusCallFlags::NONE,
            Self::TIMEOUT_MILLISECONDS,
            None::<&gio::Cancellable>,
        );
        reply.ok().and_then(|reply| reply.get::<(bool,)>()).is_none_or(|(has_owner,)| has_owner)
    }
}
