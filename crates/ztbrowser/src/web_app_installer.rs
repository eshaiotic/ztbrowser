//! "Install Site as App": turns the open page into a separate application with its own profile.
//!
//! The page's own metadata is read by a script in an isolated JavaScript world (the page cannot tamper with
//! the collector), the manifest and icons are downloaded without cookies, and only PNG icons of sane size
//! are decoded. The result is a catalog entry, an icon file and a `.desktop` file in
//! `~/.local/share/applications`, which puts the app into the desktop's application menu.

use std::error::Error;
use std::rc::Rc;

use adw::prelude::AdwDialogExt;
use adw::prelude::AlertDialogExt;
use adw::prelude::AlertDialogExtManual;
use gdk::prelude::TextureExt;
use gtk::prelude::EditableExt;
use url::Url;
use webkit6::WebView;
use webkit6::prelude::WebViewExt;
use ztbrowser_core::web_app::desktop_entry::DesktopEntry;
use ztbrowser_core::web_app::installed_web_app::InstalledWebApp;
use ztbrowser_core::web_app::page_metadata::PageMetadata;
use ztbrowser_core::web_app::png_header::PngHeader;
use ztbrowser_core::web_app::site_icon::SiteIcon;
use ztbrowser_core::web_app::site_icon_choice::SiteIconChoice;
use ztbrowser_core::web_app::web_app_draft::WebAppDraft;
use ztbrowser_core::web_app::web_app_manifest::WebAppManifest;

use crate::app_context::AppContext;
use crate::external_launcher::ExternalLauncher;
use crate::http_client::Fetched;
use crate::interface_messages::InterfaceMessages;
use crate::stored_file::StoredFile;

pub struct WebAppInstaller;

impl WebAppInstaller {
    const ISOLATED_WORLD: &'static str = "ztbrowser-installer";
    const MANIFEST_MAX_BYTES: usize = 512 * 1024;
    const ICON_MAX_BYTES: usize = 2 * 1024 * 1024;
    const INSTALL: &'static str = "install";
    const CANCEL: &'static str = "cancel";

    /// Proposes the open page as an app and installs it once the user confirms the name, then starts the app;
    /// `when_started` runs once it was started, so the browser can hand the site over to it.
    pub fn offer(
        context: &Rc<AppContext>,
        web_view: &WebView,
        window: &adw::ApplicationWindow,
        toasts: &adw::ToastOverlay,
        when_started: impl Fn(&InstalledWebApp) + 'static,
    ) {
        let Some(page_url) = web_view.uri().map(String::from).filter(|address| Self::is_web_address(address)) else {
            toasts.add_toast(adw::Toast::new(tr!("Only web pages can be installed as apps")));
            return;
        };
        let context = Rc::clone(context);
        let web_view = web_view.clone();
        let window = window.clone();
        let toasts = toasts.clone();
        let when_started: Rc<dyn Fn(&InstalledWebApp)> = Rc::new(when_started);
        glib::spawn_future_local(async move {
            let draft = Self::draft(&context, &web_view, &page_url).await;
            Self::confirm(context, draft, web_view.favicon(), &window, toasts, when_started);
        });
    }

    async fn draft(context: &AppContext, web_view: &WebView, page_url: &str) -> WebAppDraft {
        let collected = web_view
            .evaluate_javascript_future(PageMetadata::COLLECTOR_SCRIPT, Some(Self::ISOLATED_WORLD), None)
            .await
            .ok()
            .and_then(|value| serde_json::from_str(&value.to_str()).ok());
        let metadata = collected.map(|collected| PageMetadata::from_json(&collected)).unwrap_or_default();
        let manifest = match &metadata.manifest_url {
            Some(manifest_url) => Self::manifest(context, manifest_url).await,
            None => None,
        };
        WebAppDraft::new(page_url, &metadata, manifest.as_ref())
    }

    async fn manifest(context: &AppContext, manifest_url: &str) -> Option<WebAppManifest> {
        let Ok(Fetched::Body { bytes, .. }) = context.http.get(manifest_url, Self::MANIFEST_MAX_BYTES, None).await
        else {
            return None;
        };
        let document = serde_json::from_slice(&bytes).ok()?;
        Some(WebAppManifest::from_json(&document, manifest_url))
    }

