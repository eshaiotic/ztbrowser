//! Which JavaScript world a script runs in.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptWorld {
    /// Shares the page's objects: needed to change what the page sees (fingerprinting protections).
    Page,
    /// A world of its own: the page cannot read or tamper with the script (overlays, smooth scrolling).
    Isolated,
}
