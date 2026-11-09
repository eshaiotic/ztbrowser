//! When in a page's life a script runs.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectionPoint {
    /// Before any page script: the document exists but is empty.
    DocumentCreation,
    /// When the document has been parsed.
    DocumentReady,
}
