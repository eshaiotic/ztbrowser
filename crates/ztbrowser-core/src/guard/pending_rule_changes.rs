//! Rule changes not fully in effect yet, and what waits for them.
//!
//! A change takes effect in two steps. Rules go active within a second: the tracker list and the quick lists
//! rebuilt, Guard's lists in their previous compiled copy, or left out. Guard's recompiled lists follow seconds
//! later. Rebuilds overlap: a background list refresh can start while the lists of the user's last click still
//! compile, and the newer rebuild supersedes the older one. So what a change promised (the page's reload,
//! Guard left out) is kept here, by the profile, and the rebuild that activates rules keeps the promise,
//! whichever rebuild that is.

/// What a rebuild of the rule lists follows from, as far as Guard's previous compiled copy is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleChange {
    /// Anything Guard's previous copy still suits meanwhile: fresh tracker data or filter lists, an HTTPS
    /// exemption.
    Refresh,
    /// The user switched protection off for a site, which Guard's previous copy would go on blocking: until
    /// recompiled, Guard is left out.
    SiteUnprotected,
    /// The user switched protection back on for a site, which Guard's previous copy still exempts: its waiter
    /// (the reload) waits for the recompiled lists, so the page does not come back unguarded.
    SiteProtected,
}

pub struct PendingRuleChanges<T> {
    leaves_guard_out: bool,
    once_active: Vec<T>,
    once_guard_compiled: Vec<T>,
}

impl<T> Default for PendingRuleChanges<T> {
    fn default() -> Self {
        Self { leaves_guard_out: false, once_active: Vec::new(), once_guard_compiled: Vec::new() }
    }
}

impl<T> PendingRuleChanges<T> {
    pub fn record(&mut self, change: RuleChange, waiter: T) {
        match change {
            RuleChange::Refresh => self.once_active.push(waiter),
            RuleChange::SiteUnprotected => {
                self.leaves_guard_out = true;
                self.once_active.push(waiter);
            }
            RuleChange::SiteProtected => self.once_guard_compiled.push(waiter),
        }
    }

    /// Whether rules activated before Guard's lists are recompiled must leave them out.
    #[must_use]
    pub fn leaves_guard_out(&self) -> bool {
        self.leaves_guard_out
    }

    /// The waiters satisfied by the rules just activated, in the order they came; `guard_compiled` when every
    /// Guard list among them was compiled from the current inputs.
    pub fn settled(&mut self, guard_compiled: bool) -> Vec<T> {
        let mut settled = std::mem::take(&mut self.once_active);
        if guard_compiled {
            self.leaves_guard_out = false;
            settled.append(&mut self.once_guard_compiled);
        }
        settled
    }
}
