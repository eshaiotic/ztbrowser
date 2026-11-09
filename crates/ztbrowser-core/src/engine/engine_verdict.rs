//! Whether the installed engine is recent enough to start.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineVerdict {
    Run,
    /// Start, but show a notice that security fixes may be missing.
    Warn,
    /// Do not start.
    Refuse,
}
