//! How much of a profile's data to burn.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BurnScope {
    /// Everything the profile stored.
    Everything,
    /// Only what one site (and its subdomains) stored.
    Site(String),
}
