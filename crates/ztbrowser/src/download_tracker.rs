//! Saves downloads where the person wants them, under a safe, never-overwriting name, and reports their
//! progress to the [`DownloadCenter`].
//!
//! The file name comes from the website and is cleaned by [`DownloadFileName`]. By default a save dialog asks
//! where to put each file (the person can turn that off in the preferences, then files go to their download
//! folder). Downloaded files are never opened automatically.

use std::cell::Cell;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use adw::prelude::WidgetExt;
use gio::prelude::FileExt;
use gtk::prelude::Cast;
use gtk::prelude::StaticType;
use webkit6::Download;
use webkit6::NetworkSession;
use ztbrowser_core::download::download_file_name::DownloadFileName;
use ztbrowser_core::download::download_history::DownloadId;

use crate::download_center::DownloadCenter;

pub struct DownloadTracker;

/// The download being followed: which entry of the list it is, once its destination is known.
type Tracked = Rc<Cell<Option<DownloadId>>>;

impl DownloadTracker {
    /// Progress is reported at most this often: a large file would otherwise rebuild the list thousands of times.
    const LOG_DOMAIN: &'static str = "ztbrowser";
    const PROGRESS_INTERVAL: Duration = Duration::from_millis(250);

    pub fn watch(session: &NetworkSession, center: &Rc<DownloadCenter>) -> Self {
        let center = Rc::clone(center);
        session.connect_download_started(move |_, download| Self::follow(download, &center));
        Self
    }

    fn follow(download: &Download, center: &Rc<DownloadCenter>) {
        let tracked: Tracked = Rc::new(Cell::new(None));
        let destination_center = Rc::clone(center);
        let destination_tracked = Rc::clone(&tracked);
        download.connect_decide_destination(move |download, suggested| {
            let folder = destination_center.folder();
            let name = DownloadFileName::from_suggestion(suggested);
            let proposal = name.first_free_path_in(&folder, Path::exists);
            if destination_center.asks_where_to_save() {
                Self::ask(download, &destination_center, &destination_tracked, &proposal);
            } else {
                Self::save(download, &destination_center, &destination_tracked, &proposal, false);
            }
            true
        });
        let progress_center = Rc::clone(center);
        let progress_tracked = Rc::clone(&tracked);
        let last_report = Cell::new(Instant::now());
        download.connect_received_data(move |download, _| {
            let Some(id) = progress_tracked.get() else { return };
            if last_report.get().elapsed() < Self::PROGRESS_INTERVAL {
                return;
            }
            last_report.set(Instant::now());
            let total = download.response().map(|response| response.content_length()).filter(|length| *length > 0);
            progress_center.progress(id, download.received_data_length(), total);
        });
        let failed = Rc::new(Cell::new(false));
        let failed_flag = Rc::clone(&failed);
        let failure_center = Rc::clone(center);
        let failure_tracked = Rc::clone(&tracked);
        download.connect_failed(move |download, error| {
            failed_flag.set(true);
            let cancelled = error.matches(webkit6::DownloadError::CancelledByUser);
            if let Some(id) = failure_tracked.get() {
                if cancelled { failure_center.discard(id) } else { failure_center.fail(id) }
            }
            if !cancelled {
                Self::announce(download, &tr!("Download failed: {error}", error = error), None);
            }
        });
        let finish_center = Rc::clone(center);
        download.connect_finished(move |download| {
            if failed.get() {
                return;
            }
            if let Some(id) = tracked.get() {
                finish_center.finish(id);
            }
            let destination = download.destination().map(|path| PathBuf::from(path.as_str()));
            let name = destination.as_deref().and_then(Path::file_name).map(|name| name.to_string_lossy().into_owned());
            Self::announce(download, &tr!("Downloaded {name}", name = name.unwrap_or_default()), destination);
        });
    }

    /// Lets the person pick the file's folder and name; dismissing the dialog cancels the download.
    fn ask(download: &Download, center: &Rc<DownloadCenter>, tracked: &Tracked, proposal: &Path) {
        let dialog = gtk::FileDialog::builder().title(tr!("Save File")).accept_label(tr!("_Save")).build();
        if let Some(folder) = proposal.parent() {
            dialog.set_initial_folder(Some(&gio::File::for_path(folder)));
        }
        if let Some(name) = proposal.file_name() {
            dialog.set_initial_name(Some(&name.to_string_lossy()));
        }
        let window =
            download.web_view().and_then(|view| view.root()).and_then(|root| root.downcast::<gtk::Window>().ok());
        let (download, center, tracked) = (download.clone(), Rc::clone(center), Rc::clone(tracked));
        let proposal = proposal.to_owned();
        dialog.save(window.as_ref(), None::<&gio::Cancellable>, move |chosen| match chosen {
            // The dialog already asked the person about replacing a file that exists.
            Ok(file) => match file.path() {
                Some(path) => Self::save(&download, &center, &tracked, &path, path.exists()),
                None => download.cancel(),
            },
            Err(error) if error.matches(gtk::DialogError::Dismissed) || error.matches(gtk::DialogError::Cancelled) => {
                download.cancel();
            }
            // The dialog could not be shown (no file chooser available): the file is wanted, so keep it.
            Err(error) => {
                glib::g_warning!(Self::LOG_DOMAIN, "cannot ask where to save the download: {error}");
                Self::save(&download, &center, &tracked, &proposal, false);
            }
        });
    }

    fn save(download: &Download, center: &Rc<DownloadCenter>, tracked: &Tracked, destination: &Path, overwrite: bool) {
        if let Some(directory) = destination.parent()
            && let Err(error) = std::fs::create_dir_all(directory)
        {
            Self::announce(download, &tr!("Cannot save download: {error}", error = error), None);
            download.cancel();
            return;
        }
        download.set_allow_overwrite(overwrite);
        download.set_destination(&destination.to_string_lossy());
        tracked.set(Some(center.begin(destination.to_owned())));
    }

    /// A toast in the window the download came from, with "Show in Folder" when a file was saved.
    fn announce(download: &Download, text: &str, saved: Option<PathBuf>) {
        let Some(overlay) = download
            .web_view()
            .and_then(|view| view.ancestor(adw::ToastOverlay::static_type()))
            .and_then(|widget| widget.downcast::<adw::ToastOverlay>().ok())
        else {
            return;
        };
        let toast = adw::Toast::new(text);
        if let Some(saved) = saved {
            toast.set_button_label(Some(tr!("Show in Folder")));
            let window = overlay.root().and_then(|root| root.downcast::<gtk::Window>().ok());
            toast.connect_button_clicked(move |_| {
                gtk::FileLauncher::new(Some(&gio::File::for_path(&saved))).open_containing_folder(
                    window.as_ref(),
                    None::<&gio::Cancellable>,
                    |_| {},
                );
            });
        }
        overlay.add_toast(toast);
    }
}
