//! Where the Tor window finds `tor` and the pluggable transport programs.
//!
//! The AppImage and the Debian package carry their own `tor` in `lib/ztbrowser/tor/` next to ZTBrowser's `bin/`
//! (the AppImage with the libraries it needs in `lib/ztbrowser/tor/lib/`): Debian's `tor` package would also
//! start a system-wide Tor service at every boot, which the Tor window does not need. The Arch and Fedora
//! packages depend on their distribution's `tor`, which starts no service. So the bundled directory is
//! searched first, then the system's program directories. `PATH` is never searched: a program planted in a
//! user-writable directory on the `PATH` must not become the Tor window's Tor.

use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use crate::tor::pluggable_transport::PluggableTransport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TorPrograms {
    tor: PathBuf,
    library_directory: Option<PathBuf>,
    transports: BTreeMap<PluggableTransport, PathBuf>,
}

impl TorPrograms {
    /// Relative to the installation prefix (the parent of the directory holding `ztbrowser`).
    pub const BUNDLED_DIRECTORY: &'static str = "lib/ztbrowser/tor";
    pub const SYSTEM_DIRECTORIES: [&'static str; 3] = ["/usr/bin", "/usr/sbin", "/usr/local/bin"];
    const TOR: &'static str = "tor";
    const LIBRARY_DIRECTORY: &'static str = "lib";

    /// Finds the programs for ZTBrowser's `executable` (inside an AppImage, its path in the mounted image);
    /// `is_program` tells whether a path is an executable file. `None` when there is no `tor` at all.
    pub fn locate(executable: &Path, is_program: impl Fn(&Path) -> bool) -> Option<Self> {
        let bundled = executable.parent().and_then(Path::parent).map(|prefix| prefix.join(Self::BUNDLED_DIRECTORY));
        let directories: Vec<PathBuf> =
            bundled.iter().cloned().chain(Self::SYSTEM_DIRECTORIES.into_iter().map(PathBuf::from)).collect();
        let find = |name: &str| directories.iter().map(|directory| directory.join(name)).find(|path| is_program(path));
        let tor = find(Self::TOR)?;
        let library_directory = bundled
            .filter(|bundled_directory| tor.parent() == Some(bundled_directory.as_path()))
            .map(|bundled_directory| bundled_directory.join(Self::LIBRARY_DIRECTORY));
        let transports = PluggableTransport::ALL
            .into_iter()
            .filter_map(|transport| {
                transport.program_names().iter().find_map(|name| find(name)).map(|program| (transport, program))
            })
            .collect();
        Some(Self { tor, library_directory, transports })
    }

    #[must_use]
    pub fn tor(&self) -> &Path {
        &self.tor
    }

    /// The libraries that came with a bundled `tor`, for its `LD_LIBRARY_PATH`; `None` for the system's `tor`.
    #[must_use]
    pub fn library_directory(&self) -> Option<&Path> {
        self.library_directory.as_deref()
    }

    #[must_use]
    pub fn program_for(&self, transport: PluggableTransport) -> Option<&Path> {
        self.transports.get(&transport).map(PathBuf::as_path)
    }
}
