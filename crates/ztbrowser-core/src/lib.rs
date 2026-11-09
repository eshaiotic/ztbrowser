//! Browser-independent domain logic of ZTBrowser.
//!
//! Nothing in this crate knows about GTK or WebKit: it turns DuckDuckGo's open data (Tracker Data Set,
//! privacy configuration) and user intent (typed addresses, profiles, installed web apps) into plain
//! values the `ztbrowser` application hands to WebKitGTK. That keeps the rules fast to test and easy to
//! audit.

pub mod about;
pub mod document;
pub mod download;
pub mod engine;
pub mod guard;
pub mod identity;
pub mod language;
pub mod launch;
pub mod navigation;
pub mod network;
pub mod page_zoom;
pub mod profile;
pub mod protection;
pub mod rendering;
pub mod scrolling;
pub mod storage;
pub mod tor;
pub mod web_app;
