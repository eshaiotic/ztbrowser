//! One process per identity (the browser, a profile, an installed web app).
//!
//! Chromium's profile files cannot be opened by two processes, so a second launch (a link clicked in a web app
//! that belongs in the browser, `ztbrowser-qt https://…` while the browser runs, the application menu entry of an
//! app that is already open) hands its addresses to the running process through a Unix socket in the runtime
//! directory and exits. The socket is reachable only by the person (the directory is `0700`) and only web
//! addresses are accepted from it.

use std::collections::VecDeque;
use std::fs;
use std::fs::DirBuilder;
use std::io;
use std::io::Read;
use std::io::Write;
use std::net::Shutdown;
use std::os::unix::fs::DirBuilderExt;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use url::Url;

/// Most bytes one request may carry; a longer one is cut off.
const REQUEST_LIMIT: u64 = 64 * 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

pub enum Instance {
    /// This process owns the identity and keeps listening for addresses from later launches.
    Primary(PrimaryInstance),
    /// Another process owns it; the addresses were handed over and this process should exit.
    Forwarded,
}

pub struct PrimaryInstance {
    forwarded: Arc<Mutex<VecDeque<Vec<String>>>>,
    socket: PathBuf,
}

impl PrimaryInstance {
    /// The requests later launches made since the last call, oldest first: the web addresses of each, empty
    /// when the launch only asked the window to come forward.
    #[must_use]
    pub fn take_forwarded(&self) -> Vec<Vec<String>> {
        self.forwarded.lock().unwrap_or_else(std::sync::PoisonError::into_inner).drain(..).collect()
    }
}

impl Drop for PrimaryInstance {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.socket);
    }
}

pub struct SingleInstance;

impl SingleInstance {
    /// Becomes the owner of `identity`, or hands `addresses` to the process that already is.
    ///
    /// # Errors
    /// The runtime directory or the socket cannot be created.
    pub fn acquire(runtime_directory: &Path, identity: &str, addresses: &[String]) -> io::Result<Instance> {
        DirBuilder::new().recursive(true).mode(0o700).create(runtime_directory)?;
        if Self::forward(runtime_directory, identity, addresses)? {
            return Ok(Instance::Forwarded);
        }
        let socket = runtime_directory.join(Self::socket_name(identity));
        // Nothing answered: the file is what a crashed process left behind.
        let _ = fs::remove_file(&socket);
        let listener = UnixListener::bind(&socket)?;
        let forwarded = Arc::new(Mutex::new(VecDeque::new()));
        let inbox = Arc::clone(&forwarded);
        thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                Self::receive(stream, &inbox);
            }
        });
        Ok(Instance::Primary(PrimaryInstance { forwarded, socket }))
    }

    /// Hands `addresses` to the process that owns `identity`; `false` when no process does.
    ///
    /// # Errors
    /// The owner answered, but the addresses could not be sent.
    pub fn forward(runtime_directory: &Path, identity: &str, addresses: &[String]) -> io::Result<bool> {
        let Ok(mut stream) = UnixStream::connect(runtime_directory.join(Self::socket_name(identity))) else {
            return Ok(false);
        };
        stream.write_all(addresses.join("\n").as_bytes())?;
        stream.shutdown(Shutdown::Write)?;
        Ok(true)
    }

    /// Whether any process owns an identity in `runtime_directory`. Each owner is probed as by [`Self::is_running`],
    /// so each brings its window forward; a socket a crashed process left behind does not answer.
    #[must_use]
    pub fn any_running(runtime_directory: &Path) -> bool {
        fs::read_dir(runtime_directory)
            .into_iter()
            .flatten()
            .flatten()
            .any(|entry| UnixStream::connect(entry.path()).is_ok())
    }

    /// Whether a process owns `identity` right now. The probe connects and says nothing, which the owner treats as
    /// a request to bring its window forward.
    #[must_use]
    pub fn is_running(runtime_directory: &Path, identity: &str) -> bool {
        UnixStream::connect(runtime_directory.join(Self::socket_name(identity))).is_ok()
    }

    fn receive(stream: UnixStream, inbox: &Mutex<VecDeque<Vec<String>>>) {
        let _ = stream.set_read_timeout(Some(REQUEST_TIMEOUT));
        let mut text = String::new();
        if stream.take(REQUEST_LIMIT).read_to_string(&mut text).is_err() {
            return;
        }
        let addresses = text
            .lines()
            .filter(|line| Url::parse(line).is_ok_and(|url| matches!(url.scheme(), "http" | "https")))
            .map(str::to_owned)
            .collect();
        inbox.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push_back(addresses);
    }

    /// A short fixed-length name: a socket path may be at most about a hundred bytes long, and an identity is
    /// not guaranteed to fit. FNV-1a, so that every version of the program derives the same name.
    fn socket_name(identity: &str) -> String {
        let hash = identity
            .bytes()
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3));
        format!("{hash:016x}.sock")
    }
}
