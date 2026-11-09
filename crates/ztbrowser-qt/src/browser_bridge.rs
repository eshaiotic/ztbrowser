//! What the QML window asks Rust: where an address-bar entry leads, interface text, page scripts, the shield,
//! zoom, downloads, the Fire button and the default browser.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type BrowserBridge = super::BrowserBridgeRust;

        /// The address to open for what was typed in the address bar: a web address, else a search. Empty
        /// for blank input.
        #[qinvokable]
        fn destination(self: &BrowserBridge, typed_text: &QString) -> QString;

        /// An interface message in the language this process speaks (the message itself when untranslated).
        #[qinvokable]
        fn message(self: &BrowserBridge, text: &QString) -> QString;

        /// The mouse-wheel smoothing script, or empty when the person turned smoothing off.
        #[qinvokable]
        fn smooth_wheel_script(self: &BrowserBridge) -> QString;

        /// The script that hides DuckDuckGo's offer of an extension, which this browser neither needs nor runs.
        #[qinvokable]
        fn duckduckgo_extension_prompt_script(self: &BrowserBridge) -> QString;

        /// Whether the toolbar shows the button stored as `id` (`reload`).
        #[qinvokable]
        fn toolbar_shows(self: &BrowserBridge, id: &QString) -> bool;

        #[qinvokable]
        fn set_toolbar_shown(self: &BrowserBridge, id: &QString, is_shown: bool);

        /// Whether the address bar takes all the room the toolbar leaves it.
        #[qinvokable]
        fn wide_address_bar(self: &BrowserBridge) -> bool;

        #[qinvokable]
        fn set_wide_address_bar(self: &BrowserBridge, is_wide: bool);

        /// Which side of the page the tabs sit on: `top`, `bottom`, `left` or `right`.
        #[qinvokable]
        fn tab_placement(self: &BrowserBridge) -> QString;

        #[qinvokable]
        fn set_tab_placement(self: &BrowserBridge, placement: &QString);

        /// How the window buttons look: `default` or `macos`.
        #[qinvokable]
        fn window_controls_style(self: &BrowserBridge) -> QString;

        #[qinvokable]
        fn set_window_controls_style(self: &BrowserBridge, style: &QString);

        /// Whether the mouse wheel scrolls smoothly.
        #[qinvokable]
        fn smooth_wheel(self: &BrowserBridge) -> bool;

        #[qinvokable]
        fn set_smooth_wheel(self: &BrowserBridge, is_enabled: bool);

        /// What the About dialog shows, as JSON: the name, version, page titles, comments and the legal sections.
        #[qinvokable]
        fn about_json(self: &BrowserBridge) -> QString;

        /// The interface languages as a JSON array of names; the first is the system's language.
        #[qinvokable]
        fn language_names_json(self: &BrowserBridge) -> QString;

        /// The position in `language_names_json` of the chosen language.
        #[qinvokable]
        fn language_position(self: &BrowserBridge) -> i32;

        /// Chooses a language for the next start; returns whether it changed.
        #[qinvokable]
        fn set_language_position(self: &BrowserBridge, position: i32) -> bool;

        /// Guard's page script with Qt's web-channel client in front of it, or empty when that client is missing.
        #[qinvokable]
        fn guard_page_script(self: &BrowserBridge) -> QString;

        /// The script that removes preconnect, dns-prefetch, prefetch and prerender hints from pages.
        #[qinvokable]
        fn resource_hint_blocking_script(self: &BrowserBridge) -> QString;

        /// The script that rejects cookie notices, to run in the page's own world of every frame.
        #[qinvokable]
        fn cookie_notice_script(self: &BrowserBridge) -> QString;

        /// A notice to show when the window opens, or empty.
        #[qinvokable]
        fn startup_notice(self: &BrowserBridge) -> QString;

        /// Starts asking the desktop which browser opens the web links of other programs; `default_browser_json`
        /// tells once it knows.
        #[qinvokable]
        fn check_default_browser(self: &BrowserBridge);

        /// `{"state": "checking" | "this" | "other" | "unregistered" | "unknown", "browser": name, "error": message}`.
        #[qinvokable]
        fn default_browser_json(self: &BrowserBridge) -> QString;

        /// Starts making ZTBrowser the default browser; `default_browser_json` tells how it went.
        #[qinvokable]
        fn make_default_browser(self: &BrowserBridge);

        /// Whether this window is to offer making ZTBrowser the default browser; one window a process at most.
        #[qinvokable]
        fn offers_default_browser(self: &BrowserBridge) -> bool;

        /// Whether the person wants to be asked, when ZTBrowser starts, to make it the default browser.
        #[qinvokable]
        fn asks_about_default_browser(self: &BrowserBridge) -> bool;

        #[qinvokable]
        fn set_asks_about_default_browser(self: &BrowserBridge, asks: bool);

        /// The home page of the chosen search engine.
        #[qinvokable]
        fn home_page(self: &BrowserBridge) -> QString;

        /// The host of a web address, or empty for a page that is not on the web.
        #[qinvokable]
        fn host_of(self: &BrowserBridge, address: &QString) -> QString;

        /// Whether tracker protection is on for `host` (the shield button's state).
        #[qinvokable]
        fn is_site_protected(self: &BrowserBridge, host: &QString) -> bool;

        #[qinvokable]
        fn set_site_protected(self: &BrowserBridge, host: &QString, is_protected: bool);

        /// The zoom, in percent, that pages at `address` are shown at.
        #[qinvokable]
        fn zoom_percent(self: &BrowserBridge, address: &QString) -> i32;

        /// Zooms the site of `address` in, out or back to actual size (`direction` 1, -1 or 0); returns the percent.
        #[qinvokable]
        fn change_zoom(self: &BrowserBridge, address: &QString, direction: i32) -> i32;

        /// Where a file the site calls `suggested_name` would be saved.
        #[qinvokable]
        fn download_target(self: &BrowserBridge, suggested_name: &QString) -> QString;

        #[qinvokable]
        fn asks_where_to_save(self: &BrowserBridge) -> bool;

        #[qinvokable]
        fn set_asks_where_to_save(self: &BrowserBridge, asks: bool);

        #[qinvokable]
        fn download_folder(self: &BrowserBridge) -> QString;

        #[qinvokable]
        fn set_download_folder(self: &BrowserBridge, folder: &QString);

        /// Lists a download that starts at `path`; returns the number that names it.
        #[qinvokable]
        fn download_begin(self: &BrowserBridge, path: &QString) -> i64;

        /// `total` is zero or less when the site did not say how large the file is.
        #[qinvokable]
        fn download_progress(self: &BrowserBridge, id: i64, received: i64, total: i64);

        #[qinvokable]
        fn download_finish(self: &BrowserBridge, id: i64);

        #[qinvokable]
        fn download_fail(self: &BrowserBridge, id: i64);

        #[qinvokable]
        fn download_remove(self: &BrowserBridge, id: i64);

        #[qinvokable]
        fn download_clear_history(self: &BrowserBridge);

        /// The newest downloads after the one named `cleared_through` (negative: none) as a JSON array, at most
        /// `count` of them (zero: the whole history).
        #[qinvokable]
        fn downloads_json(self: &BrowserBridge, cleared_through: i64, count: i32) -> QString;

        /// Asks for the profile's data to be deleted and restarts the browser; returns an error message, or
        /// empty when the new instance started and this one should quit.
        #[qinvokable]
        fn burn_and_restart(self: &BrowserBridge) -> QString;

        /// Shows a folder in the desktop's file manager; returns whether it was started. Only an existing absolute
        /// folder is opened, never a file: a downloaded file is not launched from here.
        #[qinvokable]
        fn open_folder(self: &BrowserBridge, folder: &QString) -> bool;

        /// Select a downloaded file in the system file manager.
        #[qinvokable]
        fn show_file_in_folder(self: &BrowserBridge, path: &QString) -> bool;
    }
}

