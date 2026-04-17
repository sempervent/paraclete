//! Axum handlers — call [`crate::service::ParacleteService`] only.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use paraclete_types::{JobId, RunId};
use uuid::Uuid;

use crate::api_types::{
    DiffQuery, HealthResponse, JobListQuery, PageQuery, PagedAssetsResponse, PagedFindingsResponse,
    StartScanRequest, TargetRunsQuery,
};
use crate::error::AppError;
use crate::http::AppState;

const KNOWN_TARGET_KINDS: &[&str] =
    &["local_file", "local_directory", "logical_dataset", "object_store_placeholder"];

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

/// Async scan: queue job and return 202 (same handler as `POST /api/v1/jobs/scans`).
pub async fn post_async_scan(
    State(state): State<AppState>,
    Json(body): Json<StartScanRequest>,
) -> Result<(StatusCode, Json<crate::api_types::ScanJobSubmissionResponse>), AppError> {
    let out = state.service.submit_scan_job(body).await?;
    Ok((StatusCode::ACCEPTED, Json(out)))
}

/// Synchronous scan for dev/tests (`POST /api/v1/scans/sync`).
pub async fn post_scan_sync(
    State(state): State<AppState>,
    Json(body): Json<StartScanRequest>,
) -> Result<(StatusCode, Json<crate::api_types::StartScanResponse>), AppError> {
    let out = state.service.start_scan_and_persist(body).await?;
    Ok((StatusCode::CREATED, Json(out)))
}

pub async fn get_job(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<crate::api_types::ScanJobView>, AppError> {
    let v = state.service.get_scan_job_view(JobId(job_id)).await?;
    Ok(Json(v))
}

pub async fn list_jobs(
    State(state): State<AppState>,
    Query(q): Query<JobListQuery>,
) -> Result<Json<crate::api_types::ScanJobListResponse>, AppError> {
    Ok(Json(state.service.list_scan_jobs(&q).await?))
}

pub async fn get_run(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<crate::api_types::RunSummaryView>, AppError> {
    let v = state.service.get_run_summary_view(RunId(run_id)).await?;
    Ok(Json(v))
}

pub async fn get_report(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
) -> Result<Json<paraclete_types::ScanReport>, AppError> {
    Ok(Json(state.service.get_run_report(RunId(run_id)).await?))
}

pub async fn get_assets(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
    Query(q): Query<PageQuery>,
) -> Result<Json<PagedAssetsResponse>, AppError> {
    let (items, total) = state.service.list_assets_page(RunId(run_id), &q).await?;
    Ok(Json(PagedAssetsResponse {
        items,
        total,
        limit: q.limit.clamp(1, crate::api_types::MAX_PAGE_SIZE),
        offset: q.offset,
    }))
}

pub async fn get_findings(
    State(state): State<AppState>,
    Path(run_id): Path<Uuid>,
    Query(q): Query<PageQuery>,
) -> Result<Json<PagedFindingsResponse>, AppError> {
    let (items, total) = state.service.list_findings_page(RunId(run_id), &q).await?;
    Ok(Json(PagedFindingsResponse {
        items,
        total,
        limit: q.limit.clamp(1, crate::api_types::MAX_PAGE_SIZE),
        offset: q.offset,
    }))
}

pub async fn list_target_runs(
    State(state): State<AppState>,
    Path(target_kind): Path<String>,
    Query(q): Query<TargetRunsQuery>,
) -> Result<Json<Vec<paraclete_types::ScanRunListItem>>, AppError> {
    if !KNOWN_TARGET_KINDS.contains(&target_kind.as_str()) {
        return Err(AppError::TargetNotFound(format!("unknown target_kind `{target_kind}`")));
    }
    let key = q.normalized_key.trim();
    if key.is_empty() {
        return Err(AppError::InvalidRequest("normalized_key query parameter is required".into()));
    }
    let identity = paraclete_types::TargetIdentity { target_kind, normalized_key: key.to_string() };
    Ok(Json(state.service.list_runs_for_target(&identity, q.limit.clamp(1, 500)).await?))
}

pub async fn get_diff(
    State(state): State<AppState>,
    Query(q): Query<DiffQuery>,
) -> Result<Json<paraclete_types::RunDiff>, AppError> {
    let d = state.service.diff_runs(RunId(q.left_run_id), RunId(q.right_run_id)).await?;
    Ok(Json(d))
}
