//! Reading a site's web app manifest and picking the sharpest icon for the application menu.

use serde_json::json;
use ztbrowser_core::web_app::site_icon::IconSource;
use ztbrowser_core::web_app::site_icon::SiteIcon;
use ztbrowser_core::web_app::site_icon_choice::SiteIconChoice;
use ztbrowser_core::web_app::web_app_manifest::WebAppManifest;

struct ManifestScenario {
    manifest: WebAppManifest,
}

impl ManifestScenario {
    fn given_manifest_published_at(url: &str, document: &serde_json::Value) -> Self {
        Self { manifest: WebAppManifest::from_json(document, url) }
    }

    fn given_gmail_manifest() -> Self {
        Self::given_manifest_published_at(
            "https://mail.google.com/mail/manifest.json",
            &json!({
                "name": "Gmail",
                "short_name": "Gmail",
                "start_url": "/mail/u/0/",
                "icons": [
                    {"src": "icons/192.png", "sizes": "192x192", "type": "image/png"},
                    {"src": "https://www.gstatic.com/512.png", "sizes": "256x256 512x512"},
                    {"src": "/maskable.png", "sizes": "1024x1024", "purpose": "maskable"}
                ]
            }),
        )
    }
}

struct IconChoiceScenario {
    icons: Vec<SiteIcon>,
}

impl IconChoiceScenario {
    fn given_page_icons(icons: Vec<SiteIcon>) -> Self {
        Self { icons }
    }

    fn when_choosing_application_icon(&self) -> Option<&str> {
        SiteIconChoice::new(&self.icons).best().map(|icon| icon.url.as_str())
    }
}

#[test]
fn should_prefer_short_name_for_the_application_menu() {
    assert_eq!(ManifestScenario::given_gmail_manifest().manifest.display_name(), Some("Gmail"));
}

#[test]
fn should_resolve_start_url_against_manifest_address() {
    assert_eq!(
        ManifestScenario::given_gmail_manifest().manifest.start_url.as_deref(),
        Some("https://mail.google.com/mail/u/0/")
    );
}

#[test]
fn should_resolve_icon_addresses_and_read_their_largest_size() {
    let icons: Vec<(String, Option<u32>)> = ManifestScenario::given_gmail_manifest()
        .manifest
        .icons
        .iter()
        .map(|icon| (icon.url.clone(), icon.size))
        .collect();

    assert_eq!(
        icons,
        [
            ("https://mail.google.com/mail/icons/192.png".to_owned(), Some(192)),
            ("https://www.gstatic.com/512.png".to_owned(), Some(512)),
            ("https://mail.google.com/maskable.png".to_owned(), Some(1024)),
        ]
    );
}

#[test]
fn should_ignore_start_url_on_another_site() {
    let scenario = ManifestScenario::given_manifest_published_at(
        "https://shop.com/manifest.json",
        &json!({"name": "Shop", "start_url": "https://evil.com/"}),
    );

    assert_eq!(scenario.manifest.start_url, None);
}

#[test]
fn should_choose_largest_regular_icon_over_maskable_one() {
    let manifest = ManifestScenario::given_gmail_manifest().manifest;

    assert_eq!(
        IconChoiceScenario::given_page_icons(manifest.icons).when_choosing_application_icon(),
        Some("https://www.gstatic.com/512.png")
    );
}

#[test]
fn should_prefer_apple_touch_icon_over_small_favicon() {
    let scenario = IconChoiceScenario::given_page_icons(vec![
        SiteIcon::new("https://bank.pl/favicon.ico", Some(32), IconSource::Favicon),
        SiteIcon::new("https://bank.pl/apple-touch-icon.png", None, IconSource::AppleTouchIcon),
    ]);

    assert_eq!(scenario.when_choosing_application_icon(), Some("https://bank.pl/apple-touch-icon.png"));
}

#[test]
fn should_fall_back_to_favicon_when_nothing_better_exists() {
    let scenario = IconChoiceScenario::given_page_icons(vec![SiteIcon::new(
        "https://bank.pl/favicon.ico",
        None,
        IconSource::Favicon,
    )]);

    assert_eq!(scenario.when_choosing_application_icon(), Some("https://bank.pl/favicon.ico"));
}

#[test]
fn should_use_maskable_icon_only_when_it_is_the_only_one() {
    let scenario = IconChoiceScenario::given_page_icons(vec![SiteIcon::maskable("https://a.pl/m.png", Some(512))]);

    assert_eq!(scenario.when_choosing_application_icon(), Some("https://a.pl/m.png"));
}

#[test]
fn should_choose_nothing_for_page_without_icons() {
    assert_eq!(IconChoiceScenario::given_page_icons(vec![]).when_choosing_application_icon(), None);
}
