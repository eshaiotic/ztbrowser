//! One web page on screen, with the behaviour every ZTBrowser page shares, in a tab or in a web app window.
//!
//! * Tracking parameters: the `links` rule list stops a top-frame navigation that carries one before its
//!   request leaves the machine; WebKit reports it as a failed load, and the page opens the cleaned address.
//! * HTTPS upgrades: when an upgraded load fails, a banner explains that the site has no secure version and
//!   offers to load it insecurely (never silently, so an attacker cannot force a downgrade).
//! * Links to other applications (`mailto:`, `tel:`, ...) open outside, and only after a real click; never from
//!   the Tor window, since the other application would connect outside Tor and could reveal the address.
//! * Permission requests are asked, never granted silently.
//! * A page that cannot be shown (unknown host, refused connection, untrusted certificate, crash, an error
//!   status with an empty body) is replaced by an ZTBrowser error page that explains it.
//! * Zoom: each page is drawn at the size the user gave its site; Ctrl and the scroll wheel change it.

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::rc::Weak;

use adw::prelude::WidgetExt;
use gtk::prelude::BoxExt;
use gtk::prelude::Cast;
use gtk::prelude::EventControllerExt;
use gtk::prelude::IsA;
use gtk::prelude::StaticType;
use url::Url;
use webkit6::LoadEvent;
use webkit6::NavigationPolicyDecision;
use webkit6::PolicyDecision;
use webkit6::PolicyDecisionType;
use webkit6::PolicyError;
use webkit6::ResponsePolicyDecision;
use webkit6::WebProcessTerminationReason;
use webkit6::WebView;
use webkit6::prelude::PolicyDecisionExt;
use webkit6::prelude::WebViewExt;
use ztbrowser_core::navigation::load_failure::FailureCause;
use ztbrowser_core::navigation::load_failure::LoadFailure;
use ztbrowser_core::page_zoom::zoom_level::ZoomLevel;

use crate::error_page_view::ErrorPageView;
use crate::external_launcher::ExternalLauncher;
use crate::permission_prompt::PermissionPrompt;
use crate::profile_session::ProfileSession;
use crate::site_zoom::SiteZoom;

pub struct PageView {
    session: Rc<ProfileSession>,
    web_view: WebView,
    root: gtk::Box,
    insecure_banner: adw::Banner,
    /// Recent plain-HTTP navigation targets, to recognise the failure of a load the HTTPS rules upgraded.
    recent_plain_http: RefCell<VecDeque<String>>,
    /// `(https, http)` of the failed upgrade the banner offers to load insecurely.
    insecure_fallback: RefCell<Option<(String, String)>>,
    /// The last address this page cleaned, so a rule/cleaner mismatch can never loop.
    last_cleaned: RefCell<Option<String>>,
    error_page: RefCell<ErrorPageState>,
    /// `(address, status)` of the empty error answer the page stopped, to explain when its load fails.
    empty_error_answer: RefCell<Option<(String, u16)>>,
}

/// Whether the view shows one of ZTBrowser's error pages instead of a page of the site.
#[derive(PartialEq, Eq)]
enum ErrorPageState {
    Absent,
    /// Requested for this address; only a load of that address can be the error page's own load.
    Loading(String),
    Shown,
}

