//! The Tor window's own `tor`: started by this process, configured by ZTBrowser, followed over its control port.
//!
//! [`TorConnection::start`] writes Tor's configuration ([`TorConfiguration`]), starts `tor`, waits for the file
//! naming its control port, proves access with the cookie Tor wrote, takes ownership (Tor exits when the
//! connection closes, so when ZTBrowser exits or crashes) and follows Tor's bootstrap to 100 %, telling every
//! observer each step: the connection page of each Tor window, and the session, which sends its traffic
//! through the SOCKS port once Tor is connected. Choosing other bridges restarts Tor; New Identity asks Tor
//! for new circuits and moves the session to new SOCKS credentials, which Tor isolates from the old ones.

use std::cell::Cell;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use gio::prelude::DataInputStreamExtManual;
use ztbrowser_core::tor::bootstrap_status::BootstrapStatus;
use ztbrowser_core::tor::bridge_choice::BridgeChoice;
use ztbrowser_core::tor::control_command::ControlCommand;
use ztbrowser_core::tor::control_reply::ControlReply;
use ztbrowser_core::tor::tor_configuration::TorConfiguration;
use ztbrowser_core::tor::tor_endpoint::TorEndpoint;
use ztbrowser_core::tor::tor_programs::TorPrograms;

use crate::installation_paths::InstallationPaths;
use crate::interface_messages::InterfaceMessages;
use crate::stored_file::StoredFile;
use crate::tor_control_port::TorControlPort;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TorState {
    Starting,
    Connecting(BootstrapStatus),
    /// Ready: requests go through `proxy_uri`.
    Connected {
        proxy_uri: String,
    },
    Failed(String),
}

/// The `tor` process of one start, and what is known about it so far.
struct RunningTor {
    process: gio::Subprocess,
    control: Option<Rc<TorControlPort>>,
    socks: Option<TorEndpoint>,
}

type TorObserver = Box<dyn Fn(&TorState) -> glib::ControlFlow>;

pub struct TorConnection {
    data_directory: PathBuf,
    settings_file: StoredFile,
    bridges: RefCell<BridgeChoice>,
    state: RefCell<TorState>,
    observers: RefCell<Vec<TorObserver>>,
    running: RefCell<Option<RunningTor>>,
    /// Bumped by every start; the work of an older start stops at its next step.
    generation: Cell<u64>,
    /// The SOCKS credentials of the current identity (see [`TorEndpoint::socks_proxy_uri`]).
    identity: Cell<u32>,
    /// Tor's last log lines, shown when Tor stops unexpectedly.
    last_log: RefCell<VecDeque<String>>,
}

impl TorConnection {
    const LOG_DOMAIN: &'static str = "ztbrowser";
    const SUPERSEDED: &'static str = "superseded by a newer start";
    const TERMINATE_SIGNAL: i32 = 15;
    const KEPT_LOG_LINES: usize = 4;
    const CONTROL_PORT_POLL: Duration = Duration::from_millis(100);
    const CONTROL_PORT_TIMEOUT: Duration = Duration::from_secs(60);
    const EXIT_TIMEOUT: Duration = Duration::from_secs(5);

    pub fn new(paths: &InstallationPaths) -> Rc<Self> {
        let settings_file = StoredFile::new(paths.tor_connection_settings());
        let bridges = settings_file.read_json().map(|document| BridgeChoice::from_json(&document)).unwrap_or_default();
        Rc::new(Self {
            data_directory: paths.tor_data(),
            settings_file,
            bridges: RefCell::new(bridges),
            state: RefCell::new(TorState::Starting),
            observers: RefCell::new(Vec::new()),
            running: RefCell::new(None),
            generation: Cell::new(0),
            identity: Cell::new(1),
            last_log: RefCell::new(VecDeque::new()),
        })
    }

    /// Calls `observer` with the current state now and with every later one, until it returns `Break`.
    pub fn observe(&self, observer: impl Fn(&TorState) -> glib::ControlFlow + 'static) {
        let state = self.state.borrow().clone();
        if observer(&state).is_continue() {
            self.observers.borrow_mut().push(Box::new(observer));
        }
    }

    pub fn bridges(&self) -> BridgeChoice {
        self.bridges.borrow().clone()
    }

    /// The programs this installation would run, for telling which bridges can work.
    pub fn programs() -> Option<TorPrograms> {
        TorPrograms::locate(&std::env::current_exe().ok()?, Self::is_program)
    }

    /// Saves `bridges` and restarts Tor with them.
    pub fn use_bridges(self: &Rc<Self>, bridges: BridgeChoice) {
        if let Err(error) = self.settings_file.write_json(&bridges.to_json()) {
            glib::g_warning!(Self::LOG_DOMAIN, "cannot save the Tor connection settings: {error}");
        }
        self.bridges.replace(bridges);
        self.start();
    }

