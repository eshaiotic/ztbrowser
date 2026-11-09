//! A Chromium version number such as `140.0.7339.207`.

use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EngineVersion {
    parts: [u32; 4],
}

impl EngineVersion {
    /// Missing trailing parts count as zero (`140` is `140.0.0.0`). `None` when a part is not a number or
    /// there are more than four.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut parts = [0_u32; 4];
        let mut found = text.trim().split('.');
        for slot in &mut parts {
            match found.next() {
                Some(part) => *slot = part.parse().ok()?,
                None => break,
            }
        }
        found.next().is_none().then_some(Self { parts })
    }

    #[must_use]
    pub fn major(&self) -> u32 {
        self.parts[0]
    }
}

impl PartialOrd for EngineVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EngineVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.parts.cmp(&other.parts)
    }
}

impl fmt::Display for EngineVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c, d] = self.parts;
        write!(formatter, "{a}.{b}.{c}.{d}")
    }
}
