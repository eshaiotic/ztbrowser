//! Writes ZTBrowser's HTML test report after a cargo-nextest run; see `report_command` for the arguments.

use std::process::ExitCode;

use ztbrowser_test_report::report_command::ReportCommand;

fn main() -> ExitCode {
    match ReportCommand::from_arguments(std::env::args().skip(1)).and_then(|command| command.run()) {
        Ok(path) => {
            println!("Raport HTML: file://{}", path.display());
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("ztbrowser-test-report: {message}");
            ExitCode::FAILURE
        }
    }
}
