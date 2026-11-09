//! Managing profiles and installed web apps: open, create, delete.
//!
//! Deleting removes everything the profile or app stored on disk. It is refused while that profile or app is
//! running in another process, since its files are in use there.

use std::cell::RefCell;
use std::io;
use std::path::Path;
use std::rc::Rc;

use adw::prelude::ActionRowExt;
use adw::prelude::AdwDialogExt;
use adw::prelude::EntryRowExt;
use adw::prelude::PreferencesDialogExt;
use adw::prelude::PreferencesGroupExt;
use adw::prelude::PreferencesPageExt;
use gtk::prelude::ButtonExt;
use gtk::prelude::EditableExt;
use gtk::prelude::IsA;
use gtk::prelude::WidgetExt;
use ztbrowser_core::identity::application_id::ApplicationId;
use ztbrowser_core::profile::profile_directories::ProfileLocation;
use ztbrowser_core::profile::profile_id::ProfileId;
use ztbrowser_core::web_app::desktop_entry::DesktopEntry;
use ztbrowser_core::web_app::installed_web_app::InstalledWebApp;

use crate::app_context::AppContext;
use crate::external_launcher::ExternalLauncher;
use crate::running_instances::RunningInstances;
use crate::stored_file::StoredFile;

/// Rows of a preferences group that are rebuilt whenever the catalog changes.
struct RowList {
    group: adw::PreferencesGroup,
    rows: RefCell<Vec<adw::ActionRow>>,
}

impl RowList {
    fn new(title: &str, description: &str) -> Self {
        Self {
            group: adw::PreferencesGroup::builder().title(title).description(description).build(),
            rows: RefCell::new(Vec::new()),
        }
    }

    fn replace(&self, rows: Vec<adw::ActionRow>) {
        for old in self.rows.borrow_mut().drain(..) {
            self.group.remove(&old);
        }
        for row in &rows {
            self.group.add(row);
        }
        self.rows.replace(rows);
    }

    fn row(title: &str, subtitle: &str, buttons: &[gtk::Button]) -> adw::ActionRow {
        let row = adw::ActionRow::builder().title(title).subtitle(subtitle).build();
        for button in buttons {
            button.set_valign(gtk::Align::Center);
            row.add_suffix(button);
        }
        row
    }

    fn button(label: &str, destructive: bool, on_click: impl Fn() + 'static) -> gtk::Button {
        let button = gtk::Button::with_label(label);
        if destructive {
            button.add_css_class("destructive-action");
        }
        button.connect_clicked(move |_| on_click());
        button
    }
}

/// Keeps a dialog's owner alive while the dialog is open. The owner's buttons hold only weak references to it
/// (a strong one would form a cycle through the dialog), so without this the owner is freed as soon as
/// `present` returns and every button that needs it does nothing.
struct OpenDialog;

impl OpenDialog {
    fn keep_alive<Owner: 'static>(dialog: &adw::PreferencesDialog, owner: &Rc<Owner>) {
        let owner = RefCell::new(Some(Rc::clone(owner)));
        dialog.connect_closed(move |_| drop(owner.take()));
    }
}

/// Removes a profile's (or web app's) directories; missing directories are fine.
struct ProfileFiles;

impl ProfileFiles {
    fn delete(location: &ProfileLocation) -> io::Result<()> {
        [&location.data, &location.cache].into_iter().try_for_each(|directory| Self::delete_directory(directory))
    }

    fn delete_directory(directory: &Path) -> io::Result<()> {
        match std::fs::remove_dir_all(directory) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            result => result,
        }
    }

    /// Error text when `application_id` runs elsewhere, so its data must stay.
    fn in_use_message(name: &str, application_id: &ApplicationId) -> Option<String> {
        RunningInstances::may_be_running(application_id).then(|| tr!("Close {name} before deleting it", name = name))
    }
}

pub struct ProfilesDialog {
    context: Rc<AppContext>,
    dialog: adw::PreferencesDialog,
    profiles: RowList,
}

impl ProfilesDialog {
    pub fn present(context: &Rc<AppContext>, parent: &impl IsA<gtk::Widget>) {
        let dialog = adw::PreferencesDialog::builder().title(tr!("Profiles")).build();
        let page = adw::PreferencesPage::new();
        let profiles = RowList::new(
            tr!("Profiles"),
            tr!("Each profile keeps its own cookies, logins, history and cache, and runs as a separate process."),
        );
        let creation = adw::PreferencesGroup::new();
        let new_profile = adw::EntryRow::builder().title(tr!("New profile name")).show_apply_button(true).build();
        creation.add(&new_profile);
        page.add(&profiles.group);
        page.add(&creation);
        dialog.add(&page);
        let manager = Rc::new(Self { context: Rc::clone(context), dialog, profiles });
        let weak = Rc::downgrade(&manager);
        new_profile.connect_apply(move |entry| {
            let Some(manager) = weak.upgrade() else { return };
            let name = entry.text().trim().to_owned();
            if !name.is_empty() {
                manager.create(&name);
                entry.set_text("");
            }
        });
        manager.rebuild();
        OpenDialog::keep_alive(&manager.dialog, &manager);
        manager.dialog.present(Some(parent));
    }

