//! Stable JSON API errors (`error.code`, `error.message`, `error.details`).

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use paraclete_core::CoreError;
use paraclete_store::StoreError;
use paraclete_types::ValidationError;
use serde::Serialize;

/// Wire-level error codes (stable strings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidRequest,
    RunNotFound,
    JobNotFound,
    TargetNotFound,
    ScanFailed,
    StoreError,
    InternalError,
}

#[derive(Debug, Serialize)]
pub struct ErrorEnvelope {
    pub code: ErrorCode,
    pub message: String,
    #[serde(default)]
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: ErrorEnvelope,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    InvalidRequest(String),
    #[error("run not found")]
    RunNotFound,
    #[error("job not found")]
    JobNotFound,
    #[error("target not found")]
    TargetNotFound(String),
    #[error("scan failed: {0}")]
    ScanFailed(String),
    #[error("store error: {0}")]
    Store(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> ErrorCode {
        match self {
            AppError::InvalidRequest(_) => ErrorCode::InvalidRequest,
            AppError::RunNotFound => ErrorCode::RunNotFound,
            AppError::JobNotFound => ErrorCode::JobNotFound,
            AppError::TargetNotFound(_) => ErrorCode::TargetNotFound,
            AppError::ScanFailed(_) => ErrorCode::ScanFailed,
            AppError::Store(_) => ErrorCode::StoreError,
            AppError::Internal(_) => ErrorCode::InternalError,
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            AppError::InvalidRequest(_) => StatusCode::BAD_REQUEST,
            AppError::RunNotFound => StatusCode::NOT_FOUND,
            AppError::JobNotFound => StatusCode::NOT_FOUND,
            AppError::TargetNotFound(_) => StatusCode::NOT_FOUND,
            AppError::ScanFailed(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Store(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = ErrorBody {
            error: ErrorEnvelope {
                code: self.code(),
                message: self.to_string(),
                details: serde_json::Value::Object(Default::default()),
            },
        };
        (status, Json(body)).into_response()
    }
}

impl From<StoreError> for AppError {
    fn from(e: StoreError) -> Self {
        match e {
            StoreError::RunNotFound(_) => AppError::RunNotFound,
            StoreError::JobNotFound(_) => AppError::JobNotFound,
            other => AppError::Store(other.to_string()),
        }
    }
}

impl From<CoreError> for AppError {
    fn from(e: CoreError) -> Self {
        match &e {
            CoreError::MissingPath(_) => AppError::TargetNotFound(e.to_string()),
            CoreError::Unsupported(_) => AppError::InvalidRequest(e.to_string()),
            CoreError::NonUtf8Path(_) => AppError::InvalidRequest(e.to_string()),
            CoreError::ReportValidation(msg) => AppError::InvalidRequest(msg.clone()),
            _ => AppError::ScanFailed(e.to_string()),
        }
    }
}

impl From<ValidationError> for AppError {
    fn from(e: ValidationError) -> Self {
        AppError::InvalidRequest(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::InvalidRequest(e.to_string())
    }
}
