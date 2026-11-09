//! The Tor window's line to its Tor's control port.
//!
//! Commands go out one line each. Tor answers them in the order they arrived and sends asynchronous events
//! (`650`) in between, so answers are matched to commands first in, first out, and events go to the observer
//! given when connecting. Once the connection breaks, every waiting and every later command fails at once.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use futures_channel::oneshot;
use gio::prelude::DataInputStreamExt;
use gio::prelude::DataInputStreamExtManual;
use gio::prelude::IOStreamExt;
use gio::prelude::OutputStreamExtManual;
use gio::prelude::SocketClientExt;
use ztbrowser_core::tor::control_command::ControlCommand;
use ztbrowser_core::tor::control_reply::ControlReply;
use ztbrowser_core::tor::control_reply::ControlReplyReader;
use ztbrowser_core::tor::tor_endpoint::TorEndpoint;

type AnswerSender = oneshot::Sender<Result<ControlReply, String>>;

/// Commands waiting for their answers; `None` once the connection broke.
type WaitingCommands = Rc<RefCell<Option<VecDeque<AnswerSender>>>>;

pub struct TorControlPort {
    connection: gio::SocketConnection,
    waiting: WaitingCommands,
}

impl TorControlPort {
    fn closed() -> String {
        tr!("Tor closed its control port").to_owned()
    }

    /// # Errors
    /// The control port cannot be reached.
    pub async fn connect(endpoint: TorEndpoint, on_event: impl Fn(&ControlReply) + 'static) -> Result<Self, String> {
        let address = endpoint.socket_address();
        let socket_address = gio::InetSocketAddress::new(&gio::InetAddress::from(address.ip()), address.port());
        let connection = gio::SocketClient::new()
            .connect_future(&socket_address)
            .await
            .map_err(|error| tr!("Cannot reach Tor's control port: {error}", error = error))?;
        let input = gio::DataInputStream::new(&connection.input_stream());
        input.set_newline_type(gio::DataStreamNewlineType::CrLf);
        let waiting: WaitingCommands = Rc::new(RefCell::new(Some(VecDeque::new())));
        glib::spawn_future_local(Self::read_answers(input, Rc::clone(&waiting), on_event));
        Ok(Self { connection, waiting })
    }

    /// Sends `command` and waits for Tor's answer.
    ///
    /// # Errors
    /// The connection broke before Tor answered.
    pub async fn send(&self, command: &ControlCommand) -> Result<ControlReply, String> {
        let (sender, answer) = oneshot::channel();
        self.waiting.borrow_mut().as_mut().ok_or_else(Self::closed)?.push_back(sender);
        self.connection
            .output_stream()
            .write_all_future(command.to_line().into_bytes(), glib::Priority::DEFAULT)
            .await
            .map_err(|(_, error)| tr!("Cannot write to Tor's control port: {error}", error = error))?;
        answer.await.map_err(|_| Self::closed())?
    }

    /// Closes the connection; Tor, owned by this connection, exits.
    pub fn close(&self) {
        let _ = self.connection.close(None::<&gio::Cancellable>);
    }

    async fn read_answers(input: gio::DataInputStream, waiting: WaitingCommands, on_event: impl Fn(&ControlReply)) {
        let mut reader = ControlReplyReader::default();
        let failure = loop {
            match input.read_line_utf8_future(glib::Priority::DEFAULT).await {
                Ok(Some(line)) => match reader.push_line(&line) {
                    Ok(Some(reply)) if reply.is_event() => on_event(&reply),
                    Ok(Some(reply)) => {
                        let sender = waiting.borrow_mut().as_mut().and_then(VecDeque::pop_front);
                        if let Some(sender) = sender {
                            let _ = sender.send(Ok(reply));
                        }
                    }
                    Ok(None) => {}
                    Err(malformed) => break malformed.to_string(),
                },
                Ok(None) => break Self::closed(),
                Err(error) => break tr!("Reading Tor's control port failed: {error}", error = error),
            }
        };
        for sender in waiting.borrow_mut().take().into_iter().flatten() {
            let _ = sender.send(Err(failure.clone()));
        }
    }
}
