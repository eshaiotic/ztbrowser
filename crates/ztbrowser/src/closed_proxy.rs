//! A SOCKS proxy address that leads nowhere, for the Tor window whenever Tor is not connected.
//!
//! WebKit always needs a proxy for the Tor window: without one it would connect directly. Keeping the last
//! Tor SOCKS port is no good either once that Tor has stopped, because another local program could then
//! listen on the freed port and read the window's requests. This listener holds a loopback port of
//! ZTBrowser's own for as long as the process runs and closes every connection at once, so requests fail
//! immediately and nothing leaves the process.

use gio::prelude::Cast;
use gio::prelude::IOStreamExt;
use gio::prelude::InetSocketAddressExt;
use gio::prelude::SocketListenerExt;
use gio::prelude::SocketServiceExt;

pub struct ClosedProxy {
    _service: gio::SocketService,
    uri: String,
}

impl ClosedProxy {
    /// Starts listening on a free loopback port.
    ///
    /// # Errors
    /// The loopback port cannot be opened.
    pub fn open() -> Result<Self, glib::Error> {
        let service = gio::SocketService::new();
        let loopback = gio::InetSocketAddress::new(&gio::InetAddress::new_loopback(gio::SocketFamily::Ipv4), 0);
        let bound =
            service.add_address(&loopback, gio::SocketType::Stream, gio::SocketProtocol::Tcp, None::<&glib::Object>)?;
        let port = bound
            .downcast_ref::<gio::InetSocketAddress>()
            .map(InetSocketAddressExt::port)
            .filter(|port| *port != 0)
            .ok_or_else(|| glib::Error::new(gio::IOErrorEnum::Failed, "no loopback port for the closed proxy"))?;
        service.connect_incoming(|_, connection, _| {
            let _ = connection.close(gio::Cancellable::NONE);
            true
        });
        Ok(Self { _service: service, uri: format!("socks5://127.0.0.1:{port}") })
    }

    pub fn uri(&self) -> &str {
        &self.uri
    }
}
