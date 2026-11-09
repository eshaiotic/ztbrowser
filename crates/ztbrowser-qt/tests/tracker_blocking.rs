//! The real application, started headless, protects the page it shows.
//!
//! Network protection: the test serves a page from `first.test` that embeds one image from `tracker.test` (listed
//! as a blocked tracker in a fixture data set) and one from `other.test`. Every name is answered by the local
//! server, which records which ones were requested.
//!
//! Page scripts: a page served as `duckduckgo.com` carries DuckDuckGo's offer of its browser extension, which
//! ZTBrowser hides (it has the protections built in and runs no extensions). The page reports whether the offer
//! is hidden by requesting an image from `hidden.test` or `shown.test`.
//!
//! Cookie notices: a page served as `notice.test` carries a consent dialog that the Termly rule of DuckDuckGo's
//! autoconsent library knows. The dialog reports which button was pressed (`rejected.test` or `accepted.test`),
//! or, when nobody pressed any, `ignored.test` after a few seconds.

use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::time::Instant;

use serde_json::json;
use tempfile::TempDir;

// The browser's Qt bridge lives in this library; the test starts the binary, but links it too.
use ztbrowser_qt as _;

/// Host names of the requests the local server received, in order.
type RequestedHosts = Arc<Mutex<Vec<String>>>;

struct BrowsingScenario {
    port: u16,
    requested: RequestedHosts,
    /// Removed when the scenario ends; one per scenario, because scenarios of one test process run side by side.
    home: TempDir,
}

impl BrowsingScenario {
    /// Given: `tracker.test` is a tracker the data set blocks by default, and a local server, used as the
    /// browser's upstream HTTP proxy, answers for every `.test` name.
    fn given_a_tracker_listed_in_the_downloaded_data() -> Self {
        let home = TempDir::new().expect("scratch directory");
        Self::write_protection_cache(&home.path().join("cache/ztbrowser/protections"));
        let listener = TcpListener::bind("127.0.0.1:0").expect("listen");
        let port = listener.local_addr().expect("address").port();
        let requested: RequestedHosts = Arc::default();
        let recorder = Arc::clone(&requested);
        thread::spawn(move || Self::serve(&listener, &recorder));
        Self { port, requested, home }
    }

    fn write_protection_cache(directory: &Path) {
        fs::create_dir_all(directory).expect("cache directory");
        let tracker_data = json!({
            "trackers": {"tracker.test": {"domain": "tracker.test", "owner": {"name": "Tracker Inc"}, "default": "block", "rules": []}},
            "entities": {"Tracker Inc": {"domains": ["tracker.test"]}},
            "cnames": {}
        });
        // The local server speaks plain HTTP, so the fixture exempts the test names from the HTTPS upgrade.
        let exceptions: Vec<_> = [
            "first.test",
            "other.test",
            "tracker.test",
            "duckduckgo.com",
            "hidden.test",
            "shown.test",
            "notice.test",
            "rejected.test",
            "accepted.test",
            "ignored.test",
        ]
        .iter()
        .map(|domain| json!({"domain": domain}))
        .collect();
        let configuration = json!({"features": {"https": {"state": "enabled", "exceptions": exceptions}}});
        fs::write(directory.join("tracker-data.json"), tracker_data.to_string()).expect("tracker data");
        fs::write(directory.join("configuration.json"), configuration.to_string()).expect("configuration");
    }

    fn serve(listener: &TcpListener, requested: &RequestedHosts) {
        // One thread per connection: Chromium opens speculative connections that never send a request.
        for stream in listener.incoming().flatten() {
            let requested = Arc::clone(requested);
            thread::spawn(move || Self::answer(stream, &requested));
        }
    }

