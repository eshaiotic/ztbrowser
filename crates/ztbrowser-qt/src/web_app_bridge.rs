//! What the interface asks about web apps: proposing and installing one, starting one, and keeping a running
//! app to its own site.
//!
//! QML collects the page's metadata and downloads the manifest and icons (without the profile's cookies); this
//! side decides, validates and writes, so nothing the site sent reaches a file unchecked.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type WebAppBridge = super::WebAppBridgeRust;

        /// How this process was launched, as `{"webApp": {"id", "name", "startUrl", "host"} | null, "addresses":
        /// [...]}`.
        #[qinvokable]
        fn launch_json(self: &WebAppBridge) -> QString;

        /// The script that reads the open page's title, manifest link and icon links; it returns JSON text.
        #[qinvokable]
        fn collector_script(self: &WebAppBridge) -> QString;

        /// The address of the manifest the collected `metadata` names, empty when there is none.
        #[qinvokable]
        fn manifest_address(self: &WebAppBridge, metadata: &QString) -> QString;

        /// The app proposed for `page_address`, as `{"name", "startUrl", "host", "icons": [addresses, best
        /// first]}`; `manifest` is the manifest's text, or empty.
        #[qinvokable]
        fn draft_json(self: &WebAppBridge, page_address: &QString, metadata: &QString, manifest: &QString) -> QString;

        /// Keeps the PNG (base64) as the icon of the next installation when it is sound; returns whether it was.
        #[qinvokable]
        fn accept_icon(self: &WebAppBridge, base64_png: &QString) -> bool;

        /// The same for a PNG file, such as the page's favicon saved by the interface.
        #[qinvokable]
        fn accept_icon_file(self: &WebAppBridge, path: &QString) -> bool;

        /// Where the interface may save an icon for [`accept_icon_file`].
        #[qinvokable]
        fn icon_staging_path(self: &WebAppBridge) -> QString;

        /// Installs the app and forgets the kept icon; `{"ok": true, "id", "name"}` or `{"ok": false, "error"}`.
        #[qinvokable]
        fn install(self: &WebAppBridge, name: &QString, start_address: &QString) -> QString;

        /// Starts the installed app `id` in its own process.
        #[qinvokable]
        fn start(self: &WebAppBridge, id: &QString) -> bool;

        /// Opens `address` in the browser, in its own process or the one that already runs.
        #[qinvokable]
        fn open_in_browser(self: &WebAppBridge, address: &QString) -> bool;

        /// Whether `address` is a page of the installed app `id` (the same origin as its start page).
        #[qinvokable]
        fn owns_page(self: &WebAppBridge, id: &QString, address: &QString) -> bool;

        /// Whether the running web app must hand this navigation to the browser rather than show it.
        #[qinvokable]
        fn leaves_app(self: &WebAppBridge, address: &QString, is_link_click: bool) -> bool;

        /// Whether `address` is a page of the running web app's own site.
        #[qinvokable]
        fn is_app_page(self: &WebAppBridge, address: &QString) -> bool;

        /// The page that lists the installed apps as tiles, in the light or dark scheme of the window.
        #[qinvokable]
        fn apps_page_html(self: &WebAppBridge, is_dark: bool) -> QString;

        /// The address the apps page is shown under.
        #[qinvokable]
        fn apps_page_address(self: &WebAppBridge) -> QString;

        /// What a link of the apps page asks for, as `{"kind": "open" | "uninstall", "id", "heading", "body"}`; empty when
        /// `address` is not such a link.
        #[qinvokable]
        fn apps_link_json(self: &WebAppBridge, address: &QString) -> QString;

        /// Removes the installed app `id` with its data; empty on success, else the message to show.
        #[qinvokable]
        fn uninstall(self: &WebAppBridge, id: &QString) -> QString;

        /// The requests later launches made since the last call, as a JSON array of address arrays.
        #[qinvokable]
        fn take_forwarded_json(self: &WebAppBridge) -> QString;
    }
}

use std::fs;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;
use std::sync::Mutex;
use std::thread;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use cxx_qt_lib::QString;
use serde_json::Value;
use serde_json::json;
use url::Url;
use ztbrowser_core::launch::program_location::ProgramLocation;
use ztbrowser_core::network::public_suffix_lookup::PublicSuffixLookup;
use ztbrowser_core::storage::owned_json_file::OwnedJsonFile;
use ztbrowser_core::web_app::desktop_entry::DesktopEntry;
use ztbrowser_core::web_app::installed_apps_page::InstalledAppsLink;
use ztbrowser_core::web_app::installed_apps_page::InstalledAppsPage;
use ztbrowser_core::web_app::page_metadata::PageMetadata;
use ztbrowser_core::web_app::web_app_catalog::WebAppCatalog;
use ztbrowser_core::web_app::web_app_id::WebAppId;
use ztbrowser_core::web_app::web_app_navigation_policy::NavigationTrigger;
use ztbrowser_core::web_app::web_app_navigation_policy::WebAppNavigation;
use ztbrowser_core::web_app::web_app_navigation_policy::WebAppNavigationPolicy;
use ztbrowser_engine_qt::qt_web_engine::QtWebEngine;

