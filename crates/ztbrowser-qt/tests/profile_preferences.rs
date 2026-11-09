//! The choices of the preferences window outlive the window and the process.

use tempfile::TempDir;
use ztbrowser_qt::profile_preferences::ProfilePreferences;

use ztbrowser_qt as _;

#[test]
fn should_show_every_toolbar_button_until_one_is_hidden() {
    let profile = TempDir::new().expect("profile");
    assert!(ProfilePreferences::open(profile.path()).shows_toolbar_button("reload"));
}

#[test]
fn should_hide_a_button_and_remember_it_after_a_restart() {
    let profile = TempDir::new().expect("profile");
    ProfilePreferences::open(profile.path()).set_toolbar_button_shown("reload", false);
    let reopened = ProfilePreferences::open(profile.path());
    assert!(!reopened.shows_toolbar_button("reload"));
    assert!(reopened.shows_toolbar_button("newTab"));
}

#[test]
fn should_ignore_a_button_nobody_offers() {
    let profile = TempDir::new().expect("profile");
    let mut preferences = ProfilePreferences::open(profile.path());
    preferences.set_toolbar_button_shown("teleport", false);
    assert!(preferences.shows_toolbar_button("teleport"));
}

#[test]
fn should_scroll_smoothly_until_turned_off() {
    let profile = TempDir::new().expect("profile");
    assert!(ProfilePreferences::open(profile.path()).smooth_wheel());
}

#[test]
fn should_remember_that_smooth_scrolling_was_turned_off() {
    let profile = TempDir::new().expect("profile");
    ProfilePreferences::open(profile.path()).set_smooth_wheel(false);
    assert!(!ProfilePreferences::open(profile.path()).smooth_wheel());
}
