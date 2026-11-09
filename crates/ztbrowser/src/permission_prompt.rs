//! Asks the user before a site gets access to location, camera, microphone, notifications or clipboard.
//!
//! Requests that only serve fingerprinting or features ZTBrowser cannot provide (device lists before any
//! camera was granted, DRM key systems, XR) are denied without asking. Answers are not remembered: the next
//! request asks again. The Tor window never gives out the location: it would undo what Tor hides.

use adw::prelude::AdwDialogExt;
use adw::prelude::AlertDialogExt;
use adw::prelude::AlertDialogExtManual;
use gtk::prelude::Cast;
use gtk::prelude::ObjectExt;
use url::Url;
use webkit6::ClipboardPermissionRequest;
use webkit6::GeolocationPermissionRequest;
use webkit6::NotificationPermissionRequest;
use webkit6::PermissionRequest;
use webkit6::PointerLockPermissionRequest;
use webkit6::UserMediaPermissionRequest;
use webkit6::WebView;
use webkit6::WebsiteDataAccessPermissionRequest;
use webkit6::prelude::PermissionRequestExt;
use webkit6::prelude::WebViewExt;

enum Answer {
    Ask(String),
    Allow,
    Deny,
}

pub struct PermissionPrompt;

impl PermissionPrompt {
    const ALLOW: &'static str = "allow";
    const DENY: &'static str = "deny";

    /// Handles a `permission-request` signal of a page in the Tor window (`in_tor_window`) or elsewhere; always
    /// claims the request.
    pub fn ask(web_view: &WebView, request: &PermissionRequest, in_tor_window: bool) -> bool {
        let site = web_view
            .uri()
            .and_then(|address| Url::parse(&address).ok())
            .and_then(|url| url.host_str().map(str::to_owned))
            .unwrap_or_else(|| tr!("This page").to_owned());
        let answer = if in_tor_window && request.is::<GeolocationPermissionRequest>() {
            Answer::Deny
        } else {
            Self::answer(&site, request)
        };
        match answer {
            Answer::Allow => request.allow(),
            Answer::Deny => request.deny(),
            Answer::Ask(question) => {
                let dialog = adw::AlertDialog::new(Some(&question), None);
                dialog.add_responses(&[(Self::DENY, tr!("_Deny")), (Self::ALLOW, tr!("_Allow"))]);
                dialog.set_response_appearance(Self::ALLOW, adw::ResponseAppearance::Suggested);
                dialog.set_default_response(Some(Self::DENY));
                dialog.set_close_response(Self::DENY);
                let request = request.clone();
                dialog.connect_response(
                    None,
                    move |_, response| {
                        if response == Self::ALLOW { request.allow() } else { request.deny() }
                    },
                );
                dialog.present(Some(web_view));
            }
        }
        true
    }

    fn answer(site: &str, request: &PermissionRequest) -> Answer {
        if request.is::<GeolocationPermissionRequest>() {
            return Answer::Ask(tr!("Allow {site} to know your location?", site = site));
        }
        if request.is::<NotificationPermissionRequest>() {
            return Answer::Ask(tr!("Allow {site} to show notifications?", site = site));
        }
        if request.is::<ClipboardPermissionRequest>() {
            return Answer::Ask(tr!("Allow {site} to read your clipboard?", site = site));
        }
        if request.is::<PointerLockPermissionRequest>() {
            return Answer::Allow;
        }
        if let Some(media) = request.downcast_ref::<UserMediaPermissionRequest>() {
            return Answer::Ask(match (media.is_for_video_device(), media.is_for_audio_device()) {
                (true, true) => tr!("Allow {site} to use your camera and microphone?", site = site),
                (true, false) => tr!("Allow {site} to use your camera?", site = site),
                (false, true) => tr!("Allow {site} to use your microphone?", site = site),
                (false, false) => tr!("Allow {site} to use your screen?", site = site),
            });
        }
        if let Some(storage) = request.downcast_ref::<WebsiteDataAccessPermissionRequest>() {
            let requesting = storage.requesting_domain().unwrap_or_default();
            let current = storage.current_domain().unwrap_or_default();
            return Answer::Ask(tr!(
                "Allow {requesting} to use its cookies and data while you are on {current}?",
                requesting = requesting,
                current = current
            ));
        }
        Answer::Deny
    }
}
