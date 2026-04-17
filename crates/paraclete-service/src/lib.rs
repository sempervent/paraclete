//! Application service layer and HTTP API for Paraclete (Phase 5–6).
//!
//! HTTP handlers must not call [`paraclete_core::ScanEngine`] or [`paraclete_store::SqliteScanStore`]
//! directly — use [`ParacleteService`].

#![forbid(unsafe_code)]

pub mod api_types;
pub mod error;
pub mod http;
pub mod openapi;
pub mod service;
mod worker;

pub use error::{AppError, ErrorBody, ErrorCode};
pub use http::build_router;
pub use openapi::openapi_spec;
pub use service::ParacleteService;
