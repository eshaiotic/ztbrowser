//! A rule about cookies that the engine adapter asks for every cookie access.

use crate::storage::cookie_access::CookieAccess;

pub trait CookiePolicy: Send + Sync {
    fn allows(&self, access: &CookieAccess) -> bool;
}
