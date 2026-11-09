//! What a request is fetching.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    MainFrame,
    SubFrame,
    Script,
    Stylesheet,
    Image,
    Font,
    Media,
    Xhr,
    WebSocket,
    Other,
}
