//! Measures scrolling smoothness of whichever browser opens the served page; see `benchmark_command`.

use std::process::ExitCode;

use ztbrowser_bench::benchmark_command::BenchmarkCommand;

fn main() -> ExitCode {
    match BenchmarkCommand::from_arguments(std::env::args().skip(1)).and_then(|command| command.run()) {
        Ok(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("ztbrowser-bench: {message}");
            ExitCode::FAILURE
        }
    }
}