    /// (Re)starts Tor; the observers follow the connection from [`TorState::Starting`].
    pub fn start(self: &Rc<Self>) {
        let generation = self.generation.get() + 1;
        self.generation.set(generation);
        self.set_state(TorState::Starting);
        let connection = Rc::clone(self);
        glib::spawn_future_local(async move {
            if let Err(problem) = connection.run(generation).await
                && connection.generation.get() == generation
            {
                connection.set_state(TorState::Failed(problem));
            }
        });
    }

    /// Asks Tor for new circuits and gives the session new SOCKS credentials, so nothing that follows shares
    /// a circuit with what came before.
    ///
    /// # Errors
    /// Tor refused, or its control port broke.
    pub async fn new_identity(&self) -> Result<(), String> {
        let running = self.running.borrow().as_ref().and_then(|running| running.control.clone().zip(running.socks));
        let Some((control, socks)) = running else { return Ok(()) };
        Self::accepted(control.send(&ControlCommand::NewIdentity).await?)?;
        self.identity.set(self.identity.get() + 1);
        if matches!(*self.state.borrow(), TorState::Connected { .. }) {
            self.set_state(TorState::Connected { proxy_uri: socks.socks_proxy_uri(self.identity.get()) });
        }
        Ok(())
    }

    async fn run(self: &Rc<Self>, generation: u64) -> Result<(), String> {
        self.stop_running().await;
        self.ensure_current(generation)?;
        let programs = Self::programs().ok_or_else(Self::not_installed)?;
        let bridges = self.bridges();
        let configuration = TorConfiguration::new(self.data_directory.clone(), std::process::id(), &bridges, &programs)
            .map_err(|problem| problem.describe(InterfaceMessages::catalogue()))?;
        let torrc = configuration.to_torrc().map_err(|problem| problem.to_string())?;
        StoredFile::new(configuration.torrc_file())
            .write(torrc.as_bytes())
            .map_err(|error| tr!("Cannot write Tor's configuration: {error}", error = error))?;
        StoredFile::new(configuration.control_port_file())
            .remove()
            .map_err(|error| tr!("Cannot remove Tor's old control port file: {error}", error = error))?;
        let process = Self::spawn(&programs, &configuration)?;
        self.last_log.borrow_mut().clear();
        self.follow_log(&process);
        self.watch_exit(process.clone(), generation);
        self.running.replace(Some(RunningTor { process, control: None, socks: None }));
        let control_endpoint = self.wait_for_control_port(&configuration, generation).await?;
        let connection = Rc::downgrade(self);
        let control = Rc::new(
            TorControlPort::connect(control_endpoint, move |event| {
                if let Some(connection) = connection.upgrade() {
                    connection.observe_event(event, generation);
                }
            })
            .await?,
        );
        self.ensure_current(generation)?;
        self.with_running(|running| running.control = Some(Rc::clone(&control)));
        let cookie = std::fs::read(configuration.cookie_file())
            .map_err(|error| tr!("Cannot read Tor's control cookie: {error}", error = error))?;
        let authenticate =
            ControlCommand::authenticate(&cookie).ok_or_else(|| tr!("Tor's control cookie is damaged").to_owned())?;
        for command in [authenticate, ControlCommand::TakeOwnership, ControlCommand::FollowStatus] {
            Self::accepted(control.send(&command).await?)?;
        }
        let listeners = Self::accepted(control.send(&ControlCommand::GetSocksListeners).await?)?;
        let socks = listeners
            .value(ControlCommand::SOCKS_LISTENERS_KEY)
            .and_then(TorEndpoint::from_listeners)
            .ok_or_else(|| tr!("Tor opened no local SOCKS port").to_owned())?;
        self.ensure_current(generation)?;
        self.with_running(|running| running.socks = Some(socks));
        let phase = Self::accepted(control.send(&ControlCommand::GetBootstrapPhase).await?)?;
        if let Some(status) = phase.value(ControlCommand::BOOTSTRAP_PHASE_KEY).and_then(BootstrapStatus::from_report) {
            self.observe_bootstrap(status, generation);
        }
        Ok(())
    }

    fn spawn(programs: &TorPrograms, configuration: &TorConfiguration) -> Result<gio::Subprocess, String> {
        let launcher =
            gio::SubprocessLauncher::new(gio::SubprocessFlags::STDOUT_PIPE | gio::SubprocessFlags::STDERR_MERGE);
        if let Some(libraries) = programs.library_directory() {
            launcher.setenv("LD_LIBRARY_PATH", libraries, true);
        }
        let mut command_line: Vec<OsString> = vec![programs.tor().into()];
        command_line.extend(configuration.command_line());
        let arguments: Vec<&OsStr> = command_line.iter().map(OsString::as_os_str).collect();
        launcher.spawn(&arguments).map_err(|error| tr!("Cannot start Tor: {error}", error = error))
    }