use crate::installed_web_apps::InstalledWebApps;
use crate::installed_web_apps::UninstallError;
use crate::interface_catalogue::InterfaceCatalogue;
use crate::launch_context::LaunchContext;
use crate::web_app_installer::IconPng;
use crate::web_app_installer::WebAppInstaller;

/// The largest icon file taken from a site or from the disk.
const ICON_MAX_BYTES: usize = 2 * 1024 * 1024;
const MANIFEST_MAX_BYTES: usize = 512 * 1024;
const FALLBACK_ICON_RESOURCE: &str = ":/icons/app.svg";

static KEPT_ICON: Mutex<Option<IconPng>> = Mutex::new(None);

#[derive(Default)]
pub struct WebAppBridgeRust;

fn text(value: &Value) -> QString {
    QString::from(value.to_string().as_str())
}

fn parsed(json: &QString) -> Option<Value> {
    serde_json::from_str(&json.to_string()).ok()
}

/// The icon of the next installation: a PNG that decodes and re-encodes in Qt, so only what Qt's own decoder
/// understood is ever written for the desktop to load.
fn keep_icon(bytes: &[u8]) -> bool {
    let kept = (bytes.len() <= ICON_MAX_BYTES)
        .then(|| IconPng::accept(bytes))
        .flatten()
        .and_then(|_| QtWebEngine::reencode_png(bytes))
        .and_then(|png| IconPng::accept(&png));
    let accepted = kept.is_some();
    *KEPT_ICON.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = kept;
    accepted
}

fn program() -> Option<std::path::PathBuf> {
    let appimage = std::env::var_os(ProgramLocation::APPIMAGE_VARIABLE);
    Some(ProgramLocation::choose(appimage.as_deref(), std::env::current_exe().ok()?))
}

/// Starts ZTBrowser again with `arguments`; the process is reaped in the background.
fn start_ztbrowser(arguments: &[&str]) -> bool {
    let Some(program) = program() else { return false };
    match Command::new(program).args(arguments).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn()
    {
        Ok(mut child) => {
            thread::spawn(move || child.wait());
            true
        }
        Err(_) => false,
    }
}

fn installed_catalog(launch: &LaunchContext) -> WebAppCatalog {
    OwnedJsonFile::new(launch.directories().web_app_catalog())
        .read()
        .map(|document| WebAppCatalog::from_json(&document))
        .unwrap_or_default()
}

