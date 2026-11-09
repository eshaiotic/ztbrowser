//! The window of an installed web app: one page, no tabs, no address bar, and only its own site.
//!
//! A link the user clicks to another site opens in the browser, so the app never becomes a general browser
//! window holding the app's cookies. Redirects and script navigations stay, since sign-in flows rely on
//! them (Gmail through `accounts.google.com`, banks through 3-D Secure pages). Pop-up windows the app opens
//! (OAuth, payment confirmation) open in a small window of the app and keep their link to the opener.

use std::rc::Rc;

use adw::prelude::AdwApplicationWindowExt;
use adw::prelude::AdwDialogExt;
use adw::prelude::AlertDialogExt;
use adw::prelude::AlertDialogExtManual;
use gio::prelude::ActionMapExt;
use gio::prelude::ActionMapExtManual;
use gtk::prelude::Cast;
use gtk::prelude::GtkWindowExt;
use gtk::prelude::ObjectExt;
use gtk::prelude::ToVariant;
use gtk::prelude::WidgetExt;
use webkit6::NavigationPolicyDecision;
use webkit6::NavigationType;
use webkit6::PolicyDecision;
use webkit6::PolicyDecisionType;
use webkit6::WebView;
use webkit6::prelude::PolicyDecisionExt;
use webkit6::prelude::WebViewExt;
use ztbrowser_core::page_zoom::zoom_level::ZoomLevel;
use ztbrowser_core::web_app::installed_web_app::InstalledWebApp;
use ztbrowser_core::web_app::web_app_navigation_policy::NavigationTrigger;
use ztbrowser_core::web_app::web_app_navigation_policy::WebAppNavigation;
use ztbrowser_core::web_app::web_app_navigation_policy::WebAppNavigationPolicy;

use crate::app_context::AppContext;
use crate::external_launcher::ExternalLauncher;
use crate::page_view::PageView;
use crate::public_suffix_list::PublicSuffixList;
use crate::zoom_controls::ZoomControls;

