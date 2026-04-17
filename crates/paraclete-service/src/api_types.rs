//! HTTP-facing DTOs (transport shapes, not storage rows).

use chrono::{DateTime, Utc};
use paraclete_types::{
    JobErrorCode, JobStatus, RedactionPolicy, RunOutcome, ScanOptions, ScanProfile, ScanSummary,
    ScanTarget,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Maximum `limit` for paginated asset/finding endpoints.
pub const MAX_PAGE_SIZE: u32 = 500;
/// Default page size when `limit` is omitted.
pub const DEFAULT_PAGE_SIZE: u32 = 50;

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartScanRequest {
    pub target: ScanTarget,
    pub profile: ScanProfile,
    #[serde(default)]
    pub options: ScanOptions,
    #[serde(default)]
    pub scan_id: Option<Uuid>,
    /// When omitted, [`paraclete_types::RedactionPolicy::transport_safe_persist`] is applied before persistence.
    pub redaction: Option<RedactionPolicy>,
}

#[derive(Debug, Serialize)]
pub struct StartScanResponse {
    pub run_id: Uuid,
    pub target_kind: String,
    pub normalized_target_key: String,
    pub run_outcome: RunOutcome,
    pub summary: ScanSummary,
    /// Relative URL to fetch the canonical stored report.
    pub report_url: String,
}

#[derive(Debug, Serialize)]
pub struct RunSummaryView {
    pub run_id: Uuid,
    pub request_scan_id: Uuid,
    pub target_kind: String,
    pub normalized_target_key: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub run_outcome: RunOutcome,
    pub engine_revision: Option<String>,
    pub contract_schema_version: String,
    pub report_format_version: String,
    pub report_sha256: String,
    pub summary: ScanSummary,
}

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
    pub inspection_status: Option<String>,
    pub severity: Option<String>,
    pub code: Option<String>,
}

fn default_limit() -> u32 {
    DEFAULT_PAGE_SIZE
}

impl PageQuery {
    pub fn clamped(&self) -> (u64, u64) {
        let lim = self.limit.clamp(1, MAX_PAGE_SIZE) as u64;
        let off = self.offset as u64;
        (lim, off)
    }
}

#[derive(Debug, Serialize)]
pub struct PagedAssetsResponse {
    pub items: Vec<paraclete_store::StoredAssetRow>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Serialize)]
pub struct PagedFindingsResponse {
    pub items: Vec<paraclete_store::StoredFindingRow>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Deserialize)]
pub struct DiffQuery {
    pub left_run_id: Uuid,
    pub right_run_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct TargetRunsQuery {
    pub normalized_key: String,
    #[serde(default = "default_list_limit")]
    pub limit: i64,
}

fn default_list_limit() -> i64 {
    50
}

/// Response for `POST /api/v1/jobs/scans` and async `POST /api/v1/scans` (202 Accepted).
#[derive(Debug, Serialize)]
pub struct ScanJobSubmissionResponse {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub submitted_at: DateTime<Utc>,
    pub target_kind: String,
    pub normalized_target_key: String,
    pub job_url: String,
}

#[derive(Debug, Serialize)]
pub struct JobFailureBody {
    pub code: JobErrorCode,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ScanJobView {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub submitted_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub target_kind: String,
    pub normalized_target_key: String,
    /// Monotonic claim count for this job (each dequeue from `queued` increments).
    pub attempt_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worker_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heartbeat_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leased_until: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_id: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<JobFailureBody>,
    pub job_url: String,
}

#[derive(Debug, Serialize)]
pub struct ScanJobListResponse {
    pub items: Vec<ScanJobView>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Deserialize)]
pub struct JobListQuery {
    pub status: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub offset: u32,
}
