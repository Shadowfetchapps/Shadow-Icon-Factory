mod app;
mod error;
mod generate;
mod paths;
mod settings;
mod spec;
mod ui;

use std::process::ExitCode;

fn main() -> ExitCode {
    if std::env::args().any(|a| a == "--version") {
        println!("shadow-icon-factory {}", paths::APP_VERSION);
        return ExitCode::SUCCESS;
    }
    if let Err(err) = app::run() {
        eprintln!("{}", err.human_message());
        if let Some(tech) = err.technical_details() {
            eprintln!("{tech}");
        }
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
