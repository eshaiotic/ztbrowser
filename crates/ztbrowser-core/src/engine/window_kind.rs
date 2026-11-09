//! Which kind of window an engine is started for.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowKind {
    Normal,
    Tor,
}
