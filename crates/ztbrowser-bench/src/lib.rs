//! Scroll benchmark shared by every engine ZTBrowser has used or may use (WebKitGTK, `QtWebEngine`, Brave).
//!
//! The tool serves a real web page with a small script injected. The script scrolls the page for a fixed
//! time and posts every animation-frame gap back; the browser under test only needs to open one URL, so the
//! same measurement applies to all engines (see `docs/en/specs/2026-11-04-qtwebengine-migration-design.md` §8).

pub mod benchmark_command;
pub mod benchmark_page;
pub mod benchmark_server;
pub mod frame_statistics;
pub mod scroll_scenario;
