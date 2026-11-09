//! Answers a page's questions about what Guard hides on it, with limits on everything the page can send.
//!
//! The classes and ids come from a page that may be hostile, so their number and length are capped, and nothing
//! is answered for a site the person switched protection off for.

use serde_json::Value;
use serde_json::json;
use url::Url;
use ztbrowser_core::guard::shared_guard_engine::SharedGuardEngine;

use crate::site_protection::SiteProtection;

pub struct GuardPageHiding<'a> {
    guard: &'a SharedGuardEngine,
    protection: &'a SiteProtection,
}

impl<'a> GuardPageHiding<'a> {
    /// More classes and ids than any page uses; the rest of a longer report is ignored.
    pub const MAX_NAMES: usize = 20_000;
    const MAX_NAME_LENGTH: usize = 256;

    #[must_use]
    pub fn new(guard: &'a SharedGuardEngine, protection: &'a SiteProtection) -> Self {
        Self { guard, protection }
    }

    /// `{"selectors": [...], "exceptions": [...], "generic": bool}` for the page at `address`, or `null` when
    /// there is nothing to hide: no engine yet, a page that is not on the web, or a site without protection.
    #[must_use]
    pub fn for_page(&self, address: &str) -> Value {
        let Some(engine) = self.guard.get().filter(|_| self.is_protected_page(address)) else { return Value::Null };
        let cosmetics = engine.page_cosmetics(address);
        let mut exceptions: Vec<String> = cosmetics.exceptions.into_iter().collect();
        exceptions.sort_unstable();
        json!({
            "selectors": cosmetics.hide_selectors,
            "exceptions": exceptions,
            "generic": !cosmetics.generic_hiding_is_off,
        })
    }

    /// The generic selectors for the report `{"classes": [...], "ids": [...], "exceptions": [...]}` of the page
    /// at `address`; an empty array when there is nothing to answer or the report is not understood.
    #[must_use]
    pub fn generic_for_page(&self, address: &str, report: &str) -> Value {
        let Some(engine) = self.guard.get().filter(|_| self.is_protected_page(address)) else {
            return Value::Array(Vec::new());
        };
        let Ok(report) = serde_json::from_str::<Value>(report) else { return Value::Array(Vec::new()) };
        let classes = Self::names(&report, "classes");
        let ids = Self::names(&report, "ids");
        let exceptions = Self::names(&report, "exceptions").into_iter().map(str::to_owned).collect();
        json!(engine.generic_selectors(&classes, &ids, &exceptions))
    }

    /// Whether the cookie notices of the page at `address` (the top page, when asked from a frame) are rejected:
    /// they are wherever the shield is on, with or without Guard's lists, and never on a site it is off for.
    #[must_use]
    pub fn rejects_cookie_notices(&self, address: &str) -> bool {
        self.is_protected_page(address)
    }

    fn is_protected_page(&self, address: &str) -> bool {
        Url::parse(address)
            .ok()
            .filter(|url| matches!(url.scheme(), "http" | "https"))
            .and_then(|url| url.host_str().map(str::to_owned))
            .is_some_and(|host| self.protection.is_protected(&host))
    }

    fn names<'r>(report: &'r Value, key: &str) -> Vec<&'r str> {
        report
            .get(key)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|name| name.len() <= Self::MAX_NAME_LENGTH)
            .take(Self::MAX_NAMES)
            .collect()
    }
}
