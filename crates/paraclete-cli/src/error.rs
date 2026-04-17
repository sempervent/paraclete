//! CLI and HTTP client errors with stable exit semantics.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{0}")]
    Usage(#[from] clap::Error),
    #[error("HTTP {status}: {body}")]
    Http { status: u16, body: String },
    #[error("API error ({code}): {message}")]
    Api { code: String, message: String },
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON decode: {0}")]
    Decode(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Msg(String),
}

impl CliError {
    /// `2` for usage/clap, `1` for everything else.
    pub fn exit_code(&self) -> u8 {
        match self {
            CliError::Usage(_) => 2,
            _ => 1,
        }
    }
}
