//! A tiny local HTTP server: serves the benchmark page and receives the measured frame gaps.

use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread::sleep;
use std::time::Duration;
use std::time::Instant;

use serde_json::Value;

use crate::benchmark_page::BenchmarkPage;
use crate::frame_statistics::FrameStatistics;

const POLL_INTERVAL: Duration = Duration::from_millis(20);
const REQUEST_READ_TIMEOUT: Duration = Duration::from_secs(5);

pub struct BenchmarkServer {
    listener: TcpListener,
}

/// One HTTP request, reduced to what this server needs.
struct IncomingRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

impl BenchmarkServer {
    /// Listens on `127.0.0.1:port`; port 0 picks a free one (see `port`).
    ///
    /// # Errors
    /// The address is already in use or cannot be bound.
    pub fn bind(port: u16) -> Result<Self, String> {
        let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|error| format!("cannot listen: {error}"))?;
        listener.set_nonblocking(true).map_err(|error| format!("cannot poll the socket: {error}"))?;
        Ok(Self { listener })
    }

    /// # Errors
    /// The listener has no local address (it was closed).
    pub fn port(&self) -> Result<u16, String> {
        self.listener.local_addr().map(|address| address.port()).map_err(|error| error.to_string())
    }

    /// Serves `page` until the page script posts its frame gaps, or `timeout` passes.
    ///
    /// # Errors
    /// Timeout, or the posted body is not a JSON array of numbers.
    pub fn serve_until_result(&self, page: &BenchmarkPage, timeout: Duration) -> Result<FrameStatistics, String> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    if let Some(statistics) = Self::answer(&stream, page)? {
                        return Ok(statistics);
                    }
                }
                Err(_) => sleep(POLL_INTERVAL),
            }
        }
        Err("timed out waiting for the page to report its frame gaps".to_owned())
    }

    fn answer(stream: &TcpStream, page: &BenchmarkPage) -> Result<Option<FrameStatistics>, String> {
        let _ = stream.set_nonblocking(false);
        let _ = stream.set_read_timeout(Some(REQUEST_READ_TIMEOUT));
        let Some(request) = Self::read_request(stream) else { return Ok(None) };
        match (request.method.as_str(), request.path.as_str()) {
            ("GET", "/") => {
                Self::respond(stream, "200 OK", "text/html; charset=utf-8", page.html().as_bytes());
                Ok(None)
            }
            ("POST", "/result") => {
                Self::respond(stream, "204 No Content", "text/plain", b"");
                Self::statistics_from(&request.body).map(Some)
            }
            _ => {
                Self::respond(stream, "204 No Content", "text/plain", b"");
                Ok(None)
            }
        }
    }

    fn read_request(stream: &TcpStream) -> Option<IncomingRequest> {
        let mut reader = BufReader::new(stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line).ok()?;
        let mut parts = request_line.split_whitespace();
        let method = parts.next()?.to_owned();
        let path = parts.next()?.to_owned();
        let mut content_length = 0_usize;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).ok()?;
            let line = line.trim_end();
            if line.is_empty() {
                break;
            }
            if let Some((name, value)) = line.split_once(':')
                && name.eq_ignore_ascii_case("content-length")
            {
                content_length = value.trim().parse().ok()?;
            }
        }
        let mut body = vec![0; content_length];
        reader.read_exact(&mut body).ok()?;
        Some(IncomingRequest { method, path, body })
    }

    fn respond(mut stream: &TcpStream, status: &str, content_type: &str, body: &[u8]) {
        let head = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(head.as_bytes());
        let _ = stream.write_all(body);
    }

    fn statistics_from(body: &[u8]) -> Result<FrameStatistics, String> {
        let document: Value = serde_json::from_slice(body).map_err(|error| format!("unreadable result: {error}"))?;
        let gaps = document.as_array().ok_or("the result is not a JSON array")?;
        let gaps_ms: Option<Vec<f64>> = gaps.iter().map(Value::as_f64).collect();
        gaps_ms
            .map(|gaps_ms| FrameStatistics::from_gaps_ms(&gaps_ms))
            .ok_or_else(|| "a frame gap is not a number".to_owned())
    }
}
