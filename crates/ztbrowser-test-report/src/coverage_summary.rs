//! Code coverage of a run: how much of `ztbrowser-core` the tests executed, from the JSON summary that
//! `cargo llvm-cov … --json --summary-only` writes (LLVM's coverage export format, `data[0].totals`).
//!
//! LLVM counts lines, functions and regions (a region is a span of code that runs as one piece, roughly a
//! statement or a branch arm). Branch coverage needs a nightly compiler; when it was not measured its count
//! is zero and the metric is left out instead of being shown as 0 %.

use serde_json::Value;
use serde_json::json;

use crate::percent::Percent;

const MEASURES: [(&str, &str); 4] =
    [("lines", "Lines"), ("functions", "Functions"), ("regions", "Regions"), ("branches", "Branches")];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverageMetric {
    key: &'static str,
    name: &'static str,
    covered: u32,
    count: u32,
}

impl CoverageMetric {
    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn covered(&self) -> u32 {
        self.covered
    }

    pub fn uncovered(&self) -> u32 {
        self.count - self.covered
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn percent(&self) -> Percent {
        Percent::of(self.covered, self.count)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CoverageSummary {
    metrics: Vec<CoverageMetric>,
}

impl CoverageSummary {
    /// Every measured metric must be above this share for the report to call coverage sufficient.
    pub const THRESHOLD_PERCENT: f64 = 90.0;

    /// # Errors
    ///
    /// When the text is not an LLVM coverage export with `data[0].totals`.
    pub fn from_llvm_cov_export(text: &str) -> Result<Self, String> {
        let export: Value =
            serde_json::from_str(text).map_err(|error| format!("podsumowanie pokrycia nie jest JSON: {error}"))?;
        export
            .pointer("/data/0/totals")
            .and_then(Self::from_totals)
            .ok_or_else(|| "podsumowanie pokrycia nie ma data[0].totals".to_owned())
    }

    /// Reads the `totals` object of an LLVM export, or the copy kept in the run history.
    pub fn from_totals(totals: &Value) -> Option<Self> {
        let metrics: Vec<CoverageMetric> = MEASURES
            .iter()
            .filter_map(|&(key, name)| {
                let measure = totals.get(key)?;
                let count = u32::try_from(measure.get("count")?.as_u64()?).ok()?;
                let covered = u32::try_from(measure.get("covered")?.as_u64()?).ok()?.min(count);
                (count > 0).then_some(CoverageMetric { key, name, covered, count })
            })
            .collect();
        (!metrics.is_empty()).then_some(Self { metrics })
    }

    pub fn to_totals(&self) -> Value {
        let mut totals = serde_json::Map::new();
        for metric in &self.metrics {
            totals.insert(metric.key.to_owned(), json!({"covered": metric.covered, "count": metric.count}));
        }
        Value::Object(totals)
    }

    pub fn metrics(&self) -> &[CoverageMetric] {
        &self.metrics
    }

    pub fn lines(&self) -> Option<&CoverageMetric> {
        self.metrics.iter().find(|metric| metric.key == "lines")
    }

    pub fn is_above_threshold(&self) -> bool {
        self.metrics.iter().all(|metric| metric.percent().value() > Self::THRESHOLD_PERCENT)
    }
}
