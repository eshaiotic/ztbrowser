//! The shield button: which sites protection is switched off for.

use tempfile::TempDir;
use ztbrowser_qt::site_protection::SiteProtection;

use ztbrowser_qt as _;

#[test]
fn should_protect_every_site_until_told_otherwise() {
    let profile = TempDir::new().expect("profile");
    assert!(SiteProtection::open(profile.path()).is_protected("example.com"));
}

#[test]
fn should_remember_an_unprotected_site_after_a_restart() {
    let profile = TempDir::new().expect("profile");
    SiteProtection::open(profile.path()).set_protected("www.example.com", false);
    let reopened = SiteProtection::open(profile.path());
    assert!(!reopened.is_protected("example.com"));
}

#[test]
fn should_protect_the_site_again_when_switched_back_on() {
    let profile = TempDir::new().expect("profile");
    let protection = SiteProtection::open(profile.path());
    protection.set_protected("example.com", false);
    protection.set_protected("example.com", true);
    assert!(SiteProtection::open(profile.path()).is_protected("example.com"));
}

#[test]
fn should_show_a_change_to_the_list_the_policies_hold() {
    let profile = TempDir::new().expect("profile");
    let protection = SiteProtection::open(profile.path());
    let shared = protection.shared_list();
    protection.set_protected("example.com", false);
    assert!(shared.read().expect("lock").is_unprotected("example.com"));
}
