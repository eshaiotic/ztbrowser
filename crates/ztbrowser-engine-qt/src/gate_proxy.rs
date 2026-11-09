//! The local proxy that every connection of Chromium goes through, so the connection gate sees all of them.
//!
//! Chromium is started with `--proxy-server` pointing here. A tunnel (`CONNECT host:port`, which is every
//! HTTPS connection, including the ones a `preconnect` hint opens) is refused unless the gate admits the host;
//! a plain `http://` request is checked the same way. What passes is relayed byte for byte: the proxy never
//! reads or changes encrypted traffic, and it keeps no log.

use std::io;
use std::io::Read;
use std::io::Write;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::ToSocketAddrs;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use ztbrowser_core::network::connection_gate::ConnectionGate;

use crate::upstream_proxy::UpstreamProxy;

pub struct GateProxy {
    address: SocketAddr,
}

impl GateProxy {
    /// Listens on a free port of the loopback interface and serves connections on threads of its own.
    ///
    /// # Errors
    /// The listening socket or its thread could not be created.
    pub fn start(gate: Arc<ConnectionGate>, upstream: UpstreamProxy) -> io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0))?;
        let address = listener.local_addr()?;
        let session = Arc::new(Session { gate, upstream });
        thread::Builder::new().name("gate-proxy".to_owned()).spawn(move || {
            for client in listener.incoming().flatten() {
                let session = Arc::clone(&session);
                let _ = thread::Builder::new()
                    .name("gate-connection".to_owned())
                    .stack_size(Session::STACK_BYTES)
                    .spawn(move || {
                        let _ = session.serve(client);
                    });
            }
        })?;
        Ok(Self { address })
    }

    /// Where Chromium finds the proxy.
    #[must_use]
    pub fn address(&self) -> SocketAddr {
        self.address
    }
}

struct Session {
    gate: Arc<ConnectionGate>,
    upstream: UpstreamProxy,
}

/// The first lines of a client's request and the bytes that arrived after them.
struct RequestHead {
    method: String,
    target: String,
    headers: Vec<String>,
    surplus: Vec<u8>,
}

impl Session {
    const STACK_BYTES: usize = 256 * 1024;
    const MAX_HEAD_BYTES: usize = 32 * 1024;
    const HEAD_TIMEOUT: Duration = Duration::from_secs(30);
    const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
    const DEFAULT_HTTP_PORT: u16 = 80;

    fn serve(&self, mut client: TcpStream) -> io::Result<()> {
        client.set_read_timeout(Some(Self::HEAD_TIMEOUT))?;
        let Some(head) = Self::read_head(&mut client)? else { return Self::reply(&mut client, "400 Bad Request") };
        client.set_read_timeout(None)?;
        if head.method.eq_ignore_ascii_case("CONNECT") {
            self.tunnel(client, &head)
        } else {
            self.forward_plain_http(client, &head)
        }
    }

    fn tunnel(&self, mut client: TcpStream, head: &RequestHead) -> io::Result<()> {
        let Some((host, port)) = Self::split_authority(&head.target, None) else {
            return Self::reply(&mut client, "400 Bad Request");
        };
        if !self.gate.admits(&host) {
            return Self::reply(&mut client, "403 Forbidden");
        }
        let mut origin = match self.upstream.for_tunnel(&host) {
            Some(proxy) => match Self::open_tunnel_through(&proxy, &head.target) {
                Ok(stream) => stream,
                Err(_) => return Self::reply(&mut client, "502 Bad Gateway"),
            },
            None => match Self::connect(&host, port) {
                Ok(stream) => stream,
                Err(_) => return Self::reply(&mut client, "502 Bad Gateway"),
            },
        };
        client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n")?;
        origin.write_all(&head.surplus)?;
        Self::relay(client, origin)
    }

    fn forward_plain_http(&self, mut client: TcpStream, head: &RequestHead) -> io::Result<()> {
        let Some(rest) = head.target.strip_prefix("http://") else {
            return Self::reply(&mut client, "400 Bad Request");
        };
        let (authority, path) = rest.find('/').map_or((rest, "/"), |slash| (&rest[..slash], &rest[slash..]));
        let Some((host, port)) = Self::split_authority(authority, Some(Self::DEFAULT_HTTP_PORT)) else {
            return Self::reply(&mut client, "400 Bad Request");
        };
        if !self.gate.admits(&host) {
            return Self::reply(&mut client, "403 Forbidden");
        }
        let (mut origin, request_target) = match self.upstream.for_plain_http(&host) {
            Some(proxy) => (Self::connect_to_address(&proxy), head.target.clone()),
            None => (Self::connect(&host, port), path.to_owned()),
        };
        let Ok(origin) = origin.as_mut() else { return Self::reply(&mut client, "502 Bad Gateway") };
        let mut request = format!("{} {request_target} HTTP/1.1\r\n", head.method);
        for header in head.headers.iter().filter(|header| !Self::is_connection_header(header)) {
            request.push_str(header);
            request.push_str("\r\n");
        }
        request.push_str("Connection: close\r\n\r\n");
        origin.write_all(request.as_bytes())?;
        origin.write_all(&head.surplus)?;
        Self::relay(client, origin.try_clone()?)
    }

