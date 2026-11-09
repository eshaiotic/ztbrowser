//! Whether ZTBrowser opens the web links of other programs, making it do so, and asking the person about it.
//!
//! The desktop decides which browser opens a link clicked in a mail, chat or document; each desktop keeps that choice
//! in its own place (KDE's `kdeglobals`, the `mimeapps.list` of the freedesktop specification, GNOME's settings).
//! `xdg-settings` from `xdg-utils` knows them all, so ZTBrowser asks it instead of writing any of them itself. A
//! desktop can only choose a program that has a menu entry: an installed package has one, an AppImage writes its own,
//! a development build has none and is never offered.
//!
//! Nothing is ever changed without the person asking for it: the window offers the change at most once a process,
//! and never again once the person says so.

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;
use std::sync::PoisonError;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::thread;
use std::thread::JoinHandle;

use serde_json::Value;
use serde_json::json;
use ztbrowser_core::launch::program_location::ProgramLocation;
use ztbrowser_core::storage::owned_json_file::OwnedJsonFile;

use crate::appimage_integration::AppImageIntegration;
use crate::installation_directories::InstallationDirectories;

/// Which browser opens the web links of other programs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefaultBrowserState {
    /// No menu entry names ZTBrowser, so the desktop cannot choose it: a development build, or an AppImage that did
    /// not put itself into the menu.
    Unregistered,
    ThisBrowser,
    /// Another browser, or none: its menu name, when the desktop names one whose entry ZTBrowser finds.
    OtherBrowser(Option<String>),
    /// The desktop's tool is missing or failed.
    Unknown,
}

impl DefaultBrowserState {
    /// The state as the window reads it: `{"state": "this" | "other" | "unregistered" | "unknown", "browser": name}`.
    #[must_use]
    pub fn to_json(&self) -> Value {
        let (state, browser) = match self {
            Self::Unregistered => ("unregistered", ""),
            Self::ThisBrowser => ("this", ""),
            Self::OtherBrowser(name) => ("other", name.as_deref().unwrap_or_default()),
            Self::Unknown => ("unknown", ""),
        };
        json!({ "state": state, "browser": browser })
    }
}

/// The desktop's default-browser setting, read and changed through `xdg-settings`.
#[derive(Debug, Clone)]
pub struct DefaultBrowser {
    tool: PathBuf,
    /// The folders the desktop reads menu entries from: the person's own first, then the system's.
    applications_directories: Vec<PathBuf>,
    /// Variables of this process the tool must not inherit.
    hidden_variables: &'static [&'static str],
}

impl DefaultBrowser {
    const SETTING: &'static str = "default-web-browser";
    const ENTRY: &'static str = AppImageIntegration::ENTRY_FILE;
    /// An AppImage points these at its own Qt; the desktop's tools (`kwriteconfig6`, `qtpaths`) are built against
    /// the system's Qt and fail to start with another one.
    const APPIMAGE_VARIABLES: &'static [&'static str] =
        &["LD_LIBRARY_PATH", "QT_PLUGIN_PATH", "QML_IMPORT_PATH", "QML2_IMPORT_PATH"];

    #[must_use]
    pub fn new(tool: PathBuf, applications_directories: Vec<PathBuf>) -> Self {
        Self { tool, applications_directories, hidden_variables: &[] }
    }

    /// The `xdg-settings` on the `PATH`, and the menu entries of the person and of the system.
    #[must_use]
    pub fn for_environment(directories: &InstallationDirectories) -> Self {
        let mut applications_directories = vec![directories.applications_directory().to_path_buf()];
        applications_directories.extend(
            InstallationDirectories::system_data_directories().iter().map(|directory| directory.join("applications")),
        );
        let is_appimage = std::env::var_os(ProgramLocation::APPIMAGE_VARIABLE).is_some();
        Self {
            hidden_variables: if is_appimage { Self::APPIMAGE_VARIABLES } else { &[] },
            ..Self::new(PathBuf::from("xdg-settings"), applications_directories)
        }
    }

    /// Asks the desktop which browser opens links; takes up to a tenth of a second.
    #[must_use]
    pub fn state(&self) -> DefaultBrowserState {
        if self.menu_entry(Self::ENTRY).is_none() {
            return DefaultBrowserState::Unregistered;
        }
        let Ok(current) = self.run(&["get", Self::SETTING]) else { return DefaultBrowserState::Unknown };
        let current = current.trim();
        if current == Self::ENTRY {
            DefaultBrowserState::ThisBrowser
        } else {
            DefaultBrowserState::OtherBrowser(self.menu_entry(current).and_then(|entry| Self::menu_name(&entry)))
        }
    }

    /// Makes ZTBrowser open the web links of other programs.
    ///
    /// # Errors
    /// Fails without asking the desktop when ZTBrowser has no menu entry, and with the tool's own message when it
    /// is missing or refuses.
    pub fn make_default(&self) -> io::Result<()> {
        if self.menu_entry(Self::ENTRY).is_none() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "ZTBrowser has no menu entry the desktop could choose",
            ));
        }
        self.run(&["set", Self::SETTING, Self::ENTRY]).map(drop)
    }

    /// The tool's output; its error message when it fails.
    fn run(&self, arguments: &[&str]) -> io::Result<String> {
        let mut command = Command::new(&self.tool);
        command.args(arguments).stdin(Stdio::null());
        for variable in self.hidden_variables {
            command.env_remove(variable);
        }
        let output = command.output()?;
        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }
        let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(io::Error::other(if message.is_empty() { output.status.to_string() } else { message }))
    }

    /// The first menu entry called `name`, in the order the desktop reads them; nothing for an empty or odd name.
    fn menu_entry(&self, name: &str) -> Option<PathBuf> {
        let is_plain_name = !name.is_empty() && Path::new(name).file_name().is_some_and(|file| file == name);
        if !is_plain_name {
            return None;
        }
        self.applications_directories.iter().map(|directory| directory.join(name)).find(|entry| entry.is_file())
    }

    /// The `Name=` of the entry's `[Desktop Entry]` group (actions have names of their own).
    fn menu_name(entry: &Path) -> Option<String> {
        let text = fs::read_to_string(entry).ok()?;
        let mut is_main_group = false;
        for line in text.lines().map(str::trim) {
            if line.starts_with('[') {
                is_main_group = line == "[Desktop Entry]";
            } else if let Some(name) = line.strip_prefix("Name=").filter(|_| is_main_group) {
                return Some(name.trim().to_owned()).filter(|name| !name.is_empty());
            }
        }
        None
    }
}

