//! Time as the connection gate needs it: how long since some fixed starting point, never the wall clock.

use std::time::Duration;
use std::time::Instant;

/// `Send + Sync` because the engine's request threads and its connection threads read it at once.
pub trait ConnectionClock: Send + Sync {
    fn elapsed(&self) -> Duration;
}

/// The monotonic clock of the operating system, counted from the moment this clock was created.
pub struct SystemConnectionClock {
    started: Instant,
}

impl SystemConnectionClock {
    #[must_use]
    pub fn started_now() -> Self {
        Self { started: Instant::now() }
    }
}

impl ConnectionClock for SystemConnectionClock {
    fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
}
