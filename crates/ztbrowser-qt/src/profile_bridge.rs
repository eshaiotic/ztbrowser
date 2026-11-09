//! What the interface asks about browsing profiles: the tiles of the profiles popup, adding a profile, opening
//! one in its own process, and deleting one.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type ProfileBridge = super::ProfileBridgeRust;

        /// The profiles as `[{"id", "name", "initial", "colourIndex", "isCurrent", "isDefault"}, ...]`.
        #[qinvokable]
        fn tiles_json(self: &ProfileBridge) -> QString;

        /// Adds a profile called `name`: `{"ok": true, "id"}`, or `{"ok": false}` when the name is blank or the
        /// catalogue could not be written.
        #[qinvokable]
        fn create(self: &ProfileBridge, name: &QString) -> QString;

        /// Starts the profile in a process of its own (or brings its window forward).
        #[qinvokable]
        fn open(self: &ProfileBridge, id: &QString) -> bool;

        /// Deletes the profile with its data; the empty text when it worked, else why it did not.
        #[qinvokable]
        fn delete_profile(self: &ProfileBridge, id: &QString) -> QString;
    }
}

use std::process::Command;
use std::process::Stdio;
use std::thread;

use cxx_qt_lib::QString;
use serde_json::Value;
use serde_json::json;
use ztbrowser_core::launch::program_location::ProgramLocation;
use ztbrowser_core::profile::profile_id::ProfileId;

use crate::installed_profiles::DeleteError;
use crate::installed_profiles::InstalledProfiles;
use crate::interface_catalogue::InterfaceCatalogue;
use crate::launch_context::LaunchContext;

#[derive(Default)]
pub struct ProfileBridgeRust;

fn text(value: &Value) -> QString {
    QString::from(value.to_string().as_str())
}

/// Starts ZTBrowser again for the profile `id`; the process is reaped in the background.
fn start_profile(id: &ProfileId) -> bool {
    let appimage = std::env::var_os(ProgramLocation::APPIMAGE_VARIABLE);
    let Some(program) = std::env::current_exe().ok().map(|exe| ProgramLocation::choose(appimage.as_deref(), exe))
    else {
        return false;
    };
    let spawned = Command::new(program)
        .args(["--profile", id.as_str()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    match spawned {
        Ok(mut child) => {
            thread::spawn(move || child.wait());
            true
        }
        Err(_) => false,
    }
}

// QML invokes these on the object, so they take `&self` even where they do not read it.
#[allow(clippy::unused_self)]
impl ffi::ProfileBridge {
    fn profiles() -> Option<InstalledProfiles<'static>> {
        let directories = LaunchContext::current()?.directories();
        Some(InstalledProfiles::new(directories, directories.runtime_directory()))
    }

    fn tiles_json(&self) -> QString {
        let (Some(profiles), Some(launch)) = (Self::profiles(), LaunchContext::current()) else {
            return text(&json!([]));
        };
        let tiles: Vec<Value> = profiles
            .tiles(&launch.profile_id(), InterfaceCatalogue::current())
            .into_iter()
            .map(|tile| {
                json!({
                    "id": tile.id, "name": tile.name, "initial": tile.initial, "colourIndex": tile.colour_index,
                    "isCurrent": tile.is_current, "isDefault": tile.is_default,
                })
            })
            .collect();
        text(&json!(tiles))
    }

    fn create(&self, name: &QString) -> QString {
        let created = Self::profiles().and_then(|profiles| profiles.create(&name.to_string()).ok().flatten());
        text(&created.map_or_else(|| json!({"ok": false}), |id| json!({"ok": true, "id": id.as_str()})))
    }

    fn open(&self, id: &QString) -> bool {
        ProfileId::parse(&id.to_string()).is_ok_and(|id| start_profile(&id))
    }

    fn delete_profile(&self, id: &QString) -> QString {
        let (Some(profiles), Ok(id)) = (Self::profiles(), ProfileId::parse(&id.to_string())) else {
            return QString::default();
        };
        let catalogue = InterfaceCatalogue::current();
        let message = match profiles.delete(&id) {
            Ok(()) | Err(DeleteError::IsDefault) => String::new(),
            Err(DeleteError::StillRunning(name)) => {
                catalogue.text_with("Close {name} before deleting it", &[("name", &name)])
            }
            Err(DeleteError::Incomplete(error)) => {
                catalogue.text_with("Could not remove everything: {error}", &[("error", &error.to_string())])
            }
        };
        QString::from(message.as_str())
    }
}
