//! Terminal UI for Paraclete — HTTP API only (same contract as `paraclete` CLI).

#![forbid(unsafe_code)]

mod app;
mod run;

pub use app::{
    App, AppEvent, ConnectField, DiffField, ProjectionTab, RunsField, Screen, SubmitFocus,
    TokenField,
};

/// Entry: raw terminal, event loop, HTTP only via [`paraclete_cli::ApiClient`].
pub async fn run() -> Result<(), RunError> {
    run::run_async().await
}

/// Errors leaving [`run()`] (terminal restore, IO).
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("{0}")]
    Msg(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl From<String> for RunError {
    fn from(s: String) -> Self {
        RunError::Msg(s)
    }
}
