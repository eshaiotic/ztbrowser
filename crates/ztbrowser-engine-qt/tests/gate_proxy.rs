//! The local proxy every Chromium connection goes through refuses the hosts the gate refuses.

use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use ztbrowser_core::network::connection_clock::SystemConnectionClock;
use ztbrowser_core::network::connection_gate::ConnectionGate;
use ztbrowser_core::network::network_request::NetworkRequest;
use ztbrowser_core::network::request_decision::RequestDecision;
use ztbrowser_core::network::request_policy::RequestPolicy;
use ztbrowser_engine_qt::gate_proxy::GateProxy;
use ztbrowser_engine_qt::upstream_proxy::UpstreamProxy;

struct BlocksHosts(Vec<&'static str>);

impl RequestPolicy for BlocksHosts {
    fn decide(&self, request: &NetworkRequest) -> RequestDecision {
        if request.host().is_some_and(|host| self.0.contains(&host)) {
            RequestDecision::Block
        } else {
            RequestDecision::Pass
        }
    }
}

/// A server on the loopback interface that serves one connection: it sends back what it receives, or, when it
/// is told to speak HTTP, the first lines of the request it got and a short answer.
struct LoopbackServer {
    address: SocketAddr,
    first_lines: mpsc::Receiver<String>,
}

impl LoopbackServer {
    fn echoing(prelude: &'static [u8]) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let (sender, first_lines) = mpsc::channel();
        thread::spawn(move || {
            let Ok((mut stream, _)) = listener.accept() else { return };
            stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
            if prelude.is_empty() {
                let mut buffer = [0_u8; 256];
                while let Ok(count) = stream.read(&mut buffer) {
                    if count == 0 || stream.write_all(&buffer[..count]).is_err() {
                        break;
                    }
                }
                return;
            }
            let mut head = Vec::new();
            let mut buffer = [0_u8; 1];
            while !head.ends_with(b"\r\n\r\n") && stream.read_exact(&mut buffer).is_ok() {
                head.push(buffer[0]);
            }
            sender.send(String::from_utf8_lossy(&head).into_owned()).ok();
            stream.write_all(prelude).ok();
            let mut buffer = [0_u8; 256];
            while let Ok(count) = stream.read(&mut buffer) {
                if count == 0 || stream.write_all(&buffer[..count]).is_err() {
                    break;
                }
            }
        });
        Self { address, first_lines }
    }

    fn head_received(&self) -> String {
        self.first_lines.recv_timeout(Duration::from_secs(5)).unwrap()
    }
}

struct ProxyScenario {
    gate: Arc<ConnectionGate>,
    proxy: GateProxy,
}

impl ProxyScenario {
    fn given_a_proxy_that_filters(hosts: Vec<&'static str>) -> Self {
        Self::given_a_proxy_behind(hosts, UpstreamProxy::direct())
    }

    fn given_a_proxy_behind(hosts: Vec<&'static str>, upstream: UpstreamProxy) -> Self {
        let gate = Arc::new(ConnectionGate::new(Arc::new(SystemConnectionClock::started_now())));
        gate.install(Arc::new(BlocksHosts(hosts)));
        let proxy = GateProxy::start(Arc::clone(&gate), upstream).unwrap();
        Self { gate, proxy }
    }

    fn client(&self) -> TcpStream {
        let stream = TcpStream::connect(self.proxy.address()).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        stream
    }

    /// Sends `request` and returns the first line of the answer.
    fn answer_line_to(&self, request: &str) -> (TcpStream, String) {
        let mut stream = self.client();
        stream.write_all(request.as_bytes()).unwrap();
        let line = Self::read_line(&mut stream);
        (stream, line)
    }

    fn read_line(stream: &mut TcpStream) -> String {
        let mut line = Vec::new();
        let mut byte = [0_u8; 1];
        while !line.ends_with(b"\r\n") && stream.read_exact(&mut byte).is_ok() {
            line.push(byte[0]);
        }
        String::from_utf8_lossy(&line).trim_end().to_owned()
    }

    fn skip_headers(stream: &mut TcpStream) {
        while !Self::read_line(stream).is_empty() {}
    }
}

fn connect_request(authority: &str) -> String {
    format!("CONNECT {authority} HTTP/1.1\r\nHost: {authority}\r\n\r\n")
}

#[test]
fn should_tunnel_to_a_host_the_gate_admits() {
    let origin = LoopbackServer::echoing(b"");
    let scenario = ProxyScenario::given_a_proxy_that_filters(vec!["ads.example"]);
    let (mut stream, line) = scenario.answer_line_to(&connect_request(&origin.address.to_string()));
    assert!(line.starts_with("HTTP/1.1 200"), "{line}");
    ProxyScenario::skip_headers(&mut stream);
    stream.write_all(b"ping").unwrap();
    let mut echoed = [0_u8; 4];
    stream.read_exact(&mut echoed).unwrap();
    assert_eq!(&echoed, b"ping");
}

#[test]
fn should_refuse_a_tunnel_to_a_host_the_gate_refuses() {
    let scenario = ProxyScenario::given_a_proxy_that_filters(vec!["ads.example"]);
    let (_, line) = scenario.answer_line_to(&connect_request("ads.example:443"));
    assert!(line.starts_with("HTTP/1.1 403"), "{line}");
}

#[test]
fn should_tunnel_to_a_filtered_host_once_a_request_to_it_was_allowed() {
    let origin = LoopbackServer::echoing(b"");
    let scenario = ProxyScenario::given_a_proxy_that_filters(vec!["127.0.0.1"]);
    let (_, refused) = scenario.answer_line_to(&connect_request(&origin.address.to_string()));
    scenario.gate.approve("127.0.0.1");
    let (_, admitted) = scenario.answer_line_to(&connect_request(&origin.address.to_string()));
    assert!(refused.starts_with("HTTP/1.1 403"), "{refused}");
    assert!(admitted.starts_with("HTTP/1.1 200"), "{admitted}");
}

#[test]
fn should_refuse_everything_until_the_gate_has_policies() {
    let gate = Arc::new(ConnectionGate::new(Arc::new(SystemConnectionClock::started_now())));
    let scenario = ProxyScenario { proxy: GateProxy::start(Arc::clone(&gate), UpstreamProxy::direct()).unwrap(), gate };
    let (_, line) = scenario.answer_line_to(&connect_request("news.example:443"));
    assert!(line.starts_with("HTTP/1.1 403"), "{line}");
}

#[test]
fn should_forward_a_plain_http_request_in_the_form_a_server_expects() {
    let origin = LoopbackServer::echoing(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok");
    let scenario = ProxyScenario::given_a_proxy_that_filters(vec![]);
    let address = origin.address;
    let request = format!(
        "GET http://{address}/path?q=1 HTTP/1.1\r\nHost: {address}\r\nProxy-Connection: keep-alive\r\nAccept: */*\r\n\r\n"
    );
    let (mut stream, line) = scenario.answer_line_to(&request);
    let received = origin.head_received();
    assert!(line.starts_with("HTTP/1.1 200"), "{line}");
    assert!(received.starts_with("GET /path?q=1 HTTP/1.1\r\n"), "{received}");
    assert!(received.contains("Connection: close"), "{received}");
    assert!(!received.contains("Proxy-Connection"), "{received}");
    assert!(received.contains("Accept: */*"), "{received}");
    ProxyScenario::skip_headers(&mut stream);
    let mut body = [0_u8; 2];
    stream.read_exact(&mut body).unwrap();
    assert_eq!(&body, b"ok");
}

#[test]
fn should_refuse_a_plain_http_request_to_a_host_the_gate_refuses() {
    let scenario = ProxyScenario::given_a_proxy_that_filters(vec!["ads.example"]);
    let (_, line) = scenario.answer_line_to("GET http://ads.example/pixel.gif HTTP/1.1\r\nHost: ads.example\r\n\r\n");
    assert!(line.starts_with("HTTP/1.1 403"), "{line}");
}

#[test]
fn should_reach_the_host_through_the_proxy_of_the_environment() {
    let upstream = LoopbackServer::echoing(b"HTTP/1.1 200 Connection established\r\n\r\n");
    let environment = upstream.address.to_string();
    let behind = UpstreamProxy::from_variables(move |name| (name == "https_proxy").then(|| environment.clone()));
    let scenario = ProxyScenario::given_a_proxy_behind(vec!["ads.example"], behind);
    let (mut stream, line) = scenario.answer_line_to(&connect_request("news.example:443"));
    assert!(line.starts_with("HTTP/1.1 200"), "{line}");
    ProxyScenario::skip_headers(&mut stream);
    assert!(upstream.head_received().starts_with("CONNECT news.example:443 HTTP/1.1\r\n"));
}

#[test]
fn should_answer_a_request_it_cannot_read_with_a_refusal_and_keep_serving() {
    let origin = LoopbackServer::echoing(b"");
    let scenario = ProxyScenario::given_a_proxy_that_filters(vec![]);
    let (_, line) = scenario.answer_line_to("nonsense\r\n\r\n");
    let (_, next) = scenario.answer_line_to(&connect_request(&origin.address.to_string()));
    assert!(line.starts_with("HTTP/1.1 400"), "{line}");
    assert!(next.starts_with("HTTP/1.1 200"), "{next}");
}
