//! Plain HTTP downloads made by the browser itself (tracker lists, web app manifests and icons).
//!
//! It uses libsoup, the HTTP stack WebKitGTK already ships, so ZTBrowser adds no second TLS implementation. The
//! session has no cookie jar: these requests carry no identity. Every body is read with a size limit, since a
//! hostile server could otherwise stream forever.
//!
//! The Tor window's client sends nothing while Tor is not connected: its requests wait for the route through
//! Tor's SOCKS port, and never fall back to a direct connection. It never sends a validator (`ETag`) either:
//! one the server handed out earlier would let it link the requests that carry it.

use std::cell::RefCell;
use std::error::Error;
use std::fmt;

use futures_channel::oneshot;

use gio::prelude::InputStreamExt;
use soup::prelude::SessionExt;

pub enum Fetched {
    /// The server confirmed the copy matching the given `ETag` is still current.
    NotModified,
    Body {
        bytes: Vec<u8>,
        etag: Option<String>,
    },
}

#[derive(Debug)]
pub enum FetchError {
    Network(glib::Error),
    Status(soup::Status, String),
    TooLarge(String),
    InvalidAddress(String),
}

impl fmt::Display for FetchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(error) => write!(formatter, "{error}"),
            Self::Status(status, url) => {
                formatter.write_str(&tr!("{url} answered {status}", url = url, status = format!("{status:?}")))
            }
            Self::TooLarge(url) => formatter.write_str(&tr!("{url} is larger than allowed", url = url)),
            Self::InvalidAddress(url) => formatter.write_str(&tr!("{url} is not a valid address", url = url)),
        }
    }
}

impl Error for FetchError {}

enum Route {
    Direct,
    /// Requests waiting for Tor to connect.
    AwaitingTor(Vec<oneshot::Sender<()>>),
    ThroughTor,
}

pub struct HttpClient {
    session: soup::Session,
    route: RefCell<Route>,
}

impl HttpClient {
    const TIMEOUT_SECONDS: u32 = 30;
    const CHUNK_BYTES: usize = 64 * 1024;

    pub fn new() -> Self {
        Self::routed(Route::Direct)
    }

    /// A client whose requests wait until [`Self::route_through_tor`] gives them Tor's SOCKS port.
    pub fn awaiting_tor() -> Self {
        Self::routed(Route::AwaitingTor(Vec::new()))
    }

    /// Sends every request through `proxy_uri` (Tor's SOCKS port) from now on, and releases the waiting ones.
    pub fn route_through_tor(&self, proxy_uri: &str) {
        self.session.set_proxy_resolver(Some(&gio::SimpleProxyResolver::new(Some(proxy_uri), glib::StrV::new())));
        if let Route::AwaitingTor(waiting) = self.route.replace(Route::ThroughTor) {
            for request in waiting {
                let _ = request.send(());
            }
        }
    }

    /// Holds new requests again until the next [`Self::route_through_tor`]: Tor stopped or restarts.
    pub fn await_tor(&self) {
        let mut route = self.route.borrow_mut();
        if matches!(*route, Route::ThroughTor) {
            *route = Route::AwaitingTor(Vec::new());
        }
    }

    /// `GET url`, revalidating with `etag` when given (never through Tor); the body may be at most
    /// `max_bytes` long.
    pub async fn get(&self, url: &str, max_bytes: usize, etag: Option<&str>) -> Result<Fetched, FetchError> {
        self.wait_for_route().await;
        let etag = etag.filter(|_| matches!(*self.route.borrow(), Route::Direct));
        let message = soup::Message::new("GET", url).map_err(|_| FetchError::InvalidAddress(url.to_owned()))?;
        if let (Some(etag), Some(headers)) = (etag, message.request_headers()) {
            headers.append("If-None-Match", etag);
        }
        let stream = self.session.send_future(&message, glib::Priority::DEFAULT).await.map_err(FetchError::Network)?;
        match message.status() {
            soup::Status::NotModified if etag.is_some() => Ok(Fetched::NotModified),
            soup::Status::Ok => {
                let bytes = Self::read_limited(&stream, url, max_bytes).await?;
                let etag = message.response_headers().and_then(|headers| headers.one("ETag")).map(String::from);
                Ok(Fetched::Body { bytes, etag })
            }
            status => Err(FetchError::Status(status, url.to_owned())),
        }
    }

    fn routed(route: Route) -> Self {
        let session = soup::Session::new();
        session.set_timeout(Self::TIMEOUT_SECONDS);
        Self { session, route: RefCell::new(route) }
    }

    async fn wait_for_route(&self) {
        let waiting = match &mut *self.route.borrow_mut() {
            Route::AwaitingTor(waiting) => {
                let (released, release) = oneshot::channel();
                waiting.push(released);
                release
            }
            Route::Direct | Route::ThroughTor => return,
        };
        let _ = waiting.await;
    }

    async fn read_limited(stream: &gio::InputStream, url: &str, max_bytes: usize) -> Result<Vec<u8>, FetchError> {
        let mut body = Vec::new();
        loop {
            let chunk = stream
                .read_bytes_future(Self::CHUNK_BYTES, glib::Priority::DEFAULT)
                .await
                .map_err(FetchError::Network)?;
            if chunk.is_empty() {
                return Ok(body);
            }
            if body.len() + chunk.len() > max_bytes {
                return Err(FetchError::TooLarge(url.to_owned()));
            }
            body.extend_from_slice(&chunk);
        }
    }
}
