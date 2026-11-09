//! What the browser does with one request.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestDecision {
    /// Send the request unchanged.
    Pass,
    /// Never send it.
    Block,
    /// Send this address instead.
    Redirect(String),
}

impl RequestDecision {
    #[must_use]
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass)
    }
}