impl PageView {
    const RECENT_NAVIGATIONS: usize = 16;
    /// Schemes WebKit displays itself; every other scheme belongs to another application.
    const WEB_SCHEMES: [&'static str; 7] = ["http", "https", "about", "data", "blob", "file", "webkit-pdfjs-viewer"];
    /// Touchpad scrolling is measured in pixels; this many make one zoom step, like one notch of a wheel.
    const PIXELS_PER_ZOOM_STEP: f64 = 40.0;

    pub fn new(session: &Rc<ProfileSession>, web_view: WebView) -> Rc<Self> {
        let insecure_banner = adw::Banner::builder().button_label(tr!("Load Insecure Page")).revealed(false).build();
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        web_view.set_vexpand(true);
        root.append(&insecure_banner);
        root.append(&web_view);
        let page = Rc::new(Self {
            session: Rc::clone(session),
            web_view,
            root,
            insecure_banner,
            recent_plain_http: RefCell::new(VecDeque::new()),
            insecure_fallback: RefCell::new(None),
            last_cleaned: RefCell::new(None),
            error_page: RefCell::new(ErrorPageState::Absent),
            empty_error_answer: RefCell::new(None),
        });
        page.connect_signals();
        page.zoom_on_control_scroll();
        session.zoom().follow(&page.web_view);
        page
    }

    pub fn web_view(&self) -> &WebView {
        &self.web_view
    }

    pub fn widget(&self) -> &gtk::Widget {
        self.root.upcast_ref()
    }

    /// Loads `address` as soon as tracker blocking is active.
    pub fn open(&self, address: &str) {
        let web_view = self.web_view.clone();
        let address = address.to_owned();
        self.session.when_protected(move || web_view.load_uri(&address));
    }

    /// Whether the view shows an ZTBrowser error page rather than the site. The page runs in the site's origin,
    /// so the site's other windows can still rewrite it: never treat it as more trustworthy than the site.
    pub fn shows_error_page(&self) -> bool {
        *self.error_page.borrow() == ErrorPageState::Shown
    }

    /// The size the page is drawn at.
    pub fn zoom_level(&self) -> ZoomLevel {
        SiteZoom::current(&self.web_view)
    }

    /// Zooms the page, and with it every open page of its site.
    pub fn change_zoom(&self, change: fn(ZoomLevel) -> ZoomLevel) {
        self.session.zoom().change(&self.web_view, change);
    }

    /// Host of the current page, if it is a web page.
    pub fn host(&self) -> Option<String> {
        let address = self.web_view.uri()?;
        let url = Url::parse(&address).ok().filter(|url| matches!(url.scheme(), "http" | "https"))?;
        url.host_str().map(str::to_owned)
    }

    fn connect_signals(self: &Rc<Self>) {
        let page = Rc::downgrade(self);
        self.web_view.connect_decide_policy(move |_, decision, kind| {
            page.upgrade().is_some_and(|page| page.decide_policy(decision, kind))
        });
        let page = Rc::downgrade(self);
        self.web_view.connect_load_failed(move |_, _, failing_address, error| {
            page.upgrade().is_some_and(|page| page.recover_from_failed_load(failing_address, error))
        });
        let page = Rc::downgrade(self);
        self.web_view.connect_load_failed_with_tls_errors(move |_, failing_address, certificate, errors| {
            page.upgrade().is_some_and(|page| page.refuse_untrusted_certificate(failing_address, certificate, errors))
        });
        let page = Rc::downgrade(self);
        self.web_view.connect_load_changed(move |_, event| {
            let Some(page) = page.upgrade() else { return };
            match event {
                LoadEvent::Started => page.forget_failed_load(),
                LoadEvent::Committed => page.note_committed_page(),
                _ => {}
            }
        });
        let page = Rc::downgrade(self);
        self.web_view.connect_web_process_terminated(move |web_view, reason| {
            let cause = match reason {
                WebProcessTerminationReason::Crashed => FailureCause::PageCrashed,
                WebProcessTerminationReason::ExceededMemoryLimit => FailureCause::OutOfMemory,
                _ => return,
            };
            if let Some(page) = page.upgrade()
                && let Some(address) = web_view.uri()
            {
                page.show_error_page(&LoadFailure::new(&address, cause, ""));
            }
        });
        let page = Rc::downgrade(self);
        self.insecure_banner.connect_button_clicked(move |_| Self::load_insecurely(&page));
        let in_tor_window = self.session.is_tor_window();
        self.web_view
            .connect_permission_request(move |view, request| PermissionPrompt::ask(view, request, in_tor_window));
    }

    /// Ctrl with the scroll wheel zooms (up enlarges), as in every browser. The event is caught before it
    /// reaches the page, so a page cannot zoom itself this way and does not scroll meanwhile.
    fn zoom_on_control_scroll(self: &Rc<Self>) {
        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        scroll.set_propagation_phase(gtk::PropagationPhase::Capture);
        let pending = Cell::new(0.0);
        let page = Rc::downgrade(self);
        scroll.connect_scroll(move |scroll, _, delta| {
            let Some(page) = page.upgrade() else { return glib::Propagation::Proceed };
            if delta == 0.0 || !scroll.current_event_state().contains(gdk::ModifierType::CONTROL_MASK) {
                pending.set(0.0);
                return glib::Propagation::Proceed;
            }
            let steps = if scroll.unit() == gdk::ScrollUnit::Surface { Self::PIXELS_PER_ZOOM_STEP } else { 1.0 };
            let scrolled = pending.get() + delta / steps;
            if scrolled.abs() < 1.0 {
                pending.set(scrolled);
            } else {
                pending.set(0.0);
                page.change_zoom(if scrolled < 0.0 { ZoomLevel::zoomed_in } else { ZoomLevel::zoomed_out });
            }
            glib::Propagation::Stop
        });
        self.root.add_controller(scroll);
    }

    /// Remembers plain-HTTP targets, sends links to other applications outside and stops empty error answers;
    /// WebKit decides the rest.
    fn decide_policy(&self, decision: &PolicyDecision, kind: PolicyDecisionType) -> bool {
        match kind {
            PolicyDecisionType::NavigationAction | PolicyDecisionType::NewWindowAction => {
                self.decide_navigation(decision)
            }
            PolicyDecisionType::Response => self.stop_empty_error_answer(decision),
            _ => false,
        }
    }

    fn decide_navigation(&self, decision: &PolicyDecision) -> bool {
        let Some(navigation) = decision.downcast_ref::<NavigationPolicyDecision>() else { return false };
        let Some(action) = navigation.navigation_action() else { return false };
        let Some(address) = action.request().and_then(|request| request.uri()) else { return false };
        let scheme = address.split_once(':').map(|(scheme, _)| scheme.to_ascii_lowercase()).unwrap_or_default();
        if scheme == "http" {
            let mut recent = self.recent_plain_http.borrow_mut();
            recent.push_back(address.to_string());
            if recent.len() > Self::RECENT_NAVIGATIONS {
                recent.pop_front();
            }
        }
        if Self::WEB_SCHEMES.contains(&scheme.as_str()) {
            return false;
        }
        decision.ignore();
        if !action.is_user_gesture() {
            return true;
        }
        if self.session.is_tor_window() {
            self.announce(tr!("Other applications do not open from the Tor window: they would connect outside Tor."));
        } else {
            ExternalLauncher::open_outside(&address, self.window().as_ref());
        }
        true
    }

    /// A toast in the window showing this page.
    fn announce(&self, text: &str) {
        if let Some(overlay) = self
            .web_view
            .ancestor(adw::ToastOverlay::static_type())
            .and_then(|widget| widget.downcast::<adw::ToastOverlay>().ok())
        {
            overlay.add_toast(adw::Toast::new(text));
        }
    }

    /// A server that answers an error status with an empty body (`Content-Length: 0`) would leave a blank page.
    /// Stopping the response makes the load fail, so its error page replaces it in the history like the page of
    /// any failed load (shown later, it would be a history entry Back returns to and re-explains forever).
    fn stop_empty_error_answer(&self, decision: &PolicyDecision) -> bool {
        let Some(decision) = decision.downcast_ref::<ResponsePolicyDecision>() else { return false };
        let Some(response) = decision.response() else { return false };
        let status = u16::try_from(response.status_code()).ok().filter(|status| *status >= 400);
        let length = response.http_headers().and_then(|headers| headers.one("Content-Length"));
        let is_empty = length.is_some_and(|length| length.trim() == "0");
        let (Some(status), true, true, Some(address)) =
            (status, is_empty, decision.is_main_frame_main_resource(), response.uri())
        else {
            return false;
        };
        self.empty_error_answer.replace(Some((address.to_string(), status)));
        decision.ignore();
        true
    }

    /// Opens the cleaned address of a navigation stopped for its tracking parameters; explains any other failure.
    fn recover_from_failed_load(&self, failing_address: &str, error: &glib::Error) -> bool {
        if let Some((address, status)) = self.empty_error_answer.take()
            && address == failing_address
        {
            self.show_error_page(&LoadFailure::new(failing_address, FailureCause::HttpStatus(status), ""));
            return true;
        }
        let Some(cause) = ErrorPageView::cause_of(error) else { return false };
        if error.kind::<PolicyError>().is_some()
            && self.last_cleaned.borrow().as_deref() != Some(failing_address)
            && let Some(cleaned) = self.session.cleaned_address(failing_address)
        {
            self.last_cleaned.replace(Some(cleaned.clone()));
            self.web_view.load_uri(&cleaned);
            return true;
        }
        self.offer_insecure_page(failing_address);
        self.show_error_page(&LoadFailure::new(failing_address, cause, error.message()));
        true
    }

    /// WebKit never loads a page whose certificate fails verification; this explains why, and there is no way
    /// past it.
    fn refuse_untrusted_certificate(
        &self,
        failing_address: &str,
        certificate: &gio::TlsCertificate,
        errors: gio::TlsCertificateFlags,
    ) -> bool {
        self.offer_insecure_page(failing_address);
        let detail = ErrorPageView::certificate_detail(certificate);
        self.show_error_page(&LoadFailure::new(failing_address, ErrorPageView::certificate_cause(errors), &detail));
        true
    }

    fn show_error_page(&self, failure: &LoadFailure) {
        self.error_page.replace(ErrorPageState::Loading(failure.address.clone()));
        ErrorPageView::show(&self.web_view, failure, self.session.search_engine());
    }

    /// Whether the load in progress is the requested error page: it loads under the failed address.
    fn is_loading_error_page(&self) -> bool {
        let address = self.web_view.uri();
        matches!(&*self.error_page.borrow(), ErrorPageState::Loading(requested) if address.as_deref() == Some(requested))
    }

    /// A new load hides the banner about the last failure, unless it is that failure's error page.
    fn forget_failed_load(&self) {
        if !self.is_loading_error_page() {
            self.error_page.replace(ErrorPageState::Absent);
            self.insecure_banner.set_revealed(false);
        }
    }

    fn note_committed_page(&self) {
        self.session.zoom().apply(&self.web_view);
        let state = if self.is_loading_error_page() { ErrorPageState::Shown } else { ErrorPageState::Absent };
        self.error_page.replace(state);
    }

    /// When the failed load was an HTTPS upgrade of a plain-HTTP address, offers to load that address insecurely.
    fn offer_insecure_page(&self, failing_address: &str) {
        let upgraded_from = self
            .recent_plain_http
            .borrow()
            .iter()
            .find(|plain| self.session.was_upgraded(plain, failing_address))
            .cloned();
        if let Some(plain_http) = upgraded_from {
            let host =
                Url::parse(&plain_http).ok().and_then(|url| url.host_str().map(str::to_owned)).unwrap_or_default();
            self.insecure_banner.set_title(&tr!("{host} does not offer a secure connection", host = host));
            self.insecure_banner.set_revealed(true);
            self.insecure_fallback.replace(Some((failing_address.to_owned(), plain_http)));
        }
    }

    fn load_insecurely(page: &Weak<Self>) {
        let Some(page) = page.upgrade() else { return };
        let Some((https, http)) = page.insecure_fallback.take() else { return };
        page.insecure_banner.set_revealed(false);
        let web_view = page.web_view.clone();
        page.session.stop_upgrading(&https, move || web_view.load_uri(&http));
    }

    fn window(&self) -> Option<gtk::Window> {
        self.web_view.root().and_then(|root| root.downcast::<gtk::Window>().ok())
    }

    /// Whether `widget` is this page's content (tab pages hold [`PageView::widget`]).
    pub fn shows(&self, widget: &impl IsA<gtk::Widget>) -> bool {
        self.root.upcast_ref::<gtk::Widget>() == widget.upcast_ref::<gtk::Widget>()
    }
}