    fn reply(client: &mut TcpStream, status: &str) -> io::Result<()> {
        client.write_all(format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").as_bytes())
    }

    /// Reads up to the blank line that ends the head; `None` when the head is not a request line plus headers.
    fn read_head(client: &mut TcpStream) -> io::Result<Option<RequestHead>> {
        let mut received = Vec::new();
        let mut chunk = [0_u8; 4096];
        let end = loop {
            if let Some(position) = received.windows(4).position(|window| window == b"\r\n\r\n") {
                break position;
            }
            if received.len() > Self::MAX_HEAD_BYTES {
                return Ok(None);
            }
            let count = client.read(&mut chunk)?;
            if count == 0 {
                return Ok(None);
            }
            received.extend_from_slice(&chunk[..count]);
        };
        let surplus = received.split_off(end + 4);
        received.truncate(end);
        let Ok(text) = String::from_utf8(received) else { return Ok(None) };
        let mut lines = text.split("\r\n");
        let mut request_line = lines.next().unwrap_or_default().split(' ');
        let (Some(method), Some(target), Some(version), None) =
            (request_line.next(), request_line.next(), request_line.next(), request_line.next())
        else {
            return Ok(None);
        };
        if !version.starts_with("HTTP/") {
            return Ok(None);
        }
        Ok(Some(RequestHead {
            method: method.to_owned(),
            target: target.to_owned(),
            headers: lines.map(str::to_owned).collect(),
            surplus,
        }))
    }

    /// `host:port` or `[v6]:port`, the host without brackets and in lower case.
    fn split_authority(authority: &str, default_port: Option<u16>) -> Option<(String, u16)> {
        let (host, port) = match authority.rsplit_once(':') {
            Some((host, port)) if !host.is_empty() && !port.contains(']') => (host, port.parse().ok()?),
            _ => (authority, default_port?),
        };
        let host = host.trim_start_matches('[').trim_end_matches(']');
        (!host.is_empty()).then(|| (host.to_ascii_lowercase(), port))
    }

    fn is_connection_header(header: &str) -> bool {
        let name = header.split(':').next().unwrap_or_default().trim();
        ["connection", "proxy-connection", "proxy-authorization"].iter().any(|hop| name.eq_ignore_ascii_case(hop))
    }

    fn connect(host: &str, port: u16) -> io::Result<TcpStream> {
        Self::connect_to_address(&format!("{host}:{port}"))
            .or_else(|_| Self::connect_to_address(&format!("[{host}]:{port}")))
    }

    fn connect_to_address(address: &str) -> io::Result<TcpStream> {
        let mut last = io::Error::new(io::ErrorKind::NotFound, "the name has no address");
        for candidate in address.to_socket_addrs()? {
            match TcpStream::connect_timeout(&candidate, Self::CONNECT_TIMEOUT) {
                Ok(stream) => {
                    stream.set_nodelay(true)?;
                    return Ok(stream);
                }
                Err(error) => last = error,
            }
        }
        Err(last)
    }

    /// Asks the proxy at `proxy` for a tunnel to `target` (`host:port`).
    fn open_tunnel_through(proxy: &str, target: &str) -> io::Result<TcpStream> {
        let mut stream = Self::connect_to_address(proxy)?;
        stream.write_all(format!("CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n\r\n").as_bytes())?;
        stream.set_read_timeout(Some(Self::HEAD_TIMEOUT))?;
        let mut answer = Vec::new();
        let mut byte = [0_u8; 1];
        while !answer.ends_with(b"\r\n\r\n") {
            if answer.len() > Self::MAX_HEAD_BYTES || stream.read(&mut byte)? == 0 {
                return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "the proxy closed the connection"));
            }
            answer.push(byte[0]);
        }
        stream.set_read_timeout(None)?;
        let accepted = String::from_utf8_lossy(&answer).split(' ').nth(1).is_some_and(|code| code.starts_with('2'));
        accepted
            .then_some(stream)
            .ok_or_else(|| io::Error::new(io::ErrorKind::ConnectionRefused, "the proxy refused the tunnel"))
    }

    /// Copies both directions until either side closes, then closes the other so neither thread waits forever.
    fn relay(client: TcpStream, origin: TcpStream) -> io::Result<()> {
        let (mut from_client, mut to_origin) = (client.try_clone()?, origin.try_clone()?);
        let upstream = thread::Builder::new().stack_size(Self::STACK_BYTES).spawn(move || {
            let _ = io::copy(&mut from_client, &mut to_origin);
            let _ = to_origin.shutdown(Shutdown::Write);
        })?;
        let (mut from_origin, mut to_client) = (origin, client);
        let _ = io::copy(&mut from_origin, &mut to_client);
        let _ = to_client.shutdown(Shutdown::Write);
        let _ = upstream.join();
        Ok(())
    }
}