    fn answer(mut stream: TcpStream, requested: &RequestedHosts) {
        let (path, host) = Self::read_request(&stream);
        // Chromium opens speculative connections that never send a request.
        if path.is_empty() {
            return;
        }
        if path != "/favicon.ico" {
            requested.lock().expect("lock").push(host.clone());
        }
        let port = stream.local_addr().map_or(0, |address| address.port());
        let page = match (path.as_str(), host.as_str()) {
            ("/", "duckduckgo.com") => Self::extension_offer_page(port),
            ("/", "notice.test") => Self::cookie_notice_page(port),
            ("/", _) => format!(
                "<html><body>hi<img src=\"http://tracker.test:{port}/pixel.gif\"><img src=\"http://other.test:{port}/pixel.gif\"></body></html>"
            ),
            _ => "GIF89a".to_owned(),
        };
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            page.len()
        );
        let _ = stream.write_all(head.as_bytes());
        let _ = stream.write_all(page.as_bytes());
    }

    /// The offer as DuckDuckGo marks it up, and a script that reports, after the page had time to be styled,
    /// whether the offer is displayed.
    fn extension_offer_page(port: u16) -> String {
        const PAGE: &str = "<html><body><a id=\"offer\" data-testid=\"cta-extension-button\" href=\"#\">Add Extension</a>\
            <script>setTimeout(function () {\
            var shown = getComputedStyle(document.getElementById('offer')).display !== 'none';\
            new Image().src = 'http://' + (shown ? 'shown' : 'hidden') + '.test:PORT/report.gif';\
            }, 1500)</script></body></html>";
        PAGE.replace("PORT", &port.to_string())
    }

    /// A consent dialog marked up as Termly's is, whose buttons report which one was pressed.
    fn cookie_notice_page(port: u16) -> String {
        const PAGE: &str = "<html><body><div id=\"termly-code-snippet-support\"><div>We use cookies\
            <button data-tid=\"banner-accept\" onclick=\"report('accepted')\">Accept</button>\
            <button data-tid=\"banner-decline\" onclick=\"report('rejected')\">Decline</button></div></div>\
            <script>var pressed = false;\
            function report(name) { pressed = true; new Image().src = 'http://' + name + '.test:PORT/report.gif'; }\
            setTimeout(function () { if (!pressed) report('ignored'); }, 7000)</script></body></html>";
        PAGE.replace("PORT", &port.to_string())
    }

    /// Returns the request path and the `Host` header's name (without port). A proxied request names its target
    /// as an absolute URL (`http://first.test:1234/x`); only the path part is returned.
    fn read_request(stream: &TcpStream) -> (String, String) {
        let mut reader = BufReader::new(stream);
        let mut host = String::new();
        let mut path = String::new();
        let mut line = String::new();
        while reader.read_line(&mut line).is_ok_and(|bytes| bytes > 0) {
            let header = line.trim_end().to_ascii_lowercase();
            if header.is_empty() {
                break;
            }
            if path.is_empty() {
                path = Self::path_of(header.split_whitespace().nth(1).unwrap_or_default());
            } else if let Some(value) = header.strip_prefix("host:") {
                value.trim().split(':').next().unwrap_or_default().clone_into(&mut host);
            }
            line.clear();
        }
        (path, host)
    }

    /// The path of a request target, whether it is origin-form (`/x`) or an absolute URL (`http://host:1/x`).
    fn path_of(target: &str) -> String {
        match target.strip_prefix("http://") {
            Some(rest) => rest.find('/').map_or("/", |slash| &rest[slash..]).to_owned(),
            None => target.to_owned(),
        }
    }

    /// Given: the person switched the shield off for `host`.
    fn given_protection_switched_off_for(self, host: &str) -> Self {
        let profile = self.home.path().join("data/ztbrowser/profiles/default");
        fs::create_dir_all(&profile).expect("profile directory");
        fs::write(profile.join("unprotected-sites.json"), json!({"domains": [host]}).to_string())
            .expect("unprotected sites");
        self
    }

    fn when_the_page_is_opened_in_the_browser(&self, host: &str) -> BrowserProcess {
        let home = self.home.path();
        let child = Command::new(env!("CARGO_BIN_EXE_ztbrowser-qt"))
            .arg(format!("http://{host}:{}/", self.port))
            .env("QT_QPA_PLATFORM", "offscreen")
            .env("XDG_DATA_HOME", home.join("data"))
            .env("XDG_CACHE_HOME", home.join("cache"))
            // Without a runtime directory the single-instance socket lives in the scratch cache, so the browsers of
            // parallel scenarios (and the person's own running browser) never hand their addresses to one another.
            .env_remove("XDG_RUNTIME_DIR")
            .env("http_proxy", format!("http://127.0.0.1:{}", self.port))
            .env_remove("https_proxy")
            .env_remove("no_proxy")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("start the browser");
        BrowserProcess { child }
    }

    /// The hosts requested once one of `awaited` has been (or after a minute), plus a moment for stragglers.
    fn requested_hosts_after_any_of(&self, awaited: &[&str]) -> Vec<String> {
        let deadline = Instant::now() + Duration::from_secs(60);
        while Instant::now() < deadline
            && !self.requested.lock().expect("lock").iter().any(|host| awaited.contains(&host.as_str()))
        {
            thread::sleep(Duration::from_millis(100));
        }
        thread::sleep(Duration::from_secs(2));
        self.requested.lock().expect("lock").clone()
    }
}

