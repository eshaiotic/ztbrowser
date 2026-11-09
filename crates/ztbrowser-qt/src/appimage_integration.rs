//! Puts a running AppImage into the application menu, so the desktop can show its icon.
//!
//! A Wayland desktop gives a window the icon of the `.desktop` file named like the window's `app_id`; a package
//! installs that file, an AppImage does not. Without it the window shows a generic Wayland icon. The entry lives in
//! the person's own data directory and names the AppImage file, so moving the file only needs one more start.
//! A packaged ZTBrowser already owns the entry system-wide and is left alone; an entry the AppImage wrote earlier is
//! withdrawn once a package is installed, because the person's entry wins over the system one and would keep
//! starting the AppImage instead of the package.
//!
//! The entry makes the desktop run the AppImage file whenever the person clicks it, so it is written only for a
//! file that nobody else can replace: the path is resolved first (the entry names the real file), and the file
//! and every folder above it up to the person's home folder (to `/` for a file outside it) belong to root or to the
//! person and are not writable by the group or by others. A shared folder such as `/tmp` never qualifies, sticky
//! bit or not: after a clean-up anyone could recreate the path and put a file of that name in its place. The entry
//! is withdrawn at the next start when the file it names stops qualifying, whichever AppImage starts then. An entry
//! the person wrote themselves is never overwritten or removed.

use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use ztbrowser_core::launch::appimage_menu_entry::AppImageMenuEntry;

use crate::installation_directories::InstallationDirectories;

pub struct AppImageIntegration {
    applications: PathBuf,
    /// The owner of the person's data directory, who is trusted to own the AppImage and its folders (root is too).
    person: u32,
    /// The person's home folder, resolved; the walk up from the AppImage ends there. Unknown: the walk ends at `/`.
    home: Option<PathBuf>,
    icons: PathBuf,
    system_data_directories: Vec<PathBuf>,
}

impl AppImageIntegration {
    /// The `.desktop` file name the window's `app_id` selects.
    pub const ENTRY_FILE: &'static str = "io.github.ztbrowser.ZTBrowser.desktop";
    const ICON_FILE: &'static str = "io.github.ztbrowser.ZTBrowser.svg";
    const PACKAGED_ENTRY: &'static str = include_str!("../../../packaging/linux/io.github.ztbrowser.ZTBrowser.desktop");
    const ICON: &'static str = include_str!("../../../assets/icons/io.github.ztbrowser.ZTBrowser.svg");
    /// The group-write and other-write permission bits.
    const WRITABLE_BY_OTHERS: u32 = 0o022;
    const ROOT: u32 = 0;

    #[must_use]
    pub fn new(directories: &InstallationDirectories, home: &Path, system_data_directories: Vec<PathBuf>) -> Self {
        let applications = directories.applications_directory().to_path_buf();
        // The data directory may not exist yet: its first existing parent belongs to the same person.
        let person =
            applications.ancestors().find_map(|folder| fs::metadata(folder).ok()).map_or(0, |folder| folder.uid());
        Self {
            applications,
            person,
            home: fs::canonicalize(home).ok(),
            icons: directories.application_icons_directory(),
            system_data_directories,
        }
    }

    /// Looks for an installed package in `$XDG_DATA_DIRS`, or in the specification's default directories.
    #[must_use]
    pub fn for_environment(directories: &InstallationDirectories) -> Self {
        let home = std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        Self::new(directories, &home, InstallationDirectories::system_data_directories())
    }

    /// Writes the menu entry and the icon for `appimage`, unless a package owns the entry or the files are current.
    ///
    /// # Errors
    /// Returns the I/O error when the person's data directory cannot be written.
    pub fn install(&self, appimage: &Path) -> io::Result<()> {
        let entry_file = self.applications.join(Self::ENTRY_FILE);
        if let Ok(text) = fs::read_to_string(&entry_file) {
            match AppImageMenuEntry::file_named_by(&text) {
                // Not ours: the person wrote it.
                None => return Ok(()),
                // Ours, but a package owns the entry now (ours would shadow it and keep starting the AppImage),
                // or the file it makes the desktop run no longer qualifies: whichever AppImage starts now.
                Some(named) if self.is_installed_system_wide() || self.trusted_file(&named).is_none() => {
                    fs::remove_file(&entry_file)?;
                }
                Some(_) => {}
            }
        }
        if self.is_installed_system_wide() {
            return Ok(());
        }
        let Some(real) = self.trusted_file(appimage) else { return Ok(()) };
        let icon = self.icons.join(Self::ICON_FILE);
        Self::write_when_changed(&icon, Self::ICON)?;
        let entry = AppImageMenuEntry::new(Self::PACKAGED_ENTRY, &real, &icon).render();
        Self::write_when_changed(&entry_file, &entry)
    }

    /// The real path of `appimage` when nobody but root and the person could replace it, else nothing.
    fn trusted_file(&self, appimage: &Path) -> Option<PathBuf> {
        let real = fs::canonicalize(appimage).ok()?;
        let file = fs::metadata(&real).ok().filter(fs::Metadata::is_file)?;
        let mut folders = real.ancestors().skip(1).take_while(|folder| Some(*folder) != self.home.as_deref());
        let folders_are_closed =
            folders.all(|folder| fs::metadata(folder).is_ok_and(|metadata| self.is_closed(&metadata)));
        (self.is_closed(&file) && folders_are_closed).then_some(real)
    }

    /// Owned by root or the person, and not writable by the group or by others.
    fn is_closed(&self, metadata: &fs::Metadata) -> bool {
        let is_trusted_owner = metadata.uid() == Self::ROOT || metadata.uid() == self.person;
        is_trusted_owner && metadata.permissions().mode() & Self::WRITABLE_BY_OTHERS == 0
    }

    fn is_installed_system_wide(&self) -> bool {
        self.system_data_directories
            .iter()
            .any(|directory| directory.join("applications").join(Self::ENTRY_FILE).exists())
    }

    fn write_when_changed(path: &Path, content: &str) -> io::Result<()> {
        if fs::read_to_string(path).is_ok_and(|current| current == content) {
            return Ok(());
        }
        if let Some(directory) = path.parent() {
            fs::create_dir_all(directory)?;
        }
        fs::write(path, content)
    }
}
