//! `QtWebEngine` as the application sees it.

use std::path::Path;
use std::sync::Arc;

use ztbrowser_core::engine::engine_version::EngineVersion;
use ztbrowser_core::network::request_policy::RequestPolicy;
use ztbrowser_core::storage::cookie_policy::CookiePolicy;

use crate::engine_bridge::ffi;
use crate::engine_policies::EnginePolicies;
use crate::gate_proxy::GateProxy;
use crate::upstream_proxy::UpstreamProxy;

pub struct QtWebEngine;

impl QtWebEngine {
    /// Prepares `QtWebEngine`. Call it before the `QGuiApplication` is created.
    ///
    /// Starts the connection proxy first and tells Chromium to use it: connections no request announced
    /// (`preconnect` hints) can be refused only there. Should the proxy not start, Chromium runs without it
    /// and the problem is reported, since the request filters still work.
    pub fn initialize() {
        let upstream = UpstreamProxy::from_variables(|name| std::env::var(name).ok());
        let flags = match GateProxy::start(EnginePolicies::process_gate(), upstream) {
            Ok(proxy) => format!("--proxy-server=http://{}", proxy.address()),
            Err(error) => {
                eprintln!(
                    "ztbrowser: the connection proxy did not start, so connection hints are not filtered: {error}"
                );
                String::new()
            }
        };
        ffi::initialize_web_engine(&flags);
    }

    /// Keeps everything Chromium stores under `storage` and `cache`, and has it ask the policies installed by
    /// [`Self::install_policies`] about every request and cookie. Call it once, after the application exists
    /// and before the first window. Until the policies are installed Chromium refuses every request and
    /// cookie, so a request can never skip protection because it came first.
    pub fn open_profile(storage: &Path, cache: &Path) {
        ffi::install_hooks(&storage.to_string_lossy(), &cache.to_string_lossy());
    }

    /// Makes `request` and `cookies` the policies Chromium consults. A second call changes nothing and
    /// returns `false`.
    pub fn install_policies(request: Arc<dyn RequestPolicy>, cookies: Arc<dyn CookiePolicy>) -> bool {
        EnginePolicies::with_gate(request, cookies, EnginePolicies::process_gate()).install()
    }

    /// Gives every window the icon stored at `resource_path` in the Qt resources. Call it once the application
    /// exists.
    pub fn set_window_icon(resource_path: &str) {
        ffi::set_window_icon(resource_path);
    }

    /// The text of the Qt resource at `path` (`:/qtwebchannel/qwebchannel.js`); `None` when there is none.
    #[must_use]
    pub fn resource_text(path: &str) -> Option<String> {
        Some(ffi::read_qt_resource(path)).filter(|text| !text.is_empty())
    }

    /// The PNG `bytes` decoded and encoded again by Qt; `None` when Qt cannot decode them.
    #[must_use]
    pub fn reencode_png(bytes: &[u8]) -> Option<Vec<u8>> {
        Some(ffi::reencode_png(bytes)).filter(|png| !png.is_empty())
    }

    /// The Chromium version inside the installed `QtWebEngine`; `None` if it cannot be read.
    #[must_use]
    pub fn chromium_version() -> Option<EngineVersion> {
        EngineVersion::parse(&ffi::chromium_version())
    }
}
