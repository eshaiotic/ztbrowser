//! ZTBrowser's HTML test report: a run-history dashboard built from what cargo-nextest and cargo-llvm-cov
//! write after a test run (`./run-tests`). The layout follows the house test report template: KPI strip,
//! suite and coverage cards with the last 20 runs, the latest run, failure rankings, features and slowest
//! tests. Labels are Polish, like the template.

pub mod coverage_summary;
pub mod failure_message;
pub mod feature_summary;
pub mod junit_results;
pub mod milliseconds;
pub mod percent;
pub mod recorded_run;
pub mod report_command;
pub mod report_page;
pub mod report_verdict;
pub mod report_widgets;
pub mod run_history;
pub mod run_timestamp;
pub mod run_verdict;
pub mod status_counts;
pub mod suite_summary;
pub mod test_case;
pub mod test_run_report;
pub mod test_status;
