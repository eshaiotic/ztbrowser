//! Starts the browser: language, engine, policies, then the QML window.

use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;
use std::thread;

use cxx_qt_lib::QFont;
use cxx_qt_lib::QGuiApplication;
use cxx_qt_lib::QQmlApplicationEngine;
use cxx_qt_lib::QString;
use cxx_qt_lib::QUrl;
use serde_json::Value;
use ztbrowser_core::engine::engine_verdict::EngineVerdict;
use ztbrowser_core::engine::minimum_engine_version::MinimumEngineVersion;
use ztbrowser_core::engine::window_kind::WindowKind;
use ztbrowser_core::guard::shared_guard_engine::SharedGuardEngine;
use ztbrowser_core::launch::program_location::ProgramLocation;
use ztbrowser_core::network::site_allow_list::SiteAllowList;
use ztbrowser_core::profile::profile_directories::ProfileLocation;
use ztbrowser_engine_qt::qt_document_fetcher::QtDocumentFetcher;
use ztbrowser_engine_qt::qt_web_engine::QtWebEngine;

use crate::appimage_integration::AppImageIntegration;
use crate::burn_restart::BurnRestart;
use crate::default_browser::DefaultBrowser;
use crate::default_browser::DefaultBrowserChoice;
use crate::desktop_font::DesktopFont;
use crate::download_center::DownloadCenter;
use crate::induckgo_migration::InDuckGoMigration;
use crate::installation_directories::InstallationDirectories;
use crate::interface_catalogue::InterfaceCatalogue;
use crate::interface_language_setting::InterfaceLanguageSetting;
use crate::launch_context::LaunchContext;
use crate::profile_policies::ProfilePolicies;
use crate::profile_policies::ProfileStrictness;
use crate::profile_preferences::ProfilePreferences;
use crate::profile_session::ProfileSession;
use crate::protection_updates::ProtectionUpdates;
use crate::single_instance::Instance;
use crate::single_instance::SingleInstance;
use crate::site_protection::SiteProtection;
use crate::site_zoom_store::SiteZoomStore;
use crate::startup_notice::StartupNotice;

pub struct BrowserApplication;

impl BrowserApplication {
    const WINDOW: &'static str = "qrc:/qt/qml/io/ztbrowser/browser/qml/Main.qml";
    /// The name of the installed desktop entry; Wayland compositors take the window's icon and grouping from it.
    const DESKTOP_ENTRY: &'static str = "io.github.ztbrowser.ZTBrowser";
    const WINDOW_ICON: &'static str = ":/icons/app.svg";

    #[must_use]
    pub fn run() -> ExitCode {
        let migration = InDuckGoMigration::from_environment();
        if let Err(error) = migration.move_folders() {
            eprintln!("ztbrowser: cannot move InDuckGo's data, using it where it is: {error}");
        }
        let directories = migration.directories();
        let arguments: Vec<String> = std::env::args().skip(1).collect();
        let launch = match LaunchContext::resolve(&arguments, &directories) {
            Ok(launch) => launch,
            Err(error) => {
                eprintln!("ztbrowser: {error}");
                return ExitCode::from(2);
            }
        };
        if migration.hand_over_to_running_induckgo(&launch.application_id(), launch.addresses()) {
            return ExitCode::SUCCESS;
        }
        match SingleInstance::acquire(&directories.runtime_directory(), &launch.application_id(), launch.addresses()) {
            Ok(Instance::Forwarded) => return ExitCode::SUCCESS,
            Ok(Instance::Primary(primary)) => LaunchContext::hold(primary),
            Err(error) => eprintln!("ztbrowser: cannot listen for later launches: {error}"),
        }
        if launch.web_app().is_none() {
            Self::integrate_appimage(&directories);
        }
        InterfaceCatalogue::choose(&directories);
        Self::repoint_induckgo_menu_entries(&migration);
        DefaultBrowserChoice::open(DefaultBrowser::for_environment(&directories), &directories.default_browser_file())
            .install();
        QtWebEngine::initialize();

        let mut application = QGuiApplication::new();
        let identity = launch.web_app().map_or_else(|| Self::DESKTOP_ENTRY.to_owned(), |_| launch.application_id());
        if let Some(mut application) = application.as_mut() {
            let name = if launch.web_app().is_some() { identity.as_str() } else { "ZTBrowser" };
            application.as_mut().set_application_name(&QString::from(name));
            Self::use_desktop_font(application);
        }
        // The desktop matches the window to the `.desktop` file of this name, and takes its icon from there.
        QGuiApplication::set_desktop_file_name(&QString::from(identity.as_str()));
        let icon = launch
            .window_icon()
            .map_or_else(|| Self::WINDOW_ICON.to_owned(), |path| path.to_string_lossy().into_owned());
        QtWebEngine::set_window_icon(&icon);
        let profile = directories.profile(&launch.profile_id());
        launch.install();
        Self::install_policies(&directories, &profile);
        Self::check_engine_version();

        let mut engine = QQmlApplicationEngine::new();
        if let Some(mut engine) = engine.as_mut() {
            engine.as_mut().load(&QUrl::from(Self::WINDOW));
        }
        match application.as_mut() {
            Some(application) => ExitCode::from(u8::try_from(application.exec()).unwrap_or(1)),
            None => ExitCode::FAILURE,
        }
    }

