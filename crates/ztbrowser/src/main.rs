//! ZTBrowser: a privacy-first WebKitGTK browser for Linux built on DuckDuckGo's open-source protections.

#[macro_use]
mod interface_messages;
mod app_context;
mod app_icons;
mod browser_window;
mod browser_windows;
mod catalog_files;
mod closed_proxy;
mod download_center;
mod download_row;
mod download_tracker;
mod downloads_list;
mod downloads_manager;
mod error_page_view;
mod external_launcher;
mod feed_document;
mod filter_list_feed;
mod find_bar;
mod graphics_renderer;
mod http_client;
mod installation_paths;
mod interface_language_setting;
mod main_menu;
mod management_dialogs;
mod page_protections;
mod page_view;
mod permission_prompt;
mod preferences_dialog;
mod profile_session;
mod protection_feed;
mod public_suffix_list;
mod running_instances;
mod site_zoom;
mod smooth_wheel_script;
mod stored_file;
mod toolbar_preferences;
mod tor_connection;
mod tor_connection_page;
mod tor_control_port;
mod tor_settings_dialog;
mod web_app_installer;
mod web_app_window;
mod zoom_controls;
mod ztbrowser_application;

use crate::graphics_renderer::GraphicsRenderer;
use crate::ztbrowser_application::ZTBrowserApplication;

fn main() -> glib::ExitCode {
    GraphicsRenderer::prefer_opengl();
    ZTBrowserApplication::run()
}
