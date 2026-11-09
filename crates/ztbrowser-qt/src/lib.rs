//! ZTBrowser 1.2: the Qt window on `QtWebEngine`. See `docs/en/specs/2026-11-04-qtwebengine-migration-design.md`.

pub mod appimage_integration;
pub mod browser_application;
// `cxx-qt` generates `unsafe` code and a boxed constructor in its bridge; this is the only module that may contain them.
#[allow(unsafe_code, clippy::unnecessary_box_returns)]
pub mod browser_bridge;
pub mod burn_restart;
pub mod default_browser;
pub mod desktop_font;
pub mod download_center;
pub mod file_manager_request;
// `cxx-qt` bridge, like `browser_bridge`.
#[allow(unsafe_code, clippy::unnecessary_box_returns)]
pub mod guard_bridge;
pub mod guard_lists;
pub mod guard_page_hiding;
pub mod induckgo_migration;
pub mod installation_directories;
pub mod installed_profiles;
pub mod installed_web_apps;
pub mod interface_catalogue;
pub mod interface_language_setting;
pub mod launch_context;
// `cxx-qt` bridge, like `browser_bridge`.
#[allow(unsafe_code, clippy::unnecessary_box_returns)]
pub mod profile_bridge;
pub mod profile_policies;
pub mod profile_preferences;
pub mod profile_session;
pub mod protection_documents;
pub mod protection_updates;
pub mod single_instance;
pub mod site_protection;
pub mod site_zoom_store;
pub mod startup_notice;
pub mod system_downloads_folder;
pub mod system_locales;
// `cxx-qt` bridge, like `browser_bridge`.
#[allow(unsafe_code, clippy::unnecessary_box_returns)]
pub mod web_app_bridge;
pub mod web_app_installer;