use std::path::PathBuf;

use cxx_qt_lib::QString;
use url::Url;
use ztbrowser_core::download::download_history::DownloadHistory;
use ztbrowser_core::download::download_history::DownloadId;
use ztbrowser_core::engine::duckduckgo_extension_prompt_script::DuckDuckGoExtensionPromptScript;
use ztbrowser_core::engine::resource_hint_blocking_script::ResourceHintBlockingScript;
use ztbrowser_core::engine::smooth_wheel_page_script::SmoothWheelPageScript;
use ztbrowser_core::guard::cookie_notice_script::CookieNoticeScript;
use ztbrowser_core::guard::guard_page_script::GuardPageScript;
use ztbrowser_core::language::message_catalogue::MessageCatalogue;
use ztbrowser_core::navigation::address_bar_input::AddressBarInput;
use ztbrowser_core::navigation::search_engine::SearchEngine;
use ztbrowser_core::page_zoom::zoom_level::ZoomLevel;
use ztbrowser_core::scrolling::smooth_wheel_scroll::SmoothWheelScroll;
use ztbrowser_core::web_app::desktop_entry::DesktopEntry;
use ztbrowser_engine_qt::qt_web_engine::QtWebEngine;

use ztbrowser_core::about::about_content::AboutContent;

use crate::burn_restart::BurnRestart;
use crate::default_browser::DefaultBrowserChoice;
use crate::file_manager_request::FileManagerRequest;
use crate::interface_catalogue::InterfaceCatalogue;
use crate::interface_language_setting::InterfaceLanguageSetting;
use crate::launch_context::LaunchContext;
use crate::profile_session::ProfileSession;
use crate::startup_notice::StartupNotice;

