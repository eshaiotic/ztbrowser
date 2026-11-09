//! The report as one self-contained HTML page (no network, no server), in the house test report layout:
//! KPI strip, suite and coverage cards with recent runs, the latest run, failure rankings, failed and flaky
//! tests, features and the slowest tests.

use std::fmt::Write;

use crate::coverage_summary::CoverageMetric;
use crate::coverage_summary::CoverageSummary;
use crate::percent::Percent;
use crate::recorded_run::RecordedRun;
use crate::report_widgets::ChartSlice;
use crate::report_widgets::StripBar;
use crate::report_widgets::badge;
use crate::report_widgets::donut;
use crate::report_widgets::escape_html;
use crate::report_widgets::result_bar;
use crate::report_widgets::run_strip;
use crate::report_widgets::sparkline;
use crate::report_widgets::type_tag;
use crate::status_counts::StatusCounts;
use crate::test_case::TestCase;
use crate::test_run_report::TestRunReport;
use crate::test_status::TestStatus;

const STYLES: &str = include_str!("report_page.css");
const COVERAGE_NOTE_MEASURED: &str = "cargo-llvm-cov · ztbrowser-core · ten przebieg";
const COVERAGE_NOTE_MISSING: &str = "brak pomiaru w tym przebiegu (pacman -S cargo-llvm-cov)";

pub struct ReportPage<'report, 'run> {
    report: &'report TestRunReport<'run>,
}

impl<'report, 'run> ReportPage<'report, 'run> {
    pub fn new(report: &'report TestRunReport<'run>) -> Self {
        Self { report }
    }

    /// `test-report-YYYYMMDD-HHmm.html`, in the local time the run started.
    pub fn file_name(&self) -> String {
        format!("test-report-{}.html", self.report.results().started_at().file_stamp())
    }

    pub fn render(&self) -> String {
        let report = self.report;
        let context = report.context();
        let verdict = report.verdict();
        let started_at = report.results().started_at().polish_label();
        let failed =
            Self::case_table("Testy nieudane", &report.cases_with_status(TestStatus::Failed), Some(TestStatus::Failed));
        let flaky = Self::case_table(
            "Testy niestabilne (flaky)",
            &report.cases_with_status(TestStatus::Flaky),
            Some(TestStatus::Flaky),
        );
        format!(
            r##"<!doctype html>
<html lang="pl"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>ZTBrowser — przebiegi testów</title><style>{STYLES}</style></head><body>
<svg width="0" height="0" style="position:absolute"><defs><linearGradient id="spark-fill" x1="0" x2="0" y1="0" y2="1"><stop offset="0" stop-color="#c9d1d9" stop-opacity=".22"/><stop offset="1" stop-color="#c9d1d9" stop-opacity="0"/></linearGradient></defs></svg>
<div class="topbar"><span class="brand">ztbrowser</span><span class="pill"><i></i>{branch} · {commit}</span></div>
<main>
<div class="title"><div><h1>Przegląd przebiegów testów</h1><p class="subtitle">Historia raportów cargo-nextest — jeden pasek na przebieg · zakres {suite} · {started_at}</p></div>{verdict}</div>
{kpis}
<section class="cards">{suite_card}{coverage_card}</section>
{latest}
<div class="grid-two">{failures}{unstable}</div>
{failed}{flaky}
{features}
{slowest}
<p class="muted mono" style="text-align:center;margin-top:28px">FLAKY = nieudana próba i sukces po retry · źródło pokrycia: {note}</p>
</main></body></html>
"##,
            branch = escape_html(&context.branch),
            commit = escape_html(&context.commit),
            suite = TestRunReport::SUITE,
            verdict = badge(verdict.label(), verdict.colour()),
            kpis = self.kpi_strip(),
            suite_card = self.suite_card(),
            coverage_card = self.coverage_card(),
            latest = self.latest_run(),
            failures = Self::ranking("Najczęstsze błędy", &report.frequent_failures()),
            unstable = Self::ranking("Najbardziej niestabilne funkcjonalności", &report.unstable_features()),
            features = self.feature_table(),
            slowest = Self::case_table("Najwolniejsze testy", &report.slowest(), None),
            note = Self::coverage_note(report.coverage()),
        )
    }

