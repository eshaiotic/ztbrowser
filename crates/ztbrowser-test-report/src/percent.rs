//! Percentages as the report prints them: truncated to two decimals, never rounded up, so a run with one
//! failure among thousands is never shown as 100.00 %.

use std::fmt;

/// A share in percent, truncated to two decimals.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Percent(f64);

impl Percent {
    /// `part` of `whole` in percent; an empty whole is 0 %.
    pub fn of(part: u32, whole: u32) -> Self {
        if whole == 0 {
            return Self(0.0);
        }
        Self::from_value(f64::from(part) / f64::from(whole) * 100.0)
    }

    pub fn from_value(value: f64) -> Self {
        Self((value * 100.0 + 1e-6).floor() / 100.0)
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Percent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:.2}%", self.0)
    }
}
