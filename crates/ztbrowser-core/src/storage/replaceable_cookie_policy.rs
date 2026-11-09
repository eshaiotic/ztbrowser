//! A cookie policy that can be swapped for another while cookies keep being judged (see
//! `ReplaceableRequestPolicy`).

use std::sync::Arc;
use std::sync::PoisonError;
use std::sync::RwLock;

use crate::storage::cookie_access::CookieAccess;
use crate::storage::cookie_policy::CookiePolicy;

pub struct ReplaceableCookiePolicy {
    current: RwLock<Arc<dyn CookiePolicy>>,
}

impl ReplaceableCookiePolicy {
    #[must_use]
    pub fn new(initial: Arc<dyn CookiePolicy>) -> Self {
        Self { current: RwLock::new(initial) }
    }

    /// Cookies judged from now on are judged by `policy`.
    pub fn replace(&self, policy: Arc<dyn CookiePolicy>) {
        *self.current.write().unwrap_or_else(PoisonError::into_inner) = policy;
    }
}

impl CookiePolicy for ReplaceableCookiePolicy {
    fn allows(&self, access: &CookieAccess) -> bool {
        let policy = Arc::clone(&self.current.read().unwrap_or_else(PoisonError::into_inner));
        policy.allows(access)
    }
}