    /// An AppImage installs no `.desktop` file, so the desktop could not show its icon; it gets an entry of its own.
    fn integrate_appimage(directories: &InstallationDirectories) {
        let appimage = std::env::var_os(ProgramLocation::APPIMAGE_VARIABLE);
        let appdir = std::env::var_os(ProgramLocation::APPDIR_VARIABLE);
        let Ok(executable) = std::env::current_exe() else { return };
        let Some(appimage) = ProgramLocation::running_appimage(appimage.as_deref(), appdir.as_deref(), &executable)
        else {
            return;
        };
        if let Err(error) = AppImageIntegration::for_environment(directories).install(&appimage) {
            eprintln!("ztbrowser: cannot add the application menu entry: {error}");
        }
    }

    /// The menu entries InDuckGo wrote start this program from now on, described in the language just chosen.
    fn repoint_induckgo_menu_entries(migration: &InDuckGoMigration) {
        let appimage = std::env::var_os(ProgramLocation::APPIMAGE_VARIABLE);
        let Ok(executable) = std::env::current_exe() else { return };
        let program = ProgramLocation::choose(appimage.as_deref(), executable);
        if let Err(error) = migration.repoint_menu_entries(&program, InterfaceCatalogue::current()) {
            eprintln!("ztbrowser: cannot point InDuckGo's menu entries at ZTBrowser: {error}");
        }
    }

    /// Draws the interface in the font the desktop chose, as the GTK edition did.
    fn use_desktop_font(application: std::pin::Pin<&mut QGuiApplication>) {
        let mut font = QFont::default();
        let wanted = DesktopFont::from_desktop()
            .description(&font.family().map(|family| family.to_string()).unwrap_or_default());
        if font.from_string(&QString::from(wanted.as_str())) {
            application.set_application_font(&font);
        }
    }

    fn install_policies(directories: &InstallationDirectories, profile: &ProfileLocation) {
        let cache = directories.protection_cache();
        let updates = Arc::new(ProtectionUpdates::new(cache.clone(), Arc::new(QtDocumentFetcher)));
        let guard = Arc::new(SharedGuardEngine::default());
        let session = Self::open_session(directories, profile, &guard);
        let allow_list = Self::read_json(&profile.data.join("site-allow-list.json"))
            .map(|document| SiteAllowList::from_json(&document))
            .unwrap_or_default();
        // The rules are put together on a worker thread while this thread starts Chromium's profile: neither
        // needs the other, and they are the two longest parts of the start-up. The first start downloads the
        // data on that thread too, and Guard's engine is built from the lists only after that download; the
        // window still opens only after both finish, so no page loads unprotected.
        let assembly = {
            let updates = Arc::clone(&updates);
            let unprotected_sites = session.protection.shared_list();
            let session_guard = Arc::clone(&session.guard);
            let engine = Arc::clone(&guard);
            thread::spawn(move || {
                let starting = updates.starting_protection(&engine);
                let policies = ProfilePolicies::assemble(
                    starting.documents.as_ref(),
                    unprotected_sites,
                    session_guard,
                    allow_list,
                    ProfileStrictness::Standard,
                );
                (Arc::new(policies), starting.is_first_start)
            })
        };
        QtWebEngine::open_profile(&profile.data.join("engine"), &profile.cache.join("engine"));
        let (policies, is_first_start) = assembly.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic));
        session.install();
        QtWebEngine::install_policies(Arc::clone(&policies.request), Arc::clone(&policies.cookies));
        updates.keep_current(policies, guard, !is_first_start);
    }

    /// Finishes a burn the Fire button asked for before anything opens the profile's files, then reads the
    /// profile's stored choices.
    fn open_session(
        directories: &InstallationDirectories,
        profile: &ProfileLocation,
        guard: &Arc<SharedGuardEngine>,
    ) -> ProfileSession {
        let burned = BurnRestart::new(profile.clone()).complete_pending();
        let downloads = DownloadCenter::open(&profile.data, directories.system_downloads_folder());
        if let Err(error) = burned {
            StartupNotice::set(
                InterfaceCatalogue::current()
                    .text_with("Some data could not be deleted: {error}", &[("error", &error.to_string())]),
            );
        }
        ProfileSession::new(
            profile,
            SiteProtection::open(&profile.data),
            downloads,
            SiteZoomStore::open(&profile.data),
            ProfilePreferences::open(&profile.data),
            InterfaceLanguageSetting::open(&directories.interface_language_file()),
            Arc::clone(guard),
        )
    }

    fn check_engine_version() {
        let Some(installed) = QtWebEngine::chromium_version() else { return };
        if MinimumEngineVersion::current().verdict(&installed, WindowKind::Normal) == EngineVerdict::Warn {
            StartupNotice::set(format!(
                "The web engine ({installed}) is older than ZTBrowser expects and may lack security fixes."
            ));
        }
    }

    fn read_json(path: &Path) -> Option<Value> {
        serde_json::from_slice(&fs::read(path).ok()?).ok()
    }
}