/// The running browser, stopped when the scenario ends.
struct BrowserProcess {
    child: Child,
}

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn should_load_the_page_and_the_ordinary_image_but_not_the_tracker_image() {
    let scenario = BrowsingScenario::given_a_tracker_listed_in_the_downloaded_data();
    let _browser = scenario.when_the_page_is_opened_in_the_browser("first.test");
    let hosts = scenario.requested_hosts_after_any_of(&["other.test"]);
    assert!(hosts.contains(&"first.test".to_owned()), "the page itself should load: {hosts:?}");
    assert!(hosts.contains(&"other.test".to_owned()), "an unlisted third party should load: {hosts:?}");
    assert!(!hosts.contains(&"tracker.test".to_owned()), "the tracker should be blocked: {hosts:?}");
}

#[test]
fn should_hide_duckduckgos_offer_of_a_browser_extension_because_the_protections_are_built_in() {
    let scenario = BrowsingScenario::given_a_tracker_listed_in_the_downloaded_data();
    let _browser = scenario.when_the_page_is_opened_in_the_browser("duckduckgo.com");
    let hosts = scenario.requested_hosts_after_any_of(&["hidden.test", "shown.test"]);
    assert!(hosts.contains(&"hidden.test".to_owned()), "the extension offer should be hidden: {hosts:?}");
}

#[test]
fn should_reject_the_cookie_notice_of_a_protected_site_without_anyone_pressing_a_button() {
    let scenario = BrowsingScenario::given_a_tracker_listed_in_the_downloaded_data();
    let _browser = scenario.when_the_page_is_opened_in_the_browser("notice.test");
    let hosts = scenario.requested_hosts_after_any_of(&["rejected.test", "accepted.test", "ignored.test"]);
    assert!(hosts.contains(&"rejected.test".to_owned()), "the notice should be rejected: {hosts:?}");
    assert!(!hosts.contains(&"accepted.test".to_owned()), "the notice must never be accepted: {hosts:?}");
}

#[test]
fn should_leave_the_cookie_notice_alone_on_a_site_whose_shield_the_person_switched_off() {
    let scenario = BrowsingScenario::given_a_tracker_listed_in_the_downloaded_data()
        .given_protection_switched_off_for("notice.test");
    let _browser = scenario.when_the_page_is_opened_in_the_browser("notice.test");
    let hosts = scenario.requested_hosts_after_any_of(&["rejected.test", "accepted.test", "ignored.test"]);
    assert!(hosts.contains(&"ignored.test".to_owned()), "nobody should have pressed a button: {hosts:?}");
    assert!(!hosts.contains(&"rejected.test".to_owned()), "the notice should be left alone: {hosts:?}");
}