    /// Keeps Tor's last log lines (Tor's own logs scrub addresses) and passes them on as debug messages.
    fn follow_log(self: &Rc<Self>, process: &gio::Subprocess) {
        let Some(output) = process.stdout_pipe() else { return };
        let lines = gio::DataInputStream::new(&output);
        let connection = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            while let Ok(Some(line)) = lines.read_line_utf8_future(glib::Priority::DEFAULT).await {
                glib::g_debug!(Self::LOG_DOMAIN, "tor: {line}");
                let Some(connection) = connection.upgrade() else { return };
                let mut log = connection.last_log.borrow_mut();
                log.push_back(line.to_string());
                if log.len() > Self::KEPT_LOG_LINES {
                    log.pop_front();
                }
            }
        });
    }

    fn watch_exit(self: &Rc<Self>, process: gio::Subprocess, generation: u64) {
        let connection = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let _ = process.wait_future().await;
            let Some(connection) = connection.upgrade() else { return };
            if connection.generation.get() != generation {
                return;
            }
            if let Some(control) = connection.running.take().and_then(|running| running.control) {
                control.close();
            }
            let log = connection.last_log.borrow().iter().cloned().collect::<Vec<String>>().join("\n");
            connection.set_state(TorState::Failed(format!("{}\n{log}", tr!("Tor stopped.")).trim_end().to_owned()));
        });
    }

    async fn wait_for_control_port(
        &self,
        configuration: &TorConfiguration,
        generation: u64,
    ) -> Result<TorEndpoint, String> {
        let file = StoredFile::new(configuration.control_port_file());
        let mut waited = Duration::ZERO;
        while waited < Self::CONTROL_PORT_TIMEOUT {
            self.ensure_current(generation)?;
            if let Some(endpoint) = file.read_text().as_deref().and_then(TorEndpoint::from_control_port_file) {
                return Ok(endpoint);
            }
            glib::timeout_future(Self::CONTROL_PORT_POLL).await;
            waited += Self::CONTROL_PORT_POLL;
        }
        Err(tr!("Tor did not open its control port").to_owned())
    }

    async fn stop_running(&self) {
        let Some(running) = self.running.take() else { return };
        if let Some(control) = &running.control {
            control.close();
        }
        running.process.send_signal(Self::TERMINATE_SIGNAL);
        if glib::future_with_timeout(Self::EXIT_TIMEOUT, running.process.wait_future()).await.is_err() {
            running.process.force_exit();
        }
    }

    fn observe_event(&self, event: &ControlReply, generation: u64) {
        if let Some(status) = event.status_report().and_then(BootstrapStatus::from_report) {
            self.observe_bootstrap(status, generation);
        }
    }

    /// Connected once Tor reports 100 % and the SOCKS port is known; a connected Tor stays connected.
    fn observe_bootstrap(&self, status: BootstrapStatus, generation: u64) {
        if self.generation.get() != generation || matches!(*self.state.borrow(), TorState::Connected { .. }) {
            return;
        }
        let socks = self.running.borrow().as_ref().and_then(|running| running.socks);
        let state = match socks {
            Some(socks) if status.is_complete() => {
                TorState::Connected { proxy_uri: socks.socks_proxy_uri(self.identity.get()) }
            }
            _ => TorState::Connecting(status),
        };
        self.set_state(state);
    }

    fn set_state(&self, state: TorState) {
        if *self.state.borrow() == state {
            return;
        }
        self.state.replace(state);
        let observers = std::mem::take(&mut *self.observers.borrow_mut());
        let state = self.state.borrow().clone();
        let mut kept: Vec<TorObserver> =
            observers.into_iter().filter(|observer| observer(&state).is_continue()).collect();
        let mut observers = self.observers.borrow_mut();
        kept.append(&mut observers);
        *observers = kept;
    }

    fn with_running(&self, change: impl FnOnce(&mut RunningTor)) {
        if let Some(running) = self.running.borrow_mut().as_mut() {
            change(running);
        }
    }

    fn ensure_current(&self, generation: u64) -> Result<(), String> {
        if self.generation.get() == generation { Ok(()) } else { Err(Self::SUPERSEDED.to_owned()) }
    }

    fn accepted(reply: ControlReply) -> Result<ControlReply, String> {
        if reply.is_success() { Ok(reply) } else { Err(tr!("Tor refused: {reason}", reason = reply.message())) }
    }

    fn not_installed() -> String {
        tr!(
            "Tor is not installed. Install your distribution's tor package; the Tor window starts its own Tor and needs no Tor service running."
        )
        .to_owned()
    }

    fn is_program(path: &Path) -> bool {
        std::fs::metadata(path).is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
    }
}
