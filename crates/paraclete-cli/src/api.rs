//! JSON shapes returned by `/api/v1` (Deserialize for the CLI; mirrors service DTOs).

use chrono::{DateTime, Utc};
use paraclete_types::{AuthTokenStatus, JobErrorCode, JobStatus, RunOutcome, ScanSummary};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorBody {
    pub error: ErrorEnvelope,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ErrorEnvelope {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhoAmIResponse {
    pub token_id: Uuid,
    pub label: String,
    pub role: paraclete_types::AuthRole,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScanJobSubmissionResponse {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub submitted_at: DateTime<Utc>,
    pub target_kind: String,
    pub normalized_target_key: String,
    pub job_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JobFailureBody {
    pub code: JobErrorCode,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScanJobView {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub submitted_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub target_kind: String,
    pub normalized_target_key: String,
    #[serde(default)]
    pub attempt_count: i64,
    pub worker_id: Option<String>,
    pub heartbeat_at: Option<DateTime<Utc>>,
    pub leased_until: Option<DateTime<Utc>>,
    pub recovery_note: Option<String>,
    pub run_id: Option<Uuid>,
    pub failure: Option<JobFailureBody>,
    pub job_url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScanJobListResponse {
    pub items: Vec<ScanJobView>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct PagedAssets {
    pub items: Vec<AssetRow>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AssetRow {
    pub path: String,
    pub format: String,
    pub inspection_status: String,
    #[serde(default)]
    pub failure_kind: Option<serde_json::Value>,
    pub failure_message: Option<String>,
    pub dataset_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PagedFindings {
    pub items: Vec<FindingRow>,
    pub total: u64,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FindingRow {
    pub fingerprint: String,
    pub code: String,
    pub severity: String,
    pub category: String,
    pub asset_path: Option<String>,
    pub dataset_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthTokenCreateResponse {
    pub token_id: Uuid,
    pub label: String,
    pub role: paraclete_types::AuthRole,
    pub created_at: DateTime<Utc>,
    pub token_secret: String,
    pub token_prefix: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthTokenSummaryView {
    pub token_id: Uuid,
    pub label: String,
    pub role: paraclete_types::AuthRole,
    pub status: AuthTokenStatus,
    pub created_at: DateTime<Utc>,
    pub disabled_at: Option<DateTime<Utc>>,
    pub token_prefix: Option<String>,
    pub note: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub replaced_by_token_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthTokenRotateResponse {
    pub token_id: Uuid,
    pub previous_token_id: Uuid,
    pub label: String,
    pub role: paraclete_types::AuthRole,
    pub created_at: DateTime<Utc>,
    pub token_secret: String,
    pub token_prefix: Option<String>,
    pub note: Option<String>,
    pub previous_disabled_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthTokenListResponse {
    pub items: Vec<AuthTokenSummaryView>,
}