// QML invokes these on the object, so they take `&self` even where they do not read it.
#[allow(clippy::unused_self)]
impl ffi::WebAppBridge {
    fn installed_apps() -> Option<InstalledWebApps<'static>> {
        let directories = LaunchContext::current()?.directories();
        Some(InstalledWebApps::new(directories, directories.runtime_directory()))
    }

    fn launch_json(&self) -> QString {
        let Some(launch) = LaunchContext::current() else { return text(&Value::Null) };
        let web_app = launch
            .web_app()
            .map(|app| json!({"id": app.id.as_str(), "name": app.name, "startUrl": app.start_url, "host": app.host()}));
        text(&json!({"webApp": web_app, "addresses": launch.addresses()}))
    }

    fn collector_script(&self) -> QString {
        QString::from(PageMetadata::COLLECTOR_SCRIPT)
    }

    fn manifest_address(&self, metadata: &QString) -> QString {
        let address = parsed(metadata).and_then(|metadata| WebAppInstaller::manifest_address(&metadata));
        QString::from(address.unwrap_or_default().as_str())
    }

    fn draft_json(&self, page_address: &QString, metadata: &QString, manifest: &QString) -> QString {
        let page_address = page_address.to_string();
        let manifest = manifest.to_string();
        let manifest =
            (manifest.len() <= MANIFEST_MAX_BYTES).then(|| serde_json::from_str::<Value>(&manifest).ok()).flatten();
        let draft = WebAppInstaller::draft(&page_address, &parsed(metadata).unwrap_or(Value::Null), manifest.as_ref());
        let host = Url::parse(&draft.start_url).ok().and_then(|url| url.host_str().map(str::to_owned));
        text(&json!({
            "name": draft.name,
            "startUrl": draft.start_url,
            "host": host.unwrap_or_default(),
            "icons": draft.icon_addresses,
        }))
    }

    fn accept_icon(&self, base64_png: &QString) -> bool {
        let encoded = base64_png.to_string();
        if encoded.len() > ICON_MAX_BYTES * 2 {
            return false;
        }
        STANDARD.decode(encoded.trim()).is_ok_and(|bytes| keep_icon(&bytes))
    }

    fn accept_icon_file(&self, path: &QString) -> bool {
        let path = path.to_string();
        let Ok(metadata) = fs::metadata(&path) else { return false };
        if !metadata.is_file() || metadata.len() > ICON_MAX_BYTES as u64 {
            return false;
        }
        let accepted = fs::read(&path).is_ok_and(|bytes| keep_icon(&bytes));
        let _ = fs::remove_file(Path::new(&path));
        accepted
    }

    fn icon_staging_path(&self) -> QString {
        let directory = LaunchContext::current().map(|launch| launch.directories().runtime_directory());
        let path = directory.unwrap_or_else(std::env::temp_dir).join("web-app-icon.png");
        QString::from(path.to_string_lossy().as_ref())
    }

    fn install(&self, name: &QString, start_address: &QString) -> QString {
        let Some(launch) = LaunchContext::current() else {
            return text(&json!({"ok": false, "error": "not ready"}));
        };
        let Some(executable) = program() else {
            return text(&json!({"ok": false, "error": "cannot locate the ZTBrowser executable"}));
        };
        let icon = KEPT_ICON.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take();
        let fallback = QtWebEngine::resource_text(FALLBACK_ICON_RESOURCE).unwrap_or_default();
        let installer = WebAppInstaller::new(launch.directories(), executable, InterfaceCatalogue::current());
        match installer.install(&name.to_string(), &start_address.to_string(), icon.as_ref(), &fallback) {
            Ok(app) => text(&json!({"ok": true, "id": app.id.as_str(), "name": app.name})),
            Err(error) => text(&json!({"ok": false, "error": error.to_string()})),
        }
    }

    fn start(&self, id: &QString) -> bool {
        WebAppId::parse(&id.to_string()).is_ok_and(|id| start_ztbrowser(&[DesktopEntry::WEB_APP_SWITCH, id.as_str()]))
    }

    fn open_in_browser(&self, address: &QString) -> bool {
        let address = address.to_string();
        Url::parse(&address).is_ok_and(|url| matches!(url.scheme(), "http" | "https")) && start_ztbrowser(&[&address])
    }

    fn owns_page(&self, id: &QString, address: &QString) -> bool {
        let (Some(launch), Ok(id)) = (LaunchContext::current(), WebAppId::parse(&id.to_string())) else { return false };
        installed_catalog(launch).web_app(&id).is_some_and(|app| app.is_own_page(&address.to_string()))
    }

    fn leaves_app(&self, address: &QString, is_link_click: bool) -> bool {
        let Some(app) = LaunchContext::current().and_then(LaunchContext::web_app) else { return false };
        let trigger = if is_link_click { NavigationTrigger::LinkClicked } else { NavigationTrigger::RedirectOrScript };
        WebAppNavigationPolicy::new(&app.start_url, &PublicSuffixLookup).decide(&address.to_string(), trigger)
            == WebAppNavigation::OpenInBrowser
    }

    fn is_app_page(&self, address: &QString) -> bool {
        LaunchContext::current().and_then(LaunchContext::web_app).is_some_and(|app| {
            WebAppNavigationPolicy::new(&app.start_url, &PublicSuffixLookup).is_app_page(&address.to_string())
        })
    }

    fn apps_page_html(&self, is_dark: bool) -> QString {
        let Some(apps) = Self::installed_apps() else { return QString::default() };
        QString::from(apps.page_html(InterfaceCatalogue::current(), is_dark).as_str())
    }

    fn apps_page_address(&self) -> QString {
        QString::from(InstalledAppsPage::ADDRESS)
    }

    fn apps_link_json(&self, address: &QString) -> QString {
        let Some(link) = InstalledAppsLink::parse(&address.to_string()) else { return QString::default() };
        let (kind, id) = match &link {
            InstalledAppsLink::Open(id) => ("open", id),
            InstalledAppsLink::Uninstall(id) => ("uninstall", id),
        };
        let name = Self::installed_apps()
            .and_then(|apps| apps.catalog().web_app(id).map(|app| app.name.clone()))
            .unwrap_or_default();
        let catalogue = InterfaceCatalogue::current();
        text(&json!({
            "kind": kind,
            "id": id.as_str(),
            "heading": InstalledAppsPage::uninstall_heading(catalogue, &name),
            "body": InstalledAppsPage::uninstall_warning(catalogue),
        }))
    }

    fn uninstall(&self, id: &QString) -> QString {
        let (Some(apps), Ok(id)) = (Self::installed_apps(), WebAppId::parse(&id.to_string())) else {
            return QString::default();
        };
        let catalogue = InterfaceCatalogue::current();
        let message = match apps.uninstall(&id) {
            Ok(()) => String::new(),
            Err(UninstallError::StillRunning(name)) => {
                catalogue.text_with("Close {name} before deleting it", &[("name", &name)])
            }
            Err(UninstallError::Incomplete(error)) => {
                catalogue.text_with("Could not remove everything: {error}", &[("error", &error.to_string())])
            }
        };
        QString::from(message.as_str())
    }

    fn take_forwarded_json(&self) -> QString {
        text(&json!(LaunchContext::take_forwarded()))
    }
}
