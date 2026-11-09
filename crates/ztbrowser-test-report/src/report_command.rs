//! The `ztbrowser-test-report` command, run from the repository root by `./run-tests`:
//!
//! ```text
//! ztbrowser-test-report --junit target/nextest/report/junit.xml [--coverage test-results/coverage-summary.json]
//! ```
//!
//! It reads the run's results, records the run in `test-results/test-run-history.json` and writes the page
//! to `tests/reports/test-report-YYYYMMDD-HHmm.html`. Both directories are ignored by git.

use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use crate::coverage_summary::CoverageSummary;
use crate::junit_results::JunitResults;
use crate::recorded_run::RecordedRun;
use crate::report_page::ReportPage;
use crate::run_history::RunHistory;
use crate::test_run_report::RunContext;
use crate::test_run_report::TestRunReport;

const USAGE: &str = "użycie: ztbrowser-test-report --junit <junit.xml> [--coverage <coverage-summary.json>]";

pub struct ReportCommand {
    junit: PathBuf,
    coverage: Option<PathBuf>,
    history: PathBuf,
    reports: PathBuf,
}

impl ReportCommand {
    pub const HISTORY_PATH: &'static str = "test-results/test-run-history.json";
    pub const REPORTS_DIRECTORY: &'static str = "tests/reports";

    /// # Errors
    ///
    /// When `--junit` is missing or an option is unknown or lacks its value.
    pub fn from_arguments(arguments: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut junit = None;
        let mut coverage = None;
        let mut arguments = arguments;
        while let Some(option) = arguments.next() {
            let value = arguments.next().map(PathBuf::from);
            match (option.as_str(), value) {
                ("--junit", Some(path)) => junit = Some(path),
                ("--coverage", Some(path)) => coverage = Some(path),
                _ => return Err(USAGE.to_owned()),
            }
        }
        Ok(Self {
            junit: junit.ok_or_else(|| USAGE.to_owned())?,
            coverage,
            history: PathBuf::from(Self::HISTORY_PATH),
            reports: PathBuf::from(Self::REPORTS_DIRECTORY),
        })
    }

    /// Writes the report and returns its path.
    ///
    /// # Errors
    ///
    /// When the JUnit file cannot be read or parsed, or the history or the page cannot be written.
    pub fn run(&self) -> Result<PathBuf, String> {
        let junit = fs::read_to_string(&self.junit)
            .map_err(|error| format!("nie można odczytać {}: {error}", self.junit.display()))?;
        let results = JunitResults::parse(&junit)?;
        let coverage = self.coverage();
        let context = RunContext {
            branch: Self::git(&["branch", "--show-current"]).map_or_else(
                || "nieznana".to_owned(),
                |branch| if branch.is_empty() { "detached".to_owned() } else { branch },
            ),
            commit: Self::git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "nieznany".to_owned()),
            now_unix_seconds: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .and_then(|elapsed| i64::try_from(elapsed.as_secs()).ok())
                .unwrap_or_default(),
        };
        let mut history =
            fs::read_to_string(&self.history).map(|text| RunHistory::from_json(&text)).unwrap_or_default();
        history.record(
            RecordedRun::new(
                TestRunReport::SUITE,
                results.started_at().clone(),
                &context.commit,
                results.wall_clock(),
                results.counts(),
            )
            .with_coverage(coverage.clone()),
        );
        Self::write(&self.history, &history.to_json())?;
        let report = TestRunReport::new(&results, coverage.as_ref(), &history, &context);
        let page = ReportPage::new(&report);
        let path = self.reports.join(page.file_name());
        Self::write(&path, &page.render())?;
        Ok(fs::canonicalize(&path).unwrap_or(path))
    }

    /// The run's coverage, if it was measured. A damaged summary is reported and left out rather than
    /// failing the report.
    fn coverage(&self) -> Option<CoverageSummary> {
        let path = self.coverage.as_ref()?;
        let text = fs::read_to_string(path).ok()?;
        CoverageSummary::from_llvm_cov_export(&text)
            .inspect_err(|error| eprintln!("ztbrowser-test-report: pomijam pokrycie z {}: {error}", path.display()))
            .ok()
    }

    fn git(arguments: &[&str]) -> Option<String> {
        let output = Command::new("git").args(arguments).output().ok()?;
        output.status.success().then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }

    fn write(path: &Path, contents: &str) -> Result<(), String> {
        if let Some(directory) = path.parent() {
            fs::create_dir_all(directory)
                .map_err(|error| format!("nie można utworzyć {}: {error}", directory.display()))?;
        }
        fs::write(path, contents).map_err(|error| format!("nie można zapisać {}: {error}", path.display()))
    }
}
