//! The GTK application of one ZTBrowser process.
//!
//! The command line decides the application id ([`LaunchRequest`]): the default profile, another profile,
//! one web app, or the Tor window, whose process runs its own Tor. GTK makes the first process with an id the primary one; later launches with the same id
//! pass their command line to it over D-Bus and exit. So `ztbrowser https://example.com` opens a tab in the
//! running browser, and clicking a web app's icon twice shows its existing window.

use std::cell::OnceCell;
use std::rc::Rc;

use adw::prelude::AdwDialogExt;
use gio::prelude::ActionMapExtManual;
use gio::prelude::ApplicationCommandLineExt;
use gio::prelude::ApplicationExt;
use gio::prelude::ApplicationExtManual;
use gtk::prelude::GtkApplicationExt;
use gtk::prelude::GtkWindowExt;
use gtk::prelude::ToVariant;
use ztbrowser_core::launch::launch_request::LaunchRequest;
use ztbrowser_core::navigation::search_engine::SearchEngine;
use ztbrowser_core::profile::profile_id::ProfileId;

use crate::app_context::AppContext;
use crate::app_icons::AppIcons;
use crate::browser_window::BrowserWindow;
use crate::browser_windows::BrowserWindows;
use crate::catalog_files::CatalogFiles;
use crate::external_launcher::ExternalLauncher;
use crate::http_client::HttpClient;
use crate::installation_paths::InstallationPaths;
use crate::interface_language_setting::InterfaceLanguageSetting;
use crate::profile_session::ProfileSession;
use crate::stored_file::StoredFile;
use crate::toolbar_preferences::ToolbarPreferences;
use crate::tor_connection::TorConnection;
use crate::tor_connection::TorState;
use crate::web_app_window::WebAppWindow;

pub struct ZTBrowserApplication {
    /// `None` for the Tor window's process.
    profile: Option<ProfileId>,
    context: OnceCell<Rc<AppContext>>,
}

