//! Translates `QtWebEngine`'s resource-type numbers (`QWebEngineUrlRequestInfo::ResourceType`) to the core's.

use ztbrowser_core::network::resource_type::ResourceType;

pub struct ResourceTypeMapping;

impl ResourceTypeMapping {
    /// Numbers are the enum values of Qt 6; unknown values map to `Other`, never to a more permissive type.
    #[must_use]
    pub fn from_engine_code(code: i32) -> ResourceType {
        match code {
            0 | 19 => ResourceType::MainFrame,
            1 | 20 => ResourceType::SubFrame,
            2 => ResourceType::Stylesheet,
            3 => ResourceType::Script,
            4 | 12 => ResourceType::Image,
            5 => ResourceType::Font,
            8 => ResourceType::Media,
            13 | 14 => ResourceType::Xhr,
            _ => ResourceType::Other,
        }
    }
}
