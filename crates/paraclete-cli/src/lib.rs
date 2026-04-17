//! Thin HTTP client library for the Paraclete CLI (`paraclete` binary).

#![forbid(unsafe_code)]

pub mod api;
pub mod cli;
pub mod client;
mod cmd;
pub mod error;
pub mod render;
pub mod wire;

pub use cli::Cli;
pub use client::ApiClient;
pub use error::CliError;

/// Execute parsed CLI (HTTP calls).
pub async fn run(cli: Cli) -> Result<(), CliError> {
    cmd::run(cli).await
}

/// Run CLI from an argument iterator (tests; `--help` is an error like `try_parse`).
pub async fn run_from_args<I, T>(args: I) -> Result<(), CliError>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = clap::Parser::try_parse_from(args)?;
    run(cli).await
}
