//! ZTBrowser Guard: protections on top of DuckDuckGo's, aimed at blocking and hiding more than Brave Shields.
//!
//! DuckDuckGo's tracker blocking (the `protection` module) stays the base. Guard adds community filter lists
//! (ads, trackers, link-click pings, cookie notices), applied to a site's own requests as well as third parties, and
//! switches on fingerprinting defences that DuckDuckGo's macOS configuration leaves to Safari.

pub mod cookie_notice_script;
pub mod filter_list_source;
pub mod fingerprint_hardening;
pub mod generic_hiding;
pub mod generic_hiding_request;
pub mod guard_engine;
pub mod guard_page_script;
pub mod guard_rule_list;
pub mod pending_rule_changes;
pub mod shared_guard_engine;