    fn confirm(
        context: Rc<AppContext>,
        draft: WebAppDraft,
        favicon: Option<gdk::Texture>,
        window: &adw::ApplicationWindow,
        toasts: adw::ToastOverlay,
        when_started: Rc<dyn Fn(&InstalledWebApp)>,
    ) {
        let host =
            Url::parse(&draft.start_url).ok().and_then(|url| url.host_str().map(str::to_owned)).unwrap_or_default();
        let dialog = adw::AlertDialog::new(
            Some(tr!("Install as App")),
            Some(&tr!(
                "{host} will open in its own window, with cookies and logins kept apart from the browser and every other app.",
                host = host
            )),
        );
        let name = gtk::Entry::builder().text(&draft.name).activates_default(true).build();
        dialog.set_extra_child(Some(&name));
        dialog.add_responses(&[(Self::CANCEL, tr!("_Cancel")), (Self::INSTALL, tr!("_Install"))]);
        dialog.set_response_appearance(Self::INSTALL, adw::ResponseAppearance::Suggested);
        dialog.set_default_response(Some(Self::INSTALL));
        dialog.set_close_response(Self::CANCEL);
        dialog.connect_response(Some(Self::INSTALL), move |_, _| {
            let chosen_name = name.text().trim().to_owned();
            let chosen_name = if chosen_name.is_empty() { draft.name.clone() } else { chosen_name };
            let (context, draft, favicon, toasts, when_started) =
                (Rc::clone(&context), draft.clone(), favicon.clone(), toasts.clone(), Rc::clone(&when_started));
            glib::spawn_future_local(async move {
                let toast = match Self::install(&context, &draft, &chosen_name, favicon).await {
                    Ok(app) if ExternalLauncher::start_web_app(&app.id) => {
                        when_started(&app);
                        adw::Toast::new(&tr!("Installed {name}", name = app.name))
                    }
                    Ok(app) => {
                        let toast = adw::Toast::builder()
                            .title(tr!("Installed {name}", name = app.name))
                            .button_label(tr!("Open"))
                            .build();
                        toast.connect_button_clicked(move |_| {
                            ExternalLauncher::start_web_app(&app.id);
                        });
                        toast
                    }
                    Err(error) => adw::Toast::new(&tr!("Could not install the app: {error}", error = error)),
                };
                toasts.add_toast(toast);
            });
        });
        dialog.present(Some(window));
    }

    async fn install(
        context: &AppContext,
        draft: &WebAppDraft,
        name: &str,
        favicon: Option<gdk::Texture>,
    ) -> Result<InstalledWebApp, Box<dyn Error>> {
        let mut catalog = context.catalogs.web_apps();
        let app = catalog.install(name, &draft.start_url)?.clone();
        let icon = match Self::icon_png(context, &draft.icons, favicon).await {
            Some(png) => {
                let icon = context.paths.web_app_icon(&app.id);
                StoredFile::new(&icon).write(&png)?;
                icon
            }
            None => context.icons.application_icon(),
        };
        let executable = ExternalLauncher::program()?;
        let entry = DesktopEntry::new(&app, &executable, &icon, InterfaceMessages::catalogue());
        StoredFile::new(context.paths.desktop_entry(&entry.file_name())).write(entry.render().as_bytes())?;
        context.catalogs.save_web_apps(&catalog)?;
        Ok(app)
    }

    /// The best icon that downloads as a sane PNG, else the favicon WebKit already decoded.
    async fn icon_png(context: &AppContext, icons: &[SiteIcon], favicon: Option<gdk::Texture>) -> Option<Vec<u8>> {
        let mut candidates = icons.to_vec();
        while let Some(best) = SiteIconChoice::new(&candidates).best().cloned() {
            candidates.retain(|icon| icon != &best);
            if let Some(png) = Self::downloaded_png(context, &best.url).await {
                return Some(png);
            }
        }
        favicon.map(|texture| texture.save_to_png_bytes().to_vec())
    }

    async fn downloaded_png(context: &AppContext, url: &str) -> Option<Vec<u8>> {
        let Ok(Fetched::Body { bytes, .. }) = context.http.get(url, Self::ICON_MAX_BYTES, None).await else {
            return None;
        };
        PngHeader::read(&bytes).filter(PngHeader::is_reasonable_icon)?;
        let texture = gdk::Texture::from_bytes(&glib::Bytes::from_owned(bytes)).ok()?;
        Some(texture.save_to_png_bytes().to_vec())
    }

    fn is_web_address(address: &str) -> bool {
        Url::parse(address).is_ok_and(|url| matches!(url.scheme(), "http" | "https"))
    }
}