    fn rebuild(self: &Rc<Self>) {
        let current = self.context.session.profile_id().cloned();
        let rows = self
            .context
            .catalogs
            .profiles()
            .profiles()
            .map(|profile| {
                let id = profile.id.clone();
                let mut buttons =
                    vec![RowList::button(tr!("Open"), false, move || ExternalLauncher::start_profile(&id))];
                if !profile.id.is_default() && Some(&profile.id) != current.as_ref() {
                    let (manager, id, name) = (Rc::downgrade(self), profile.id.clone(), profile.name.clone());
                    buttons.push(RowList::button(tr!("Delete"), true, move || {
                        if let Some(manager) = manager.upgrade() {
                            manager.delete(&id, &name);
                        }
                    }));
                }
                RowList::row(&profile.name, profile.id.as_str(), &buttons)
            })
            .collect();
        self.profiles.replace(rows);
    }

    fn create(self: &Rc<Self>, name: &str) {
        let mut catalog = self.context.catalogs.profiles();
        catalog.create_profile(name);
        self.report(self.context.catalogs.save_profiles(&catalog).err().map(|error| error.to_string()));
        self.rebuild();
    }

    fn delete(self: &Rc<Self>, id: &ProfileId, name: &str) {
        if let Some(message) = ProfileFiles::in_use_message(name, &id.application_id()) {
            self.report(Some(message));
            return;
        }
        let mut catalog = self.context.catalogs.profiles();
        catalog.remove_profile(id);
        let saved = self.context.catalogs.save_profiles(&catalog);
        let deleted = saved.and_then(|()| ProfileFiles::delete(&self.context.paths.profile(id)));
        self.report(deleted.err().map(|error| error.to_string()));
        self.rebuild();
    }

    fn report(&self, problem: Option<String>) {
        if let Some(problem) = problem {
            self.dialog.add_toast(adw::Toast::new(&problem));
        }
    }
}

pub struct WebAppsDialog {
    context: Rc<AppContext>,
    dialog: adw::PreferencesDialog,
    web_apps: RowList,
}

impl WebAppsDialog {
    pub fn present(context: &Rc<AppContext>, parent: &impl IsA<gtk::Widget>) {
        let dialog = adw::PreferencesDialog::builder().title(tr!("Installed Apps")).build();
        let page = adw::PreferencesPage::new();
        let web_apps = RowList::new(
            tr!("Installed Apps"),
            tr!("Each app runs in its own window and process, with cookies and logins of its own."),
        );
        page.add(&web_apps.group);
        dialog.add(&page);
        let manager = Rc::new(Self { context: Rc::clone(context), dialog, web_apps });
        manager.rebuild();
        OpenDialog::keep_alive(&manager.dialog, &manager);
        manager.dialog.present(Some(parent));
    }

    fn rebuild(self: &Rc<Self>) {
        let rows = self
            .context
            .catalogs
            .web_apps()
            .web_apps()
            .map(|app| {
                let id = app.id.clone();
                let open = RowList::button(tr!("Open"), false, move || {
                    ExternalLauncher::start_web_app(&id);
                });
                let (manager, installed) = (Rc::downgrade(self), app.clone());
                let uninstall = RowList::button(tr!("Uninstall"), true, move || {
                    if let Some(manager) = manager.upgrade() {
                        manager.uninstall(&installed);
                    }
                });
                RowList::row(&app.name, &app.host(), &[open, uninstall])
            })
            .collect();
        self.web_apps.replace(rows);
    }

    fn uninstall(self: &Rc<Self>, app: &InstalledWebApp) {
        if let Some(message) = ProfileFiles::in_use_message(&app.name, &app.id.application_id()) {
            self.dialog.add_toast(adw::Toast::new(&message));
            return;
        }
        let paths = &self.context.paths;
        let mut catalog = self.context.catalogs.web_apps();
        catalog.uninstall(&app.id);
        let icon = paths.web_app_icon(&app.id);
        let removed = self
            .context
            .catalogs
            .save_web_apps(&catalog)
            .and_then(|()| StoredFile::new(paths.desktop_entry(&DesktopEntry::file_name_for(&app.id))).remove())
            .and_then(|()| icon.parent().map_or(Ok(()), ProfileFiles::delete_directory))
            .and_then(|()| ProfileFiles::delete(&paths.profile(&app.id.profile_id())));
        if let Err(error) = removed {
            self.dialog.add_toast(adw::Toast::new(&tr!("Could not remove everything: {error}", error = error)));
        }
        self.rebuild();
    }
}