#[derive(Default)]
pub struct BrowserBridgeRust;

// QML invokes these on the object, so they take `&self` even where they do not read it.
#[allow(clippy::unused_self)]
impl ffi::BrowserBridge {
    fn destination(&self, typed_text: &QString) -> QString {
        let address = AddressBarInput::new(&typed_text.to_string()).destination(SearchEngine::default());
        QString::from(address.unwrap_or_default().as_str())
    }

    fn message(&self, text: &QString) -> QString {
        let catalogue: &MessageCatalogue = InterfaceCatalogue::current();
        QString::from(catalogue.text(&text.to_string()))
    }

    fn smooth_wheel_script(&self) -> QString {
        let is_smooth = ProfileSession::current().is_none_or(|session| session.preferences().smooth_wheel());
        if is_smooth {
            QString::from(SmoothWheelPageScript::source_for(&SmoothWheelScroll::chromium_like()).as_str())
        } else {
            QString::default()
        }
    }

    fn toolbar_shows(&self, id: &QString) -> bool {
        ProfileSession::current().is_none_or(|session| session.preferences().shows_toolbar_button(&id.to_string()))
    }

    fn set_toolbar_shown(&self, id: &QString, is_shown: bool) {
        if let Some(session) = ProfileSession::current() {
            session.preferences().set_toolbar_button_shown(&id.to_string(), is_shown);
        }
    }

    fn wide_address_bar(&self) -> bool {
        ProfileSession::current().is_none_or(|session| session.preferences().wide_address_bar())
    }

    fn set_wide_address_bar(&self, is_wide: bool) {
        if let Some(session) = ProfileSession::current() {
            session.preferences().set_wide_address_bar(is_wide);
        }
    }

    fn tab_placement(&self) -> QString {
        ProfileSession::current()
            .map_or_else(|| QString::from("top"), |session| QString::from(session.preferences().tab_placement()))
    }

    fn set_tab_placement(&self, placement: &QString) {
        if let Some(session) = ProfileSession::current() {
            session.preferences().set_tab_placement(&placement.to_string());
        }
    }

    fn window_controls_style(&self) -> QString {
        ProfileSession::current().map_or_else(
            || QString::from("default"),
            |session| QString::from(session.preferences().window_controls_style()),
        )
    }

    fn set_window_controls_style(&self, style: &QString) {
        if let Some(session) = ProfileSession::current() {
            session.preferences().set_window_controls_style(&style.to_string());
        }
    }

    fn smooth_wheel(&self) -> bool {
        ProfileSession::current().is_none_or(|session| session.preferences().smooth_wheel())
    }

    fn set_smooth_wheel(&self, is_enabled: bool) {
        if let Some(session) = ProfileSession::current() {
            session.preferences().set_smooth_wheel(is_enabled);
        }
    }

