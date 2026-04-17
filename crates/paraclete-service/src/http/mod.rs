//! HTTP server wiring (Axum). Handlers delegate to [`crate::service::ParacleteService`].

pub mod handlers;

use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use tower_http::compression::CompressionLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

use crate::service::ParacleteService;
use crate::worker::spawn_scan_job_worker;

#[derive(Clone)]
pub struct AppState {
    pub service: ParacleteService,
}

/// Builds the full `/api/v1` router with compression and request tracing.
///
/// Starts an in-process background worker that drains the `scan_jobs` queue using the same
/// [`ParacleteService::execute_scan_and_persist`] path as synchronous scans.
pub fn build_router(service: ParacleteService) -> Router {
    std::mem::drop(spawn_scan_job_worker(service.clone()));
    let state = AppState { service };
    Router::new()
        .route("/api/v1/health", get(handlers::health))
        .route("/api/v1/openapi.json", get(openapi_json))
        .route("/api/v1/scans", axum::routing::post(handlers::post_async_scan))
        .route("/api/v1/scans/sync", axum::routing::post(handlers::post_scan_sync))
        .route("/api/v1/jobs/scans", axum::routing::post(handlers::post_async_scan))
        .route("/api/v1/jobs/{job_id}", get(handlers::get_job))
        .route("/api/v1/jobs", get(handlers::list_jobs))
        .route("/api/v1/runs/{run_id}", get(handlers::get_run))
        .route("/api/v1/runs/{run_id}/report", get(handlers::get_report))
        .route("/api/v1/runs/{run_id}/assets", get(handlers::get_assets))
        .route("/api/v1/runs/{run_id}/findings", get(handlers::get_findings))
        .route("/api/v1/targets/{target_kind}/runs", get(handlers::list_target_runs))
        .route("/api/v1/diff", get(handlers::get_diff))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            std::time::Duration::from_secs(300),
        ))
        .with_state(state)
}

async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(crate::openapi::openapi_spec())
}
