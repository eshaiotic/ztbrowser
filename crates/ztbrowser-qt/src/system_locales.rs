//! The languages the desktop asks for, from the environment variables gettext programs read.

use std::env;

pub struct SystemLocales;

impl SystemLocales {
    /// `LANGUAGE` (a colon-separated list, strongest preference first), then `LC_ALL`, `LC_MESSAGES`, `LANG`.
    #[must_use]
    pub fn from_environment() -> Vec<String> {
        let language_list = env::var("LANGUAGE").unwrap_or_default();
        let listed = language_list.split(':').map(str::to_owned);
        let single = ["LC_ALL", "LC_MESSAGES", "LANG"].into_iter().filter_map(|variable| env::var(variable).ok());
        listed.chain(single).filter(|locale| !locale.is_empty() && locale != "C" && locale != "POSIX").collect()
    }
}
