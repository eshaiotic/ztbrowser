//! Proposing a web app from the open page: its name, start page and icon candidates.

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::web_app::page_metadata::PageMetadata;
use ztbrowser_core::web_app::site_icon::IconSource;
use ztbrowser_core::web_app::web_app_draft::WebAppDraft;
use ztbrowser_core::web_app::web_app_manifest::WebAppManifest;

struct InstallScenario {
    page_url: String,
    metadata: PageMetadata,
    manifest: Option<WebAppManifest>,
}

impl InstallScenario {
    fn given_page(page_url: &str, collected: &Value) -> Self {
        Self { page_url: page_url.to_owned(), metadata: PageMetadata::from_json(collected), manifest: None }
    }

    fn with_manifest(mut self, manifest_url: &str, document: &Value) -> Self {
        self.manifest = Some(WebAppManifest::from_json(document, manifest_url));
        self
    }

    fn when_drafting(&self) -> WebAppDraft {
        WebAppDraft::new(&self.page_url, &self.metadata, self.manifest.as_ref())
    }
}

#[test]
fn should_read_page_title_manifest_link_and_icon_links() {
    let metadata = PageMetadata::from_json(&json!({
        "title": " mBank ",
        "manifest": "https://online.mbank.pl/manifest.json",
        "icons": [
            {"href": "https://online.mbank.pl/favicon.ico", "rel": "icon", "sizes": ""},
            {"href": "https://online.mbank.pl/touch.png", "rel": "apple-touch-icon", "sizes": "180x180"}
        ]
    }));

    assert_eq!(
        (metadata.title.as_deref(), metadata.manifest_url.as_deref(), metadata.icons[1].source, metadata.icons[1].size),
        (Some("mBank"), Some("https://online.mbank.pl/manifest.json"), IconSource::AppleTouchIcon, Some(180))
    );
}

#[test]
fn should_drop_links_that_are_not_web_addresses() {
    let metadata = PageMetadata::from_json(&json!({
        "manifest": "javascript:alert(1)",
        "icons": [{"href": "data:image/png;base64,AAAA", "rel": "icon"}, {"href": "file:///etc/passwd", "rel": "icon"}]
    }));

    assert_eq!((metadata.manifest_url, metadata.icons.len()), (None, 0));
}

#[test]
fn should_prefer_manifest_name_and_start_page() {
    let draft = InstallScenario::given_page(
        "https://mail.google.com/mail/u/0/#inbox/123",
        &json!({"title": "Inbox (3) - me@gmail.com"}),
    )
    .with_manifest("https://mail.google.com/manifest.json", &json!({"short_name": "Gmail", "start_url": "/mail/"}))
    .when_drafting();

    assert_eq!((draft.name.as_str(), draft.start_url.as_str()), ("Gmail", "https://mail.google.com/mail/"));
}

#[test]
fn should_start_on_current_page_when_manifest_points_to_another_origin() {
    let draft = InstallScenario::given_page("https://app.example.com/home", &json!({"title": "Example"}))
        .with_manifest("https://cdn.example.net/manifest.json", &json!({"start_url": "/"}))
        .when_drafting();

    assert_eq!(draft.start_url, "https://app.example.com/home");
}

#[test]
fn should_name_app_after_host_when_page_has_no_title() {
    let draft =
        InstallScenario::given_page("https://online.mbank.pl/pl/Login", &json!({"title": "  "})).when_drafting();

    assert_eq!(draft.name, "online.mbank.pl");
}

#[test]
fn should_offer_manifest_icons_together_with_page_icons() {
    let draft = InstallScenario::given_page(
        "https://bank.pl/",
        &json!({"icons": [{"href": "https://bank.pl/favicon.ico", "rel": "shortcut icon"}]}),
    )
    .with_manifest("https://bank.pl/manifest.json", &json!({"icons": [{"src": "/192.png", "sizes": "192x192"}]}))
    .when_drafting();

    let sources: Vec<IconSource> = draft.icons.iter().map(|icon| icon.source).collect();
    assert_eq!(sources, [IconSource::Manifest, IconSource::Favicon]);
}
