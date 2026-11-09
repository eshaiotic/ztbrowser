//! The server hands out the page and turns the posted frame gaps into statistics.

use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use ztbrowser_bench::benchmark_page::BenchmarkPage;
use ztbrowser_bench::benchmark_server::BenchmarkServer;
use ztbrowser_bench::frame_statistics::FrameStatistics;
use ztbrowser_bench::scroll_scenario::ScrollScenario;

struct ServerScenario {
    server: BenchmarkServer,
    page: BenchmarkPage,
}

impl ServerScenario {
    fn given_a_server_for(original_html: &str) -> Self {
        let server = BenchmarkServer::bind(0).expect("free port");
        let origin = format!("http://127.0.0.1:{}", server.port().expect("port"));
        let page = BenchmarkPage::injected_into(
            original_html,
            "https://example.test/",
            &ScrollScenario::comparison_default(),
            &origin,
        );
        Self { server, page }
    }

    fn browser_sends(&self, request: String) -> thread::JoinHandle<String> {
        let port = self.server.port().expect("port");
        thread::spawn(move || {
            let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
            stream.write_all(request.as_bytes()).expect("send");
            let mut response = String::new();
            let _ = stream.read_to_string(&mut response);
            response
        })
    }

    fn when_the_result_is_posted(&self, body: &str) -> Result<FrameStatistics, String> {
        let request = format!("POST /result HTTP/1.1\r\nContent-Length: {}\r\n\r\n{body}", body.len());
        let browser = self.browser_sends(request);
        let outcome = self.server.serve_until_result(&self.page, Duration::from_secs(5));
        browser.join().expect("browser thread");
        outcome
    }
}

#[test]
fn should_turn_the_posted_gaps_into_statistics() {
    let statistics = ServerScenario::given_a_server_for("<head></head>")
        .when_the_result_is_posted("[16.0, 17.0, 50.0]")
        .expect("statistics");
    assert_eq!(statistics.frame_count(), 3);
    assert_eq!(statistics.frames_longer_than_ms(33.0), 1);
}

#[test]
fn should_reject_a_result_that_is_not_an_array_of_numbers() {
    let outcome = ServerScenario::given_a_server_for("<head></head>").when_the_result_is_posted("{\"gaps\": 1}");
    assert!(outcome.is_err());
}

#[test]
fn should_serve_the_page_with_the_scroll_script_inside_the_head() {
    let scenario = ServerScenario::given_a_server_for("<html><head><title>t</title></head><body></body></html>");
    let page_request = scenario.browser_sends("GET / HTTP/1.1\r\n\r\n".to_owned());
    let _ = scenario.server.serve_until_result(&scenario.page, Duration::from_millis(300));
    let response = page_request.join().expect("browser thread");
    assert!(response.contains("<head><base href=\"https://example.test/\"><script>"), "{response}");
}

#[test]
fn should_inject_a_head_when_the_page_has_none() {
    let page = BenchmarkPage::injected_into(
        "<body>hi</body>",
        "https://example.test/",
        &ScrollScenario::comparison_default(),
        "http://127.0.0.1:1",
    );
    assert!(page.html().starts_with("<head><base"));
}