impl ZTBrowserApplication {
    const USAGE_ERROR: u8 = 2;
    const NOT_FOUND: u8 = 1;
    const ACCELERATORS: [(&'static str, &'static [&'static str]); 15] = [
        ("win.new-tab", &["<Control>t"]),
        ("win.close-tab", &["<Control>w"]),
        ("app.new-window", &["<Control>n"]),
        ("app.new-tor-window", &["<Alt><Shift>n"]),
        ("win.preferences", &["<Control>comma"]),
        ("win.focus-address", &["<Control>l", "F6"]),
        ("win.reload", &["<Control>r", "F5"]),
        ("win.back", &["<Alt>Left"]),
        ("win.forward", &["<Alt>Right"]),
        ("win.find", &["<Control>f"]),
        ("win.zoom-in", &["<Control>plus", "<Control>equal", "<Control>KP_Add"]),
        ("win.zoom-out", &["<Control>minus", "<Control>KP_Subtract"]),
        ("win.zoom-reset", &["<Control>0", "<Control>KP_0"]),
        ("win.fire", &["<Control><Shift>Delete"]),
        ("app.quit", &["<Control>q"]),
    ];

    /// Runs the process; the exit code is 2 for a malformed command line.
    pub fn run() -> glib::ExitCode {
        let arguments: Vec<String> =
            std::env::args_os().map(|argument| argument.to_string_lossy().into_owned()).collect();
        let request = match LaunchRequest::parse(arguments.get(1..).unwrap_or_default()) {
            Ok(request) => request,
            Err(error) => {
                eprintln!("ztbrowser: {error}");
                return glib::ExitCode::from(Self::USAGE_ERROR);
            }
        };
        InterfaceLanguageSetting::load(StoredFile::new(InstallationPaths::from_environment().interface_language()))
            .activate();
        let application = adw::Application::builder()
            .application_id(request.application_id().as_str())
            .flags(gio::ApplicationFlags::HANDLES_COMMAND_LINE)
            .build();
        let process = Rc::new(Self { profile: request.profile_id(), context: OnceCell::new() });
        let starting = Rc::clone(&process);
        application.connect_startup(move |application| starting.start(application));
        let stopping = Rc::clone(&process);
        application.connect_shutdown(move |_| {
            if let Some(context) = stopping.context.get() {
                context.session.close();
            }
        });
        application.connect_command_line(move |application, command_line| process.handle(application, command_line));
        application.run_with_args(&arguments)
    }

    fn start(self: &Rc<Self>, application: &adw::Application) {
        Self::switch_off_animations();
        let paths = InstallationPaths::from_environment();
        let icons = AppIcons::new(&paths);
        icons.install();
        let (http, session, tor, toolbar_file) = if let Some(profile) = &self.profile {
            let http = Rc::new(HttpClient::new());
            let session = ProfileSession::start(profile.clone(), &paths, Rc::clone(&http));
            let toolbar_file = StoredFile::new(paths.profile(profile).data.join(ToolbarPreferences::FILE_NAME));
            (http, session, None, toolbar_file)
        } else {
            let http = Rc::new(HttpClient::awaiting_tor());
            let session = ProfileSession::start_tor(&paths, Rc::clone(&http));
            (http, Rc::clone(&session), Some(Self::start_tor(&paths, &session)), StoredFile::discarded())
        };
        let catalogs = CatalogFiles::new(&paths);
        let interface_language = InterfaceLanguageSetting::load(StoredFile::new(paths.interface_language()));
        let _ = self.context.set(Rc::new(AppContext {
            paths,
            http,
            catalogs,
            icons,
            session,
            toolbar: ToolbarPreferences::load(toolbar_file),
            interface_language,
            browser_windows: BrowserWindows::default(),
            tor,
        }));
        for (action, accelerators) in Self::ACCELERATORS {
            application.set_accels_for_action(action, accelerators);
        }
        self.install_actions(application);
    }

    /// Starts the Tor window's Tor; the session sends its traffic through Tor once it is connected.
    fn start_tor(paths: &InstallationPaths, session: &Rc<ProfileSession>) -> Rc<TorConnection> {
        let tor = TorConnection::new(paths);
        let session = Rc::downgrade(session);
        tor.observe(move |state| {
            let Some(session) = session.upgrade() else { return glib::ControlFlow::Break };
            match state {
                TorState::Connected { proxy_uri } => session.route_through_tor(proxy_uri),
                TorState::Starting | TorState::Connecting(_) | TorState::Failed(_) => session.await_tor(),
            }
            glib::ControlFlow::Continue
        });
        tor.start();
        tor
    }

    /// Tabs, dialogs, banners, toasts and the find bar appear at once instead of sliding and fading: the
    /// interface reacts immediately, and on a busy machine animations are the first thing to stutter. Set by
    /// the application, this outranks the desktop's own animation setting for ZTBrowser's processes.
    fn switch_off_animations() {
        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_enable_animations(false);
        }
    }

    fn handle(
        self: &Rc<Self>,
        application: &adw::Application,
        command_line: &gio::ApplicationCommandLine,
    ) -> glib::ExitCode {
        let arguments: Vec<String> =
            command_line.arguments().iter().skip(1).map(|argument| argument.to_string_lossy().into_owned()).collect();
        let request = match LaunchRequest::parse(&arguments) {
            Ok(request) => request,
            Err(error) => {
                command_line.printerr_literal(&format!("ztbrowser: {error}\n"));
                return glib::ExitCode::from(Self::USAGE_ERROR);
            }
        };
        let Some(context) = self.context.get() else { return glib::ExitCode::FAILURE };
        if let Some(web_app_id) = request.web_app_id() {
            if let Some(window) = application.active_window() {
                window.present();
                return glib::ExitCode::SUCCESS;
            }
            let Some(installed) = context.catalogs.web_apps().web_app(web_app_id).cloned() else {
                command_line.printerr_literal(&format!("ztbrowser: web app {web_app_id} is not installed\n"));
                return glib::ExitCode::from(Self::NOT_FOUND);
            };
            WebAppWindow::open(application, context, installed);
            return glib::ExitCode::SUCCESS;
        }
        if let Some(profile) = &self.profile
            && !profile.is_default()
            && context.catalogs.profiles().profile(profile).is_none()
        {
            command_line.printerr_literal(&format!("ztbrowser: there is no profile {profile}\n"));
            return glib::ExitCode::from(Self::NOT_FOUND);
        }
        match context.browser_windows.active(application).filter(|_| !request.addresses().is_empty()) {
            Some(browser) => {
                browser.open_typed(request.addresses());
                browser.window().present();
            }
            None => Self::open_browser_window(application, context, request.addresses()),
        }
        glib::ExitCode::SUCCESS
    }

    fn install_actions(self: &Rc<Self>, application: &adw::Application) {
        let process = Rc::downgrade(self);
        let new_window = gio::ActionEntry::builder("new-window")
            .activate(move |application: &adw::Application, _, _| {
                if let Some(process) = process.upgrade()
                    && let Some(context) = process.context.get()
                {
                    Self::open_browser_window(application, context, &[]);
                }
            })
            .build();
        let process = Rc::downgrade(self);
        let open_profile = gio::ActionEntry::builder("open-profile")
            .parameter_type(Some(glib::VariantTy::STRING))
            .activate(move |application: &adw::Application, _, parameter| {
                let Some(process) = process.upgrade() else { return };
                let Some(profile) = parameter.and_then(|value| value.str()).and_then(|id| ProfileId::parse(id).ok())
                else {
                    return;
                };
                match process.context.get() {
                    Some(context) if process.profile.as_ref() == Some(&profile) => {
                        Self::open_browser_window(application, context, &[]);
                    }
                    _ => ExternalLauncher::start_profile(&profile),
                }
            })
            .build();
        let process = Rc::downgrade(self);
        let new_tor_window = gio::ActionEntry::builder("new-tor-window")
            .activate(move |application: &adw::Application, _, _| {
                let Some(process) = process.upgrade() else { return };
                match process.context.get() {
                    Some(context) if context.tor.is_some() => Self::open_browser_window(application, context, &[]),
                    _ => ExternalLauncher::start_tor_window(),
                }
            })
            .build();
        let process = Rc::downgrade(self);
        let current = self.context.get().map(|context| context.session.search_engine()).unwrap_or_default();
        let search_engine = gio::ActionEntry::builder("search-engine")
            .parameter_type(Some(glib::VariantTy::STRING))
            .state(current.id().to_variant())
            .activate(move |_: &adw::Application, action, parameter| {
                let (Some(process), Some(parameter)) = (process.upgrade(), parameter) else { return };
                let Some(engine) = parameter.str().and_then(SearchEngine::from_id) else { return };
                if let Some(context) = process.context.get() {
                    context.session.choose_search_engine(engine);
                    context.browser_windows.open().iter().for_each(|browser| browser.show_search_engine());
                }
                action.set_state(parameter);
            })
            .build();
        let about = gio::ActionEntry::builder("about")
            .activate(|application: &adw::Application, _, _| Self::present_about(application))
            .build();
        let quit = gio::ActionEntry::builder("quit")
            .activate(|application: &adw::Application, _, _| application.quit())
            .build();
        application.add_action_entries([new_window, new_tor_window, open_profile, search_engine, about, quit]);
    }

    fn open_browser_window(application: &adw::Application, context: &Rc<AppContext>, addresses: &[String]) {
        context.browser_windows.register(&BrowserWindow::open(application, context, addresses));
    }

    fn present_about(application: &adw::Application) {
        let about = adw::AboutDialog::builder()
            .application_name("ZTBrowser")
            .application_icon(AppIcons::APPLICATION)
            .version(env!("CARGO_PKG_VERSION"))
            .license_type(gtk::License::Gpl30)
            .comments(tr!("A privacy-first browser for Linux built on DuckDuckGo's open-source protections. \
                 Not affiliated with or endorsed by DuckDuckGo."))
            .build();
        about.add_legal_section(
            tr!("DuckDuckGo protections"),
            Some("© DuckDuckGo, Inc."),
            gtk::License::Apache20,
            Some(tr!("Content Scope Scripts, the privacy configuration and the TrackerRadarKit rule algorithm.")),
        );
        about.add_legal_section(
            tr!("Tracker Data Set"),
            Some("© DuckDuckGo, Inc."),
            gtk::License::Custom,
            Some(tr!("Downloaded at run time from DuckDuckGo; licensed CC BY-NC-SA 4.0.")),
        );
        about.add_legal_section(
            "adblock-rust",
            Some("© Brave Software, Inc."),
            gtk::License::Mpl20,
            Some(tr!(
                "Turns ZTBrowser Guard's filter lists into WebKit rules. Source: https://github.com/brave/adblock-rust"
            )),
        );
        about.add_legal_section(
            tr!("ZTBrowser Guard filter lists"),
            Some(tr!("© their maintainers")),
            gtk::License::Custom,
            Some(tr!("Downloaded at run time from their publishers: EasyList, EasyPrivacy and EasyList Cookie List \
                 (GPL-3.0 or CC BY-SA 3.0, CC BY 3.0), uBlock Origin's lists (GPL-3.0), Peter Lowe's list \
                 (pgl.yoyo.org terms), Polish Filters (CC BY-NC-SA 4.0).")),
        );
        about.add_legal_section(
            "Tor",
            Some("© The Tor Project, Inc."),
            gtk::License::Bsd3,
            Some(tr!(
                "The Tor window runs tor (bundled or your distribution's) and Tor Browser's built-in bridges. \
                 Not affiliated with or endorsed by the Tor Project. Source: https://gitlab.torproject.org/tpo/core/tor"
            )),
        );
        about.present(application.active_window().as_ref());
    }
}
