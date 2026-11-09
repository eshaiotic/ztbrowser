//! The Fire button: burning a profile's data by restarting the browser.
//!
//! Chromium keeps its storage open while it runs and offers no way to delete all of it at once, so the button
//! leaves a marker and restarts. The next start deletes the profile's engine and cache directories before the
//! engine opens them (see `BurnPlanner`), then goes on with a clean profile. The marker holds no paths: what is
//! deleted is always worked out from the profile's own location, so a doctored marker cannot point elsewhere.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use ztbrowser_core::profile::profile_directories::ProfileLocation;
use ztbrowser_core::storage::burn_planner::BurnPlanner;
use ztbrowser_core::storage::burn_scope::BurnScope;

pub struct BurnRestart {
    location: ProfileLocation,
}

impl BurnRestart {
    const MARKER: &'static str = "burn-pending";
    /// Files of what the person did (not of what they chose), deleted together with the engine's storage.
    const HISTORY_FILES: [&'static str; 2] = ["downloads.json", "site-zoom-levels.json"];
    /// Set by the AppImage runtime to the image's path: the mounted copy of the program disappears when it quits.
    const APPIMAGE_VARIABLE: &'static str = "APPIMAGE";
    /// How often the restart script looks whether the old process has gone, and for how many looks (30 s).
    const WAIT_LOOKS: u32 = 150;
    /// Positional parameters: `$0` the script's name, `$1` the id of the process to wait for, `$2` how many
    /// times to look (0.2 s apart), then the program and its arguments. Starts nothing while the old process
    /// is still alive: two instances must not share one profile directory.
    const RESTART_SCRIPT: &'static str = "pid=$1; looks=$2; shift 2; n=0; while kill -0 \"$pid\" 2>/dev/null; do [ \"$n\" -ge \"$looks\" ] && exit 1; sleep 0.2; n=$((n+1)); done; exec \"$@\"";

    #[must_use]
    pub fn new(location: ProfileLocation) -> Self {
        Self { location }
    }

    /// Marks the profile for burning at the next start.
    ///
    /// # Errors
    /// Fails when the marker cannot be written; nothing is then deleted.
    pub fn request(&self) -> io::Result<()> {
        fs::create_dir_all(&self.location.data)?;
        fs::write(self.marker(), b"")
    }

    /// Starts the browser again, as the image or program that is running now, once this process has quit.
    ///
    /// # Errors
    /// Fails when the program cannot be started; the caller then keeps the running browser.
    pub fn start_new_instance(arguments: &[String]) -> io::Result<()> {
        let program = std::env::var_os(Self::APPIMAGE_VARIABLE)
            .map_or_else(std::env::current_exe, |image| Ok(PathBuf::from(image)))?;
        Self::restart_command(std::process::id(), Self::WAIT_LOOKS, &program, arguments).spawn().map(drop)
    }

    /// The command that waits for process `waiting_for` to exit (looking `looks` times) and then runs `program`.
    #[must_use]
    pub fn restart_command(waiting_for: u32, looks: u32, program: &Path, arguments: &[String]) -> Command {
        let mut command = Command::new("sh");
        command
            .arg("-c")
            .arg(Self::RESTART_SCRIPT)
            .arg("ztbrowser-restart")
            .arg(waiting_for.to_string())
            .arg(looks.to_string())
            .arg(program)
            .args(arguments);
        command
    }

    /// Deletes what a requested burn names. Call before the engine starts; returns whether a burn was pending.
    ///
    /// # Errors
    /// Fails when a directory cannot be deleted; the marker then stays, so the next start tries again.
    pub fn complete_pending(&self) -> io::Result<bool> {
        if !self.marker().exists() {
            return Ok(false);
        }
        for directory in BurnPlanner::new(self.location.clone()).plan(&BurnScope::Everything).storage_directories {
            match fs::remove_dir_all(&directory) {
                Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
                _ => {}
            }
        }
        for file in Self::HISTORY_FILES {
            match fs::remove_file(self.location.data.join(file)) {
                Err(error) if error.kind() != io::ErrorKind::NotFound => return Err(error),
                _ => {}
            }
        }
        fs::remove_file(self.marker())?;
        Ok(true)
    }

    fn marker(&self) -> PathBuf {
        self.location.data.join(Self::MARKER)
    }
}
