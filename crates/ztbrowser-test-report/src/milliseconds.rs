//! A duration in milliseconds, printed the way the report shows times: `850 ms`, `2.6s`, `3m 4s`, `1h 2m`.

use std::fmt;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct Milliseconds(f64);

impl Milliseconds {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn from_seconds(seconds: f64) -> Self {
        Self(seconds * 1000.0)
    }

    pub fn value(self) -> f64 {
        self.0
    }
}

impl fmt::Display for Milliseconds {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let milliseconds = self.0;
        if milliseconds <= 0.0 {
            return formatter.write_str("—");
        }
        if milliseconds < 1000.0 {
            return write!(formatter, "{milliseconds:.0} ms");
        }
        let seconds = milliseconds / 1000.0;
        if seconds < 60.0 {
            return write!(formatter, "{seconds:.1}s");
        }
        let minutes = (seconds / 60.0).floor();
        if minutes < 60.0 {
            return write!(formatter, "{minutes:.0}m {:.0}s", (seconds % 60.0).floor());
        }
        write!(formatter, "{:.0}h {:.0}m", (minutes / 60.0).floor(), minutes % 60.0)
    }
}
