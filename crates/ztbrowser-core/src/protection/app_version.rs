//! A dotted application version (`1.214.0`) compared number by number, never alphabetically.

/// Trailing zero components are ignored, so `1.214` equals `1.214.0`. Non-numeric parts count as `0`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AppVersion {
    components: Vec<u64>,
}

impl AppVersion {
    #[must_use]
    pub fn parse(text: &str) -> Self {
        let mut components: Vec<u64> = text.trim().split('.').map(|part| part.parse().unwrap_or(0)).collect();
        while components.last() == Some(&0) {
            components.pop();
        }
        Self { components }
    }
}
