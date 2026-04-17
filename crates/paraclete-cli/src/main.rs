//! `paraclete` — HTTP client entrypoint.

use std::process::ExitCode;

use clap::Parser;

#[tokio::main]
async fn main() -> ExitCode {
    let cli = paraclete_cli::Cli::parse();
    match paraclete_cli::run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("paraclete: {e}");
            ExitCode::from(e.exit_code())
        }
    }
}
