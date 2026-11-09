//! Zoom is per site: every page of a site is drawn at the size the person gave it.

use tempfile::TempDir;
use ztbrowser_qt::site_zoom_store::SiteZoomStore;

use ztbrowser_qt as _;

#[test]
fn should_open_pages_at_actual_size_until_zoomed() {
    let profile = TempDir::new().expect("profile");
    assert_eq!(SiteZoomStore::open(profile.path()).level_for("https://example.com/").percent(), 100_u16);
}

#[test]
fn should_give_every_page_of_the_site_the_new_size() {
    let profile = TempDir::new().expect("profile");
    let mut store = SiteZoomStore::open(profile.path());
    let zoomed = store.zoomed_in("https://www.example.com/a");
    assert_eq!(store.level_for("https://shop.example.com/b"), zoomed);
}

#[test]
fn should_keep_other_sites_at_actual_size() {
    let profile = TempDir::new().expect("profile");
    let mut store = SiteZoomStore::open(profile.path());
    let _ = store.zoomed_in("https://example.com/");
    assert_eq!(store.level_for("https://example.org/").percent(), 100_u16);
}

#[test]
fn should_remember_the_size_after_a_restart() {
    let profile = TempDir::new().expect("profile");
    let zoomed = SiteZoomStore::open(profile.path()).zoomed_out("https://example.com/");
    assert_eq!(SiteZoomStore::open(profile.path()).level_for("https://example.com/"), zoomed);
}

#[test]
fn should_return_to_actual_size_on_reset() {
    let profile = TempDir::new().expect("profile");
    let mut store = SiteZoomStore::open(profile.path());
    let _ = store.zoomed_in("https://example.com/");
    assert_eq!(store.reset("https://example.com/").percent(), 100_u16);
}

#[test]
fn should_not_zoom_pages_that_are_not_on_the_web() {
    let profile = TempDir::new().expect("profile");
    assert_eq!(SiteZoomStore::open(profile.path()).zoomed_in("about:blank").percent(), 100_u16);
}
