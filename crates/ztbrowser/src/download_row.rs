//! One download as a row: its name and where it is (or how far it got), a button that shows its folder, and,
//! in the manager, a button that takes it out of the history.
//!
//! Clicking a finished download opens the file, but only a file that merely displays something (see
//! [`PassiveContentType`]) and never in the Tor window; any other file is shown in its folder instead, and a
//! file that is gone says so.

use std::path::Path;
use std::rc::Rc;

use gio::prelude::FileExt;
use gtk::prelude::BoxExt;
use gtk::prelude::ButtonExt;
use gtk::prelude::Cast;
use gtk::prelude::WidgetExt;
use ztbrowser_core::download::download_history::DownloadEntry;
use ztbrowser_core::download::download_history::DownloadId;
use ztbrowser_core::download::download_history::DownloadState;
use ztbrowser_core::download::passive_content_type::PassiveContentType;

pub struct DownloadRow;

impl DownloadRow {
    /// `opens_files` is `false` in the Tor window: a file opened in another application would connect outside
    /// Tor. `remover` adds the "Remove from List" button.
    pub fn build(entry: &DownloadEntry, opens_files: bool, remover: Option<Rc<dyn Fn(DownloadId)>>) -> gtk::Box {
        let row = gtk::Box::builder().spacing(4).build();
        let exists = entry.path().exists();
        let finished = *entry.state() == DownloadState::Finished;
        let name = gtk::Label::builder()
            .label(entry.name())
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::Middle)
            .max_width_chars(34)
            .css_classes(["body"])
            .build();
        let status = gtk::Label::builder()
            .label(Self::status_of(entry, exists))
            .xalign(0.0)
            .ellipsize(gtk::pango::EllipsizeMode::Start)
            .max_width_chars(40)
            .css_classes(["caption", "dim-label"])
            .build();
        let text = gtk::Box::builder().orientation(gtk::Orientation::Vertical).build();
        text.append(&name);
        text.append(&status);
        let open = gtk::Button::builder().child(&text).css_classes(["flat"]).hexpand(true).build();
        let path = entry.path().to_owned();
        open.set_sensitive(finished && exists);
        let opens_directly = opens_files && Self::is_passive(&path);
        if !opens_files {
            open.set_tooltip_text(Some(tr!(
                "Other applications do not open from the Tor window: they would connect outside Tor."
            )));
        }
        open.connect_clicked(move |button| {
            let launcher = gtk::FileLauncher::new(Some(&gio::File::for_path(&path)));
            let window = Self::window_of(button);
            if opens_directly {
                launcher.launch(window.as_ref(), None::<&gio::Cancellable>, |_| {});
            } else {
                launcher.open_containing_folder(window.as_ref(), None::<&gio::Cancellable>, |_| {});
            }
        });
        row.append(&open);
        row.append(&Self::folder_button(entry.path()));
        if let Some(remover) = remover {
            let id = entry.id();
            let remove = Self::icon_button("window-close-symbolic", tr!("Remove from List"));
            remove.connect_clicked(move |_| remover(id));
            row.append(&remove);
        }
        row
    }

    fn folder_button(path: &Path) -> gtk::Button {
        let folder = Self::icon_button("folder-open-symbolic", tr!("Show in Folder"));
        let path = path.to_owned();
        folder.connect_clicked(move |button| {
            gtk::FileLauncher::new(Some(&gio::File::for_path(&path))).open_containing_folder(
                Self::window_of(button).as_ref(),
                None::<&gio::Cancellable>,
                |_| {},
            );
        });
        folder
    }

    fn icon_button(icon: &str, tooltip: &str) -> gtk::Button {
        gtk::Button::builder()
            .icon_name(icon)
            .tooltip_text(tooltip)
            .valign(gtk::Align::Center)
            .css_classes(["flat", "circular"])
            .build()
    }

    /// Only a file that merely displays something opens with one click, judged by what the file really is (its
    /// content, not just its name); a program, script or launcher is shown in its folder instead. Anything
    /// that cannot be examined counts as not passive.
    fn is_passive(path: &Path) -> bool {
        gio::File::for_path(path)
            .query_info("standard::content-type", gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS, None::<&gio::Cancellable>)
            .ok()
            .and_then(|info| info.content_type())
            .is_some_and(|content_type| PassiveContentType::new(&content_type).is_passive())
    }

    /// Where the file is, how far the download got, or that the file is gone.
    fn status_of(entry: &DownloadEntry, exists: bool) -> String {
        match entry.state() {
            DownloadState::Finished if !exists => tr!("File Moved or Deleted").to_owned(),
            DownloadState::Finished => {
                entry.path().parent().map(Path::display).map(|folder| folder.to_string()).unwrap_or_default()
            }
            DownloadState::Failed => tr!("Download failed").to_owned(),
            DownloadState::InProgress { received, total: Some(total) } if *total > 0 => {
                tr!("Downloading… {percent}%", percent = received.saturating_mul(100) / total)
            }
            DownloadState::InProgress { .. } => tr!("Downloading…").to_owned(),
        }
    }

    fn window_of(button: &gtk::Button) -> Option<gtk::Window> {
        button.root().and_then(|root| root.downcast::<gtk::Window>().ok())
    }
}
