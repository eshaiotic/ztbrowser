//! `QtWebEngine` adapter: carries out the core's request, cookie and script decisions inside Chromium.
//!
//! Chromium calls the interceptor and the cookie filter on its own threads, so the policies it consults are
//! `Send + Sync` and installed once per process (see `engine_policies`).

// `cxx` generates `unsafe` code in the bridge; this is the only module that may contain it.
#[allow(unsafe_code)]
pub mod engine_bridge;
pub mod engine_policies;
pub mod gate_proxy;
pub mod qt_document_fetcher;
pub mod qt_web_engine;
pub mod resource_type_mapping;
pub mod upstream_proxy;
