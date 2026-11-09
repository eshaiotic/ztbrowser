//! The Guard engine as the request policy and the page-hiding bridge share it.
//!
//! Building the engine from the downloaded lists takes a moment, so it is built on a worker thread while the
//! browser already runs; until it is installed Guard blocks and hides nothing. Lists that are refreshed while
//! the browser runs install a new engine, and requests decided from then on use it.

use std::sync::Arc;
use std::sync::PoisonError;
use std::sync::RwLock;

use crate::guard::guard_engine::GuardEngine;

#[derive(Default)]
pub struct SharedGuardEngine {
    engine: RwLock<Option<Arc<GuardEngine>>>,
}

impl SharedGuardEngine {
    /// Makes `engine` the one in use, replacing the previous one.
    pub fn install(&self, engine: GuardEngine) {
        *self.engine.write().unwrap_or_else(PoisonError::into_inner) = Some(Arc::new(engine));
    }

    /// The engine in use, `None` until the first one is installed.
    #[must_use]
    pub fn get(&self) -> Option<Arc<GuardEngine>> {
        self.engine.read().unwrap_or_else(PoisonError::into_inner).clone()
    }
}