/// What the desktop said last, or that it is still being asked.
#[derive(Debug, Clone)]
enum DefaultBrowserAnswer {
    Pending,
    Known { state: DefaultBrowserState, error: Option<String> },
}

/// The default-browser question in this process, for every window.
///
/// The desktop is asked on a thread of its own: `xdg-settings` takes up to a tenth of a second, too long for the
/// thread that draws the windows. Only the newest question writes its answer, so a check started before a change
/// never overwrites what the change found.
pub struct DefaultBrowserChoice {
    browser: DefaultBrowser,
    /// The number of the newest question, and the answer to it.
    answer: Arc<Mutex<(u64, DefaultBrowserAnswer)>>,
    question: OwnedJsonFile,
    asks_at_start: AtomicBool,
    offered: AtomicBool,
}

static CURRENT: OnceLock<DefaultBrowserChoice> = OnceLock::new();

impl DefaultBrowserChoice {
    const ASKS_AT_START: &'static str = "asksAtStart";

    /// `question_file` keeps whether the person still wants to be asked; a missing file means yes.
    #[must_use]
    pub fn open(browser: DefaultBrowser, question_file: &Path) -> Self {
        let question = OwnedJsonFile::new(question_file.to_path_buf());
        let asks_at_start =
            question.read().and_then(|document| document[Self::ASKS_AT_START].as_bool()).unwrap_or(true);
        Self {
            browser,
            answer: Arc::new(Mutex::new((0, DefaultBrowserAnswer::Pending))),
            question,
            asks_at_start: AtomicBool::new(asks_at_start),
            offered: AtomicBool::new(false),
        }
    }

    /// Makes this the choice of the process; a second call is ignored.
    pub fn install(self) {
        let _ = CURRENT.set(self);
    }

    #[must_use]
    pub fn current() -> Option<&'static Self> {
        CURRENT.get()
    }

    /// Asks the desktop which browser opens links; `answer_json` tells once it knows.
    pub fn check(&self) -> JoinHandle<()> {
        self.ask(|browser| (browser.state(), None))
    }

    /// Makes ZTBrowser the default browser; `answer_json` tells what the desktop then says, and why it failed.
    pub fn make_default(&self) -> JoinHandle<()> {
        self.ask(|browser| {
            let error = browser.make_default().err().map(|error| error.to_string());
            (browser.state(), error)
        })
    }

    /// `{"state": "checking" | "this" | "other" | "unregistered" | "unknown", "browser": name, "error": message}`.
    #[must_use]
    pub fn answer_json(&self) -> Value {
        match self.answer.lock().unwrap_or_else(PoisonError::into_inner).1.clone() {
            DefaultBrowserAnswer::Pending => json!({ "state": "checking", "browser": "", "error": "" }),
            DefaultBrowserAnswer::Known { state, error } => {
                let mut answer = state.to_json();
                answer["error"] = Value::from(error.unwrap_or_default());
                answer
            }
        }
    }

    /// Whether the person still wants to be asked when ZTBrowser starts.
    #[must_use]
    pub fn asks_at_start(&self) -> bool {
        self.asks_at_start.load(Ordering::Relaxed)
    }

    /// Whether to ask when ZTBrowser starts, from now on: in this process and in every later one.
    pub fn set_asks_at_start(&self, asks: bool) {
        self.asks_at_start.store(asks, Ordering::Relaxed);
        let _ = self.question.write(&json!({ Self::ASKS_AT_START: asks }));
    }

    /// Whether the window asking is to offer the change: only the first to ask once the desktop has named another
    /// browser, and only while the person wants to be asked.
    pub fn takes_offer(&self) -> bool {
        let answer = self.answer.lock().unwrap_or_else(PoisonError::into_inner).1.clone();
        let names_another_browser =
            matches!(answer, DefaultBrowserAnswer::Known { state: DefaultBrowserState::OtherBrowser(_), .. });
        self.asks_at_start() && names_another_browser && !self.offered.swap(true, Ordering::Relaxed)
    }

    fn ask(
        &self,
        question: impl FnOnce(&DefaultBrowser) -> (DefaultBrowserState, Option<String>) + Send + 'static,
    ) -> JoinHandle<()> {
        let number = {
            let mut answer = self.answer.lock().unwrap_or_else(PoisonError::into_inner);
            *answer = (answer.0 + 1, DefaultBrowserAnswer::Pending);
            answer.0
        };
        let browser = self.browser.clone();
        let answer = Arc::clone(&self.answer);
        thread::spawn(move || {
            let (state, error) = question(&browser);
            let mut answer = answer.lock().unwrap_or_else(PoisonError::into_inner);
            if answer.0 == number {
                answer.1 = DefaultBrowserAnswer::Known { state, error };
            }
        })
    }
}