/// A window action: its name and what it does.
type WebAppCommand = (&'static str, fn(&Rc<WebAppWindow>));

pub struct WebAppWindow {
    context: Rc<AppContext>,
    app: InstalledWebApp,
    window: adw::ApplicationWindow,
    page: Rc<PageView>,
    zoom: ZoomControls,
    toasts: adw::ToastOverlay,
    /// "Protect This Site" in the menu, the app's counterpart of the browser's shield: a web app has its own
    /// session, so it is the only way to switch protections off where they break the app.
    site_protection: gio::SimpleAction,
}

impl WebAppWindow {
    pub fn open(application: &adw::Application, context: &Rc<AppContext>, app: InstalledWebApp) -> Rc<Self> {
        let page = PageView::new(&context.session, context.session.new_web_view());
        let web_app = Rc::new(Self {
            context: Rc::clone(context),
            window: adw::ApplicationWindow::builder()
                .application(application)
                .title(&app.name)
                .default_width(1100)
                .default_height(800)
                .build(),
            app,
            page,
            zoom: ZoomControls::new(),
            toasts: adw::ToastOverlay::new(),
            site_protection: gio::SimpleAction::new_stateful("protect-site", None, &true.to_variant()),
        });
        web_app.build_layout();
        web_app.install_actions();
        // A link out of the app opens in the browser while the app stays where it was, so nothing closes.
        Self::keep_to_site(web_app.page.web_view(), web_app.app.start_url.clone(), || {});
        web_app.open_pop_ups_in_app();
        web_app.page.open(&web_app.app.start_url);
        let keep_alive = Rc::clone(&web_app);
        web_app.window.connect_destroy(move |_| {
            let _ = &keep_alive;
        });
        web_app.window.present();
        web_app
    }

    fn build_layout(self: &Rc<Self>) {
        let title = adw::WindowTitle::new(&self.app.name, &self.app.host());
        let header = adw::HeaderBar::builder().title_widget(&title).build();
        header.pack_start(
            &gtk::Button::builder()
                .icon_name("go-previous-symbolic")
                .action_name("win.back")
                .tooltip_text(tr!("Back"))
                .build(),
        );
        header.pack_start(
            &gtk::Button::builder()
                .icon_name("view-refresh-symbolic")
                .action_name("win.reload")
                .tooltip_text(tr!("Reload"))
                .build(),
        );
        let menu = gtk::MenuButton::builder().icon_name("open-menu-symbolic").tooltip_text(tr!("Main Menu")).build();
        let web_app = Rc::downgrade(self);
        menu.set_create_popup_func(move |button| {
            if let Some(web_app) = web_app.upgrade() {
                web_app.show_site_protection();
                button.set_menu_model(Some(&Self::menu_model(web_app.page.zoom_level())));
            }
        });
        header.pack_end(&menu);
        header.pack_end(self.zoom.badge());
        let web_app = Rc::downgrade(self);
        self.page.web_view().connect_zoom_level_notify(move |_| {
            if let Some(web_app) = web_app.upgrade() {
                web_app.zoom.show(&web_app.window, web_app.page.zoom_level());
            }
        });
        Self::show_current_site(&self.page, &title, &self.app);
        let name = self.app.name.clone();
        self.page.web_view().connect_title_notify(move |view| {
            title.set_title(
                &view.title().filter(|page_title| !page_title.is_empty()).map_or_else(|| name.clone(), String::from),
            );
        });
        let toolbar = adw::ToolbarView::new();
        toolbar.add_top_bar(&header);
        toolbar.set_content(Some(self.page.widget()));
        self.toasts.set_child(Some(&toolbar));
        self.window.set_content(Some(&self.toasts));
    }

    fn menu_model(zoom: ZoomLevel) -> gio::Menu {
        let menu = gio::Menu::new();
        menu.append_item(&ZoomControls::menu_row(zoom));
        let site = gio::Menu::new();
        site.append(Some(tr!("Protect This Site")), Some("win.protect-site"));
        menu.append_section(None, &site);
        let app = gio::Menu::new();
        app.append(Some(tr!("Open Page in Browser")), Some("win.open-in-browser"));
        app.append(Some(tr!("Clear App Data…")), Some("win.fire"));
        app.append(Some(tr!("About ZTBrowser")), Some("app.about"));
        menu.append_section(None, &app);
        menu
    }

    /// The subtitle names the host of the page on screen, flagged when it is not on the app's own site.
    fn show_current_site(page: &Rc<PageView>, title: &adw::WindowTitle, app: &InstalledWebApp) {
        let shown = Rc::downgrade(page);
        let (title, start_url, name) = (title.clone(), app.start_url.clone(), app.name.clone());
        page.web_view().connect_uri_notify(move |view| {
            let (Some(page), Some(address)) = (shown.upgrade(), view.uri()) else { return };
            let Some(host) = page.host() else { return };
            title.set_subtitle(&host);
            if WebAppNavigationPolicy::new(&start_url, &PublicSuffixList).is_app_page(&address) {
                title.remove_css_class("warning");
                title.set_tooltip_text(None);
            } else {
                title.add_css_class("warning");
                title.set_tooltip_text(Some(&tr!("This page is on another site, not part of {name}", name = name)));
            }
        });
    }

    fn install_actions(self: &Rc<Self>) {
        let actions: [WebAppCommand; 7] = [
            ("back", |web_app| web_app.page.web_view().go_back()),
            ("reload", |web_app| web_app.page.web_view().reload()),
            // The page moves to the browser, so the app window closes; it stays if the browser did not start.
            ("open-in-browser", |web_app| {
                if let Some(address) = web_app.page.web_view().uri()
                    && ExternalLauncher::open_in_browser(&address)
                {
                    web_app.window.close();
                }
            }),
            ("fire", Self::confirm_clear),
            ("zoom-in", |web_app| web_app.page.change_zoom(ZoomLevel::zoomed_in)),
            ("zoom-out", |web_app| web_app.page.change_zoom(ZoomLevel::zoomed_out)),
            ("zoom-reset", |web_app| web_app.page.change_zoom(|_| ZoomLevel::ACTUAL_SIZE)),
        ];
        for (name, activate) in actions {
            let web_app = Rc::downgrade(self);
            let action = gio::ActionEntry::builder(name)
                .activate(move |_: &adw::ApplicationWindow, _, _| {
                    if let Some(web_app) = web_app.upgrade() {
                        activate(&web_app);
                    }
                })
                .build();
            self.window.add_action_entries([action]);
        }
        let web_app = Rc::downgrade(self);
        self.site_protection.connect_change_state(move |action, requested| {
            let Some(web_app) = web_app.upgrade() else { return };
            let (Some(protected), Some(host)) = (requested.and_then(glib::Variant::get::<bool>), web_app.page.host())
            else {
                return;
            };
            action.set_state(&protected.to_variant());
            let web_view = web_app.page.web_view().clone();
            web_app.context.session.set_site_protected(&host, protected, move || web_view.reload());
        });
        self.window.add_action(&self.site_protection);
    }

    /// Ticks "Protect This Site" for the page on screen; a page without a host (an error page) has nothing to
    /// switch.
    fn show_site_protection(&self) {
        let host = self.page.host();
        self.site_protection.set_enabled(host.is_some());
        let protected = host.as_deref().is_none_or(|host| self.context.session.is_site_protected(host));
        self.site_protection.set_state(&protected.to_variant());
    }

    /// A clicked link to a site other than `site_url`'s opens in the browser; see [`WebAppNavigationPolicy`].
    /// `when_page_moved` runs once the view's own page went to the browser — not for a link that asked for
    /// a new window, since the page that opened it is still needed (a payment pop-up, for one).
    fn keep_to_site(web_view: &WebView, site_url: String, when_page_moved: impl Fn() + 'static) {
        web_view.connect_decide_policy(move |_, decision, kind| {
            let Some(target) = Self::address_leaving_site(&site_url, decision, kind) else { return false };
            decision.ignore();
            if ExternalLauncher::open_in_browser(&target) && kind == PolicyDecisionType::NavigationAction {
                when_page_moved();
            }
            true
        });
    }

    /// The address to hand to the browser, when this navigation leaves the app's site.
    fn address_leaving_site(site_url: &str, decision: &PolicyDecision, kind: PolicyDecisionType) -> Option<String> {
        if !matches!(kind, PolicyDecisionType::NavigationAction | PolicyDecisionType::NewWindowAction) {
            return None;
        }
        let action = decision.downcast_ref::<NavigationPolicyDecision>()?.navigation_action()?;
        let target = action.request().and_then(|request| request.uri())?;
        let trigger = if action.navigation_type() == NavigationType::LinkClicked {
            NavigationTrigger::LinkClicked
        } else {
            NavigationTrigger::RedirectOrScript
        };
        match WebAppNavigationPolicy::new(site_url, &PublicSuffixList).decide(&target, trigger) {
            WebAppNavigation::StayInWebApp => None,
            WebAppNavigation::OpenInBrowser => Some(target.into()),
        }
    }

    /// Pop-ups share the app's session, so they get the same gate and site indicator as the main window. A
    /// pop-up keeps to the site it was opened for (a sign-in or 3-D Secure page), whose own links must work.
    fn open_pop_ups_in_app(self: &Rc<Self>) {
        let web_app = Rc::downgrade(self);
        self.page.web_view().connect_create(move |opener, action| {
            let web_app = web_app.upgrade()?;
            let pop_up = WebView::builder().related_view(opener).build();
            let page = PageView::new(&web_app.context.session, pop_up.clone());
            let opened_for = action
                .request()
                .and_then(|request| request.uri())
                .map(String::from)
                .filter(|address| address.starts_with("http://") || address.starts_with("https://"))
                .unwrap_or_else(|| web_app.app.start_url.clone());
            let title = adw::WindowTitle::new("", "");
            Self::show_current_site(&page, &title, &web_app.app);
            let header = adw::HeaderBar::builder().title_widget(&title).build();
            let toolbar = adw::ToolbarView::new();
            toolbar.add_top_bar(&header);
            toolbar.set_content(Some(page.widget()));
            let window = adw::Window::builder()
                .transient_for(&web_app.window)
                .default_width(560)
                .default_height(720)
                .content(&toolbar)
                .build();
            // The pop-up's page moved to the browser, so the empty pop-up goes. Closing waits for the policy
            // decision to return: destroying the web view while it emits a signal is unsafe.
            let closing = window.downgrade();
            Self::keep_to_site(&pop_up, opened_for, move || {
                let closing = closing.clone();
                glib::idle_add_local_once(move || {
                    if let Some(window) = closing.upgrade() {
                        window.close();
                    }
                });
            });
            pop_up.connect_title_notify(glib::clone!(
                #[weak]
                window,
                #[weak]
                title,
                move |view| {
                    window.set_title(view.title().as_deref());
                    title.set_title(view.title().as_deref().unwrap_or_default());
                }
            ));
            pop_up.connect_close(glib::clone!(
                #[weak]
                window,
                move |_| window.close()
            ));
            window.connect_destroy(move |_| {
                let _ = &page;
            });
            window.present();
            Some(pop_up.upcast())
        });
    }

    fn confirm_clear(self: &Rc<Self>) {
        let dialog = adw::AlertDialog::new(
            Some(&tr!("Clear Data of {name}?", name = self.app.name)),
            Some(tr!("Signs you out and deletes this app's cookies, site data and cache. Nothing else is affected.")),
        );
        dialog.add_responses(&[("cancel", tr!("_Cancel")), ("clear", tr!("C_lear"))]);
        dialog.set_response_appearance("clear", adw::ResponseAppearance::Destructive);
        dialog.set_close_response("cancel");
        let web_app = Rc::downgrade(self);
        dialog.connect_response(Some("clear"), move |_, _| {
            let Some(web_app) = web_app.upgrade() else { return };
            glib::spawn_future_local(async move {
                let message = match web_app.context.session.burn_website_data().await {
                    Ok(()) => tr!("App data cleared").to_owned(),
                    Err(error) => tr!("Some data could not be deleted: {error}", error = error),
                };
                web_app.page.web_view().load_uri(&web_app.app.start_url);
                web_app.toasts.add_toast(adw::Toast::new(&message));
            });
        });
        dialog.present(Some(&self.window));
    }
}