    fn kpi_strip(&self) -> String {
        let report = self.report;
        let totals = report.totals();
        let failing_now = totals.of(TestStatus::Failed);
        let lines = report.coverage().and_then(CoverageSummary::lines).map(CoverageMetric::percent);
        let red = TestStatus::Failed.colour();
        let kpis = [
            ("ZESTAWY", u32::from(totals.total() > 0).to_string(), ""),
            ("GLOBALNY PASS RATE", totals.pass_rate().to_string(), ""),
            ("TESTY / OSTATNI", totals.total().to_string(), ""),
            ("PRZEBIEGI", report.history().len().to_string(), ""),
            ("FAILING NOW", failing_now.to_string(), if failing_now > 0 { red } else { "" }),
            (
                "COVERAGE LINES",
                lines.map_or_else(|| "—".to_owned(), |lines| lines.to_string()),
                if lines.is_some_and(|lines| lines.value() <= CoverageSummary::THRESHOLD_PERCENT) { red } else { "" },
            ),
        ];
        let mut strip = String::from(r#"<section class="kpis">"#);
        for (label, value, colour) in kpis {
            let style = if colour.is_empty() { String::new() } else { format!(r#" style="color:{colour}""#) };
            let _ = write!(
                strip,
                r#"<div><div class="kpi-label">{label}</div><div class="kpi-value"{style}>{value}</div></div>"#
            );
        }
        strip.push_str("</section>");
        strip
    }

    fn suite_card(&self) -> String {
        let report = self.report;
        let window = TestRunReport::RUN_WINDOW;
        let title = "Unit · cargo nextest";
        let Some(summary) = report.history().suite_summary(TestRunReport::SUITE, window) else {
            return format!(
                r#"<article class="card"><header><h3>{title}</h3>{}</header><div class="card-label">PASS RATE · OSTATNIE {window}</div><div class="card-value muted">—</div>{}<footer><span>0 przebiegów</span><span>brak wyników</span><span></span></footer></article>"#,
                badge("BRAK", TestStatus::Skipped.colour()),
                run_strip(&[], window),
            );
        };
        let bars: Vec<StripBar> = summary
            .recent()
            .iter()
            .map(|run| StripBar { colour: run.counts().verdict().colour(), title: Self::run_title(run) })
            .collect();
        let rates: Vec<f64> = summary.recent().iter().map(|run| run.counts().pass_rate().value()).collect();
        let latest = summary.latest();
        let verdict = summary.verdict();
        format!(
            r#"<article class="card"><header><h3>{title}</h3>{badge}</header>
<div class="card-body"><div><div class="card-label">PASS RATE · OSTATNIE {window}</div><div class="card-value">{rate}</div></div>{spark}</div>
{strip}
<footer><span>{runs} przebiegów</span><span>{tests} testów · {duration}</span><span>{age}</span></footer></article>"#,
            badge = badge(verdict.label(), verdict.colour()),
            rate = summary.pass_rate(),
            spark = sparkline(&rates),
            strip = run_strip(&bars, window),
            runs = summary.total_runs(),
            tests = latest.counts().total(),
            duration = latest.duration(),
            age = latest.started_at().describe_age(report.context().now_unix_seconds),
        )
    }

    fn coverage_card(&self) -> String {
        let report = self.report;
        let window = TestRunReport::RUN_WINDOW;
        let coverage = report.coverage();
        let is_above = coverage.is_some_and(CoverageSummary::is_above_threshold);
        let (passed, failed) = (TestStatus::Passed.colour(), TestStatus::Failed.colour());
        let card_badge = match coverage {
            Some(_) if is_above => badge("&gt;90%", passed),
            Some(_) => badge("≤90%", failed),
            None => badge("BRAK", TestStatus::Skipped.colour()),
        };
        let runs = report.history().coverage_runs(window);
        let bars: Vec<StripBar> = runs
            .iter()
            .filter_map(|run| run.coverage().map(|measured| (run, measured)))
            .map(|(run, measured)| StripBar {
                colour: if measured.is_above_threshold() { passed } else { failed },
                title: format!(
                    "{} · {} · lines {}",
                    run.started_at().as_str(),
                    run.commit(),
                    measured.lines().map_or(Percent::from_value(0.0), CoverageMetric::percent)
                ),
            })
            .collect();
        let line_rates: Vec<f64> = runs
            .iter()
            .filter_map(|run| run.coverage().and_then(CoverageSummary::lines))
            .map(|lines| lines.percent().value())
            .collect();
        let details = coverage.map_or_else(
            || "L — · F — · R —".to_owned(),
            |coverage| {
                coverage
                    .metrics()
                    .iter()
                    .take(3)
                    .map(|metric| format!("{} {:.2}", &metric.name()[..1], metric.percent().value()))
                    .collect::<Vec<_>>()
                    .join(" · ")
            },
        );
        let value_colour = match coverage {
            Some(_) if is_above => "#e6edf3",
            Some(_) => failed,
            None => "#6e7681",
        };
        let value = coverage
            .and_then(CoverageSummary::lines)
            .map_or_else(|| "—".to_owned(), |lines| lines.percent().to_string());
        let age =
            runs.last().map(|run| run.started_at().describe_age(report.context().now_unix_seconds)).unwrap_or_default();
        format!(
            r#"<article class="card"><header><h3>Coverage · llvm-cov</h3>{card_badge}</header>
<div class="card-body"><div><div class="card-label">LINES · OSTATNIE {window}</div><div class="card-value" style="color:{value_colour}">{value}</div></div>{spark}</div>
{strip}
<footer><span>{details}</span><span title="{note}">próg &gt;90%</span><span>{age}</span></footer></article>"#,
            spark = sparkline(&line_rates),
            strip = run_strip(&bars, window),
            note = escape_html(Self::coverage_note(coverage)),
        )
    }

    fn latest_run(&self) -> String {
        let report = self.report;
        let totals = report.totals();
        let mut counters = String::new();
        for status in TestStatus::ALL {
            let _ = write!(
                counters,
                r#"<div class="counter"><span class="kpi-label">{label}</span><b style="color:{colour}">{count}</b><small>{share}</small></div>"#,
                label = status.label(),
                colour = status.colour(),
                count = totals.of(status),
                share = totals.share_of(status),
            );
        }
        let durations = report.durations();
        let mut timings = String::new();
        for (label, value) in [
            ("ŁĄCZNY CZAS TESTÓW", durations.total),
            ("ŚREDNI CZAS TESTU", durations.average),
            ("NAJDŁUŻSZY TEST", durations.longest),
            ("CZAS PRZEBIEGU", report.results().wall_clock()),
        ] {
            let _ =
                write!(timings, r#"<div class="counter"><span class="kpi-label">{label}</span><b>{value}</b></div>"#);
        }
        format!(
            r#"<section class="panel"><h2>Ostatni przebieg</h2><div class="latest">{tests}{coverage}<div><div class="counters">{counters}</div><div class="counters">{timings}</div>{bar}</div></div></section>"#,
            tests = Self::test_donut(&totals),
            coverage = Self::coverage_donut(report.coverage()),
            bar = Self::status_bar(&totals),
        )
    }

    fn test_donut(totals: &StatusCounts) -> String {
        let slices: Vec<ChartSlice<'_>> = TestStatus::ALL
            .iter()
            .map(|status| ChartSlice { label: status.label(), colour: status.colour(), value: totals.of(*status) })
            .collect();
        let passed = totals.share_of(TestStatus::Passed).to_string();
        donut("TESTY", &slices, totals.total(), &passed, "PASSED")
    }

    fn coverage_donut(coverage: Option<&CoverageSummary>) -> String {
        let lines = coverage.and_then(CoverageSummary::lines);
        let (covered, uncovered, total) =
            lines.map_or((0, 0, 0), |lines| (lines.covered(), lines.uncovered(), lines.count()));
        let slices = [
            ChartSlice { label: "COVERED", colour: TestStatus::Passed.colour(), value: covered },
            ChartSlice { label: "NOT COVERED", colour: TestStatus::Failed.colour(), value: uncovered },
        ];
        let value = lines.map_or_else(|| "—".to_owned(), |lines| lines.percent().to_string());
        donut("COVERAGE · LINES", &slices, total, &value, "COVERED")
    }

    fn status_bar(counts: &StatusCounts) -> String {
        let slices: Vec<ChartSlice<'_>> = TestStatus::ALL
            .iter()
            .map(|status| ChartSlice { label: status.key(), colour: status.colour(), value: counts.of(*status) })
            .collect();
        result_bar(&slices)
    }

    fn feature_table(&self) -> String {
        let features = self.report.features();
        let mut rows = String::new();
        for feature in &features {
            let _ = write!(
                rows,
                r#"<tr><td>{tag}{name}</td><td class="number">{total}</td><td class="number">{pass}</td><td>{bar}</td><td class="number">{duration}</td></tr>"#,
                tag = type_tag(TestRunReport::SUITE),
                name = escape_html(feature.name()),
                total = feature.counts().total(),
                pass = feature.counts().pass_rate(),
                bar = Self::status_bar(feature.counts()),
                duration = feature.duration(),
            );
        }
        if rows.is_empty() {
            rows.push_str(r#"<tr><td colspan="5" class="muted">brak wyników</td></tr>"#);
        }
        format!(
            r#"<section class="panel"><h2>Funkcjonalności <small>{count}</small></h2><div class="scroll"><table><thead><tr><th>Funkcjonalność</th><th class="number">Testy</th><th class="number">Pass</th><th class="wide">Wyniki</th><th class="number">Czas</th></tr></thead><tbody>{rows}</tbody></table></div></section>"#,
            count = features.len(),
        )
    }

    fn ranking(title: &str, entries: &[(String, u32)]) -> String {
        let mut rows = String::new();
        for (label, count) in entries {
            let _ = write!(rows, "<li><span>{}</span><b>{count}</b></li>", escape_html(label));
        }
        if rows.is_empty() {
            rows.push_str(r#"<li class="muted">brak</li>"#);
        }
        format!(r#"<section class="panel"><h2>{title}</h2><ul class="ranking">{rows}</ul></section>"#)
    }

    /// Failed and flaky tables (`highlight` set) show the failure and disappear when empty; the slowest
    /// tests table always shows.
    fn case_table(title: &str, cases: &[&TestCase], highlight: Option<TestStatus>) -> String {
        if cases.is_empty() && highlight.is_some() {
            return String::new();
        }
        let mut rows = String::new();
        for (index, case) in cases.iter().enumerate() {
            let failure = match highlight {
                Some(_) => format!(
                    r#"<div class="error mono">{}</div>"#,
                    case.failure().map_or_else(|| "—".to_owned(), |failure| escape_html(failure.message()))
                ),
                None => String::new(),
            };
            let _ = write!(
                rows,
                r#"<tr><td class="number muted">{number}</td><td>{tag}{title}<div class="muted mono">{place}</div>{failure}</td><td class="number">{duration}</td></tr>"#,
                number = index + 1,
                tag = type_tag(TestRunReport::SUITE),
                title = escape_html(case.title()),
                place = escape_html(case.place()),
                duration = case.duration(),
            );
        }
        if rows.is_empty() {
            rows.push_str(r#"<tr><td class="muted">brak wyników</td></tr>"#);
        }
        let (panel_style, heading_style) = match highlight {
            Some(status) => (
                format!(r#" style="border-color:{}66""#, status.colour()),
                format!(r#" style="color:{}""#, status.colour()),
            ),
            None => (String::new(), String::new()),
        };
        format!(
            r#"<section class="panel"{panel_style}><h2{heading_style}>{title} <small>{count}</small></h2><table><tbody>{rows}</tbody></table></section>"#,
            count = cases.len(),
        )
    }

    fn run_title(run: &RecordedRun) -> String {
        let counts = run.counts();
        format!(
            "{} · {} · {}/{} passed, {} failed, {} flaky",
            run.started_at().as_str(),
            run.commit(),
            counts.of(TestStatus::Passed),
            counts.total(),
            counts.of(TestStatus::Failed),
            counts.of(TestStatus::Flaky),
        )
    }

    fn coverage_note(coverage: Option<&CoverageSummary>) -> &'static str {
        if coverage.is_some() { COVERAGE_NOTE_MEASURED } else { COVERAGE_NOTE_MISSING }
    }
}
