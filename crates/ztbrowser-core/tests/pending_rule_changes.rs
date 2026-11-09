//! A rule change (the user's shield, a list refresh) takes effect in two steps: rules active at once, with
//! Guard's previous copy or without Guard, then Guard's recompiled lists seconds later. Rebuilds overlap, and a
//! newer one supersedes an older one, so what waits for a change (the page's reload) is kept across rebuilds.

use ztbrowser_core::guard::pending_rule_changes::PendingRuleChanges;
use ztbrowser_core::guard::pending_rule_changes::RuleChange;

struct ChangesScenario {
    pending: PendingRuleChanges<&'static str>,
}

impl ChangesScenario {
    fn given_no_change() -> Self {
        Self { pending: PendingRuleChanges::default() }
    }

    fn given(change: RuleChange, waiter: &'static str) -> Self {
        Self::given_no_change().and(change, waiter)
    }

    fn and(mut self, change: RuleChange, waiter: &'static str) -> Self {
        self.pending.record(change, waiter);
        self
    }

    fn when_rules_activate_with_guard_still_compiling(&mut self) -> Vec<&'static str> {
        self.pending.settled(false)
    }

    fn when_guard_is_compiled(&mut self) -> Vec<&'static str> {
        self.pending.settled(true)
    }

    fn then_leaves_guard_out(&self) {
        assert!(self.pending.leaves_guard_out(), "Guard's previous copy would stay on");
    }

    fn then_keeps_guard_on(&self) {
        assert!(!self.pending.leaves_guard_out(), "Guard left out");
    }
}

#[test]
fn should_reload_at_once_when_a_site_is_unprotected() {
    let mut scenario = ChangesScenario::given(RuleChange::SiteUnprotected, "reload bank");

    assert_eq!(scenario.when_rules_activate_with_guard_still_compiling(), ["reload bank"]);
}

#[test]
fn should_leave_guard_out_while_an_unprotected_site_compiles() {
    let scenario = ChangesScenario::given(RuleChange::SiteUnprotected, "reload bank");

    scenario.then_leaves_guard_out();
}

#[test]
fn should_bring_guard_back_once_compiled() {
    let mut scenario = ChangesScenario::given(RuleChange::SiteUnprotected, "reload bank");

    scenario.when_guard_is_compiled();

    scenario.then_keeps_guard_on();
}

#[test]
fn should_keep_guard_out_through_a_refresh_that_overtakes_the_unprotected_site() {
    let mut scenario =
        ChangesScenario::given(RuleChange::SiteUnprotected, "reload bank").and(RuleChange::Refresh, "nothing");

    scenario.when_rules_activate_with_guard_still_compiling();

    scenario.then_leaves_guard_out();
}

#[test]
fn should_hold_the_reload_of_a_protected_site_until_guard_is_compiled() {
    let mut scenario = ChangesScenario::given(RuleChange::SiteProtected, "reload shop");

    assert_eq!(scenario.when_rules_activate_with_guard_still_compiling(), Vec::<&str>::new());
}

#[test]
fn should_reload_a_protected_site_once_guard_is_compiled() {
    let mut scenario = ChangesScenario::given(RuleChange::SiteProtected, "reload shop");
    scenario.when_rules_activate_with_guard_still_compiling();

    assert_eq!(scenario.when_guard_is_compiled(), ["reload shop"]);
}

#[test]
fn should_keep_the_previous_guard_copy_for_a_protected_site() {
    let scenario = ChangesScenario::given(RuleChange::SiteProtected, "reload shop");

    scenario.then_keeps_guard_on();
}

#[test]
fn should_keep_a_reload_promised_before_a_refresh_overtook_it() {
    let mut scenario =
        ChangesScenario::given(RuleChange::SiteProtected, "reload shop").and(RuleChange::Refresh, "nothing");

    assert_eq!(scenario.when_guard_is_compiled(), ["nothing", "reload shop"]);
}

#[test]
fn should_run_each_waiter_once() {
    let mut scenario = ChangesScenario::given(RuleChange::Refresh, "reload page");
    scenario.when_rules_activate_with_guard_still_compiling();

    assert_eq!(scenario.when_guard_is_compiled(), Vec::<&str>::new());
}
