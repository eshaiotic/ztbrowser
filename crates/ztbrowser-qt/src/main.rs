//! Starts ZTBrowser's Qt window; see the library for the parts.

use std::process::ExitCode;

use ztbrowser_qt::browser_application::BrowserApplication;

fn main() -> ExitCode {
    BrowserApplication::run()
}