    fn about_json(&self) -> QString {
        let content = AboutContent::build(InterfaceCatalogue::current(), env!("CARGO_PKG_VERSION"));
        QString::from(content.to_string().as_str())
    }

    fn language_names_json(&self) -> QString {
        let system_entry = InterfaceCatalogue::current().text_with(
            "System default ({language})",
            &[("language", InterfaceLanguageSetting::system_language().endonym())],
        );
        QString::from(serde_json::json!(InterfaceLanguageSetting::names(&system_entry)).to_string().as_str())
    }

    fn language_position(&self) -> i32 {
        ProfileSession::current()
            .and_then(|session| i32::try_from(session.language().position()).ok())
            .unwrap_or_default()
    }

    fn set_language_position(&self, position: i32) -> bool {
        let position = usize::try_from(position).unwrap_or_default();
        ProfileSession::current().is_some_and(|session| session.language().choose_position(position))
    }

    fn duckduckgo_extension_prompt_script(&self) -> QString {
        QString::from(DuckDuckGoExtensionPromptScript::source())
    }

    fn resource_hint_blocking_script(&self) -> QString {
        QString::from(ResourceHintBlockingScript::source())
    }

    fn cookie_notice_script(&self) -> QString {
        QString::from(CookieNoticeScript::source())
    }

    fn guard_page_script(&self) -> QString {
        let script = QtWebEngine::resource_text(GuardPageScript::CHANNEL_CLIENT_RESOURCE)
            .map(|client| GuardPageScript::page_script(&client).source);
        QString::from(script.unwrap_or_default().as_str())
    }

    fn startup_notice(&self) -> QString {
        QString::from(StartupNotice::text())
    }

    fn check_default_browser(&self) {
        if let Some(choice) = DefaultBrowserChoice::current() {
            choice.check();
        }
    }

    fn default_browser_json(&self) -> QString {
        let answer = DefaultBrowserChoice::current().map_or_else(
            || serde_json::json!({ "state": "unknown", "browser": "", "error": "" }),
            DefaultBrowserChoice::answer_json,
        );
        QString::from(answer.to_string().as_str())
    }

    fn make_default_browser(&self) {
        if let Some(choice) = DefaultBrowserChoice::current() {
            choice.make_default();
        }
    }

    fn offers_default_browser(&self) -> bool {
        DefaultBrowserChoice::current().is_some_and(DefaultBrowserChoice::takes_offer)
    }

    fn asks_about_default_browser(&self) -> bool {
        DefaultBrowserChoice::current().is_some_and(DefaultBrowserChoice::asks_at_start)
    }

    fn set_asks_about_default_browser(&self, asks: bool) {
        if let Some(choice) = DefaultBrowserChoice::current() {
            choice.set_asks_at_start(asks);
        }
    }

    fn home_page(&self) -> QString {
        QString::from(SearchEngine::default().home_page())
    }

    fn host_of(&self, address: &QString) -> QString {
        let host = Url::parse(&address.to_string())
            .ok()
            .filter(|url| matches!(url.scheme(), "http" | "https"))
            .and_then(|url| url.host_str().map(str::to_owned));
        QString::from(host.unwrap_or_default().as_str())
    }

    fn is_site_protected(&self, host: &QString) -> bool {
        ProfileSession::current().is_none_or(|session| session.protection.is_protected(&host.to_string()))
    }

    fn set_site_protected(&self, host: &QString, is_protected: bool) {
        if let Some(session) = ProfileSession::current() {
            session.protection.set_protected(&host.to_string(), is_protected);
        }
    }

    fn zoom_percent(&self, address: &QString) -> i32 {
        ProfileSession::current().map_or(100, |session| Self::percent(session.zoom().level_for(&address.to_string())))
    }

    fn change_zoom(&self, address: &QString, direction: i32) -> i32 {
        let Some(session) = ProfileSession::current() else { return 100 };
        let address = address.to_string();
        let mut zoom = session.zoom();
        Self::percent(match direction.signum() {
            1 => zoom.zoomed_in(&address),
            -1 => zoom.zoomed_out(&address),
            _ => zoom.reset(&address),
        })
    }

