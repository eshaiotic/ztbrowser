//! The C++/Rust boundary: the only place where Chromium calls into Rust and Rust calls into Qt.
//!
//! `cxx` generates `unsafe` code inside the bridge module; nothing here is written by hand.

#[cxx_qt::bridge]
pub mod ffi {
    /// What the interceptor does with a request: `action` 0 = let it pass, 1 = block, 2 = redirect.
    struct HookDecision {
        action: u8,
        redirect_to: String,
    }

    /// The answer to a document request: `status` 0 means no answer came (`error` says why), 304 means the copy
    /// the validator names is still current.
    struct FetchedBytes {
        status: i32,
        etag: String,
        body: Vec<u8>,
        error: String,
    }

    extern "Rust" {
        fn decide_request(url: &str, first_party: &str, resource_type: i32, request_id: u64) -> HookDecision;
        fn allow_cookie(domain: &str, first_party: &str, is_third_party: bool) -> bool;
    }

    unsafe extern "C++" {
        include!("engine_hooks.h");
        fn initialize_web_engine(chromium_flags: &str);
        fn install_hooks(storage_path: &str, cache_path: &str);
        fn chromium_version() -> String;
        fn set_window_icon(resource_path: &str);
        fn read_qt_resource(path: &str) -> String;
        fn reencode_png(bytes: &[u8]) -> Vec<u8>;
        fn fetch_bytes(url: &str, etag: &str, max_bytes: u64, timeout_ms: i32) -> FetchedBytes;
    }
}

use crate::engine_policies::EnginePolicies;
use crate::engine_policies::PolicyDecisions;

fn decide_request(url: &str, first_party: &str, resource_type: i32, request_id: u64) -> ffi::HookDecision {
    PolicyDecisions::for_request(EnginePolicies::installed(), url, first_party, resource_type, request_id)
}

fn allow_cookie(domain: &str, first_party: &str, is_third_party: bool) -> bool {
    PolicyDecisions::for_cookie(EnginePolicies::installed(), domain, first_party, is_third_party)
}
