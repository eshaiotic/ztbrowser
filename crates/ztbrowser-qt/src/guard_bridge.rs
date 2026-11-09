//! What a page's Guard script asks Rust: the elements ZTBrowser Guard hides on the page.
//!
//! The script runs in an isolated world of the page, reached through a web channel the page itself cannot see;
//! it names the page's address and the classes and ids found on it, and hides what comes back.

#[cxx_qt::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("cxx-qt-lib/qstring.h");
        type QString = cxx_qt_lib::QString;
    }

    extern "RustQt" {
        #[qobject]
        #[qml_element]
        type GuardBridge = super::GuardBridgeRust;

        /// What to hide at once on the page at `address`, as `{"selectors": [...], "exceptions": [...],
        /// "generic": true}`; empty when protection is off for the site.
        #[qinvokable]
        fn page_hiding(self: &GuardBridge, address: &QString) -> QString;

        /// Whether the cookie notices of the page at `address` are to be rejected: the shield is on for its site.
        #[qinvokable]
        fn rejects_cookie_notices(self: &GuardBridge, address: &QString) -> bool;

        /// The generic selectors needed by `{"classes": [...], "ids": [...], "exceptions": [...]}` found on the
        /// page at `address`, as a JSON array.
        #[qinvokable]
        fn generic_selectors(self: &GuardBridge, address: &QString, found: &QString) -> QString;
    }
}

use cxx_qt_lib::QString;
use serde_json::Value;

use crate::guard_page_hiding::GuardPageHiding;
use crate::profile_session::ProfileSession;

#[derive(Default)]
pub struct GuardBridgeRust;

// QML invokes these on the object, so they take `&self` even where they do not read it.
#[allow(clippy::unused_self)]
impl ffi::GuardBridge {
    fn page_hiding(&self, address: &QString) -> QString {
        let answer = ProfileSession::current()
            .map(|session| GuardPageHiding::new(&session.guard, &session.protection).for_page(&address.to_string()));
        QString::from(answer.unwrap_or(Value::Null).to_string().as_str())
    }

    fn rejects_cookie_notices(&self, address: &QString) -> bool {
        ProfileSession::current().is_some_and(|session| {
            GuardPageHiding::new(&session.guard, &session.protection).rejects_cookie_notices(&address.to_string())
        })
    }

    fn generic_selectors(&self, address: &QString, found: &QString) -> QString {
        let answer = ProfileSession::current().map(|session| {
            GuardPageHiding::new(&session.guard, &session.protection)
                .generic_for_page(&address.to_string(), &found.to_string())
        });
        QString::from(answer.unwrap_or_else(|| Value::Array(Vec::new())).to_string().as_str())
    }
}