    fn download_target(&self, suggested_name: &QString) -> QString {
        let path = ProfileSession::current()
            .map(|session| session.downloads().proposed_path(&suggested_name.to_string()))
            .unwrap_or_default();
        QString::from(path.to_string_lossy().as_ref())
    }

    fn asks_where_to_save(&self) -> bool {
        ProfileSession::current().is_none_or(|session| session.downloads().asks_where_to_save())
    }

    fn set_asks_where_to_save(&self, asks: bool) {
        if let Some(session) = ProfileSession::current() {
            session.downloads().set_asks_where_to_save(asks);
        }
    }

    fn download_folder(&self) -> QString {
        let folder = ProfileSession::current().map(|session| session.downloads().folder().to_path_buf());
        QString::from(folder.unwrap_or_default().to_string_lossy().as_ref())
    }

    fn set_download_folder(&self, folder: &QString) {
        if let Some(session) = ProfileSession::current() {
            session.downloads().set_folder(PathBuf::from(folder.to_string()));
        }
    }

    fn download_begin(&self, path: &QString) -> i64 {
        ProfileSession::current().map_or(-1, |session| {
            let id = session.downloads().begin(PathBuf::from(path.to_string()));
            i64::try_from(id.value()).unwrap_or(-1)
        })
    }

    fn download_progress(&self, id: i64, received: i64, total: i64) {
        if let (Some(session), Some(id)) = (ProfileSession::current(), Self::download_id(id)) {
            let total = u64::try_from(total).ok().filter(|total| *total > 0);
            session.downloads().progress(id, u64::try_from(received).unwrap_or(0), total);
        }
    }

    fn download_finish(&self, id: i64) {
        if let (Some(session), Some(id)) = (ProfileSession::current(), Self::download_id(id)) {
            session.downloads().finish(id);
        }
    }

    fn download_fail(&self, id: i64) {
        if let (Some(session), Some(id)) = (ProfileSession::current(), Self::download_id(id)) {
            session.downloads().fail(id);
        }
    }

    fn download_remove(&self, id: i64) {
        if let (Some(session), Some(id)) = (ProfileSession::current(), Self::download_id(id)) {
            session.downloads().remove(id);
        }
    }

    fn download_clear_history(&self) {
        if let Some(session) = ProfileSession::current() {
            session.downloads().clear_history();
        }
    }

    fn downloads_json(&self, cleared_through: i64, count: i32) -> QString {
        let rows = ProfileSession::current().map_or_else(Default::default, |session| {
            let count = usize::try_from(count).ok().filter(|count| *count > 0).unwrap_or(DownloadHistory::CAPACITY);
            session.downloads().rows_json(Self::download_id(cleared_through), count)
        });
        QString::from(rows.to_string().as_str())
    }

    fn burn_and_restart(&self) -> QString {
        let Some(session) = ProfileSession::current() else { return QString::from("no profile") };
        // A web app starts again as itself, not as the browser.
        let arguments: Vec<String> = LaunchContext::current()
            .and_then(LaunchContext::web_app)
            .map(|app| vec![DesktopEntry::WEB_APP_SWITCH.to_owned(), app.id.to_string()])
            .unwrap_or_default();
        let started = session.burn.request().and_then(|()| BurnRestart::start_new_instance(&arguments));
        match started {
            Ok(()) => QString::default(),
            Err(error) => QString::from(
                InterfaceCatalogue::current()
                    .text_with("Some data could not be deleted: {error}", &[("error", &error.to_string())])
                    .as_str(),
            ),
        }
    }

    fn open_folder(&self, folder: &QString) -> bool {
        FileManagerRequest::showing_folder(&PathBuf::from(folder.to_string()))
            .map(FileManagerRequest::carry_out)
            .is_some()
    }

    fn show_file_in_folder(&self, path: &QString) -> bool {
        FileManagerRequest::showing_file(&PathBuf::from(path.to_string())).map(FileManagerRequest::carry_out).is_some()
    }

    fn percent(level: ZoomLevel) -> i32 {
        i32::from(level.percent())
    }

    fn download_id(value: i64) -> Option<DownloadId> {
        u64::try_from(value).ok().map(DownloadId::from_value)
    }
}
