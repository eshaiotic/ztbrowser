//! The command line of `ztbrowser-bench`.

use std::process::Command;
use std::time::Duration;

use crate::benchmark_page::BenchmarkPage;
use crate::benchmark_server::BenchmarkServer;
use crate::frame_statistics::FrameStatistics;
use crate::scroll_scenario::ScrollScenario;

const USAGE: &str = "usage: ztbrowser-bench --page-url URL [--port N] [--label NAME]\n\
Serves URL with a scroll script injected and waits for one browser to open http://127.0.0.1:PORT/ .\n\
The page is third-party content served from loopback: open it only in a throwaway browser profile.";

#[derive(Debug)]
pub struct BenchmarkCommand {
    page_url: String,
    port: u16,
    label: String,
}

impl BenchmarkCommand {
    /// # Errors
    /// A missing or unknown argument; the message is the usage text.
    pub fn from_arguments(arguments: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut page_url = None;
        let mut port = 8770_u16;
        let mut label = "engine".to_owned();
        let mut arguments = arguments;
        while let Some(argument) = arguments.next() {
            let value = arguments.next().ok_or_else(|| format!("{argument} needs a value\n{USAGE}"))?;
            match argument.as_str() {
                "--page-url" => page_url = Some(value),
                "--port" => port = value.parse().map_err(|_| format!("bad port {value}\n{USAGE}"))?,
                "--label" => label = value,
                other => return Err(format!("unknown argument {other}\n{USAGE}")),
            }
        }
        let page_url = page_url.ok_or_else(|| USAGE.to_owned())?;
        Ok(Self { page_url, port, label })
    }

    /// Fetches the page, waits for the browser's run and returns the one-line result.
    ///
    /// # Errors
    /// The page cannot be fetched, the port is taken, or the browser does not report in time.
    pub fn run(&self) -> Result<String, String> {
        let scenario = ScrollScenario::comparison_default();
        let server = BenchmarkServer::bind(self.port)?;
        let origin = format!("http://127.0.0.1:{}", server.port()?);
        let page = BenchmarkPage::injected_into(&self.fetch_page()?, &self.page_url, &scenario, &origin);
        eprintln!("Open {origin}/ in the browser under test (fullscreen, nothing else running).");
        eprintln!(
            "The page is third-party content served from loopback: use a throwaway browser profile, never a signed-in one."
        );
        let wait = Duration::from_millis(u64::from(scenario.duration_ms())) + Duration::from_secs(90);
        let statistics = server.serve_until_result(&page, wait)?;
        Ok(Self::result_line(&self.label, &statistics))
    }

    fn fetch_page(&self) -> Result<String, String> {
        let output = Command::new("curl")
            .args([
                "--silent",
                "--show-error",
                "--location",
                "--max-time",
                "30",
                "--user-agent",
                "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0 Safari/537.36",
                &self.page_url,
            ])
            .output()
            .map_err(|error| format!("cannot run curl: {error}"))?;
        if !output.status.success() {
            return Err(format!("curl failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    fn result_line(label: &str, statistics: &FrameStatistics) -> String {
        format!(
            "{label}: frames={} fps={:.1} p50={:.1}ms p95={:.1}ms p99={:.1}ms max={:.1}ms over33ms={}",
            statistics.frame_count(),
            statistics.frames_per_second(),
            statistics.percentile_gap_ms(50),
            statistics.percentile_gap_ms(95),
            statistics.percentile_gap_ms(99),
            statistics.longest_gap_ms(),
            statistics.frames_longer_than_ms(33.0),
        )
    }
}
