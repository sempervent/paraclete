//! Application use cases: orchestrates `ScanEngine` and `SqliteScanStore` only.

use chrono::{DateTime, Utc};
use paraclete_core::ScanEngine;
use paraclete_store::{ScanJobRow, SqliteScanStore, StoredAssetRow, StoredFindingRow};
use paraclete_types::{
    validate_report, JobErrorCode, JobId, JobStatus, RedactionPolicy, RunId, RunOutcome,
    ScanReport, ScanRequest, ScanRunListItem, TargetIdentity,
};

use crate::api_types::{
    JobFailureBody, JobListQuery, PageQuery, RunSummaryView, ScanJobListResponse,
    ScanJobSubmissionResponse, ScanJobView, StartScanRequest, StartScanResponse,
};
use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct ParacleteService {
    store: SqliteScanStore,
}

impl ParacleteService {
    pub fn new(store: SqliteScanStore) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &SqliteScanStore {
        &self.store
    }

    /// Runs a local scan, applies redaction policy, validates, and persists (same path as the job worker).
    pub async fn execute_scan_and_persist(
        &self,
        req: StartScanRequest,
    ) -> Result<StartScanResponse, AppError> {
        Self::assert_local_scan_target(&req)?;

        let mut scan_req = ScanRequest::new(req.target.clone(), req.profile);
        scan_req.options = req.options.clone();
        if let Some(id) = req.scan_id {
            scan_req.scan_id = id;
        }

        let started = Utc::now();
        let report = ScanEngine::run(&scan_req)?;
        validate_report(&report)?;
        let completed = Utc::now();

        let redaction =
            req.redaction.clone().unwrap_or_else(RedactionPolicy::transport_safe_persist);

        let run_id = RunId::new();
        self.store.persist_scan_run(run_id, started, completed, &report, &redaction).await?;

        let outcome = if report.summary.partial_inspection {
            RunOutcome::CompletedPartial
        } else {
            RunOutcome::Completed
        };
        let identity = TargetIdentity::from_scan_target(&scan_req.target);
        Ok(StartScanResponse {
            run_id: run_id.0,
            target_kind: identity.target_kind,
            normalized_target_key: identity.normalized_key,
            run_outcome: outcome,
            summary: report.summary,
            report_url: format!("/api/v1/runs/{}/report", run_id.0),
        })
    }

    /// Synchronous scan + persist (dev/tests via `POST /api/v1/scans/sync`).
    pub async fn start_scan_and_persist(
        &self,
        req: StartScanRequest,
    ) -> Result<StartScanResponse, AppError> {
        self.execute_scan_and_persist(req).await
    }

    fn assert_local_scan_target(req: &StartScanRequest) -> Result<(), AppError> {
        match &req.target {
            paraclete_types::ScanTarget::LocalFile { .. }
            | paraclete_types::ScanTarget::LocalDirectory { .. } => Ok(()),
            _ => Err(AppError::TargetNotFound(
                "only local_file and local_directory targets are supported by this API".into(),
            )),
        }
    }

    pub async fn submit_scan_job(
        &self,
        req: StartScanRequest,
    ) -> Result<ScanJobSubmissionResponse, AppError> {
        Self::assert_local_scan_target(&req)?;
        let jid = JobId::new();
        let identity = TargetIdentity::from_scan_target(&req.target);
        let request_json = serde_json::to_string(&req)?;
        self.store.insert_scan_job_queued(jid, &identity, &request_json).await?;
        let submitted_at = Utc::now();
        Ok(ScanJobSubmissionResponse {
            job_id: jid.0,
            status: JobStatus::Queued,
            submitted_at,
            target_kind: identity.target_kind,
            normalized_target_key: identity.normalized_key,
            job_url: format!("/api/v1/jobs/{}", jid.0),
        })
    }

    pub async fn get_scan_job_view(&self, job_id: JobId) -> Result<ScanJobView, AppError> {
        let row = self.store.get_scan_job(job_id).await?;
        scan_job_row_to_view(row)
    }

    pub async fn list_scan_jobs(&self, q: &JobListQuery) -> Result<ScanJobListResponse, AppError> {
        let status_filter = match q.status.as_deref() {
            None => None,
            Some(s) => Some(normalize_job_status_filter(s)?),
        };
        let limit = q.limit.clamp(1, crate::api_types::MAX_PAGE_SIZE) as u64;
        let offset = q.offset as u64;
        let total = self.store.count_scan_jobs(status_filter).await?;
        let rows = self.store.list_scan_jobs_page(status_filter, offset, limit).await?;
        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            items.push(scan_job_row_to_view(row)?);
        }
        Ok(ScanJobListResponse {
            items,
            total,
            limit: q.limit.clamp(1, crate::api_types::MAX_PAGE_SIZE),
            offset: q.offset,
        })
    }

    pub async fn get_run_summary_view(&self, run_id: RunId) -> Result<RunSummaryView, AppError> {
        let meta = self.store.get_run_public_meta(run_id).await?;
        Ok(RunSummaryView {
            run_id: meta.run_id.0,
            request_scan_id: meta.request_scan_id,
            target_kind: meta.target_identity.target_kind,
            normalized_target_key: meta.target_identity.normalized_key,
            started_at: meta.started_at,
            completed_at: meta.completed_at,
            run_outcome: meta.run_outcome,
            engine_revision: meta.engine_revision,
            contract_schema_version: meta.contract_schema_version,
            report_format_version: meta.report_format_version,
            report_sha256: meta.report_sha256,
            summary: meta.summary,
        })
    }

    pub async fn get_run_report(&self, run_id: RunId) -> Result<ScanReport, AppError> {
        Ok(self.store.load_report(run_id).await?)
    }

    pub async fn list_runs_for_target(
        &self,
        identity: &TargetIdentity,
        limit: i64,
    ) -> Result<Vec<ScanRunListItem>, AppError> {
        Ok(self.store.list_runs_for_target(identity, limit).await?)
    }

    pub async fn diff_runs(
        &self,
        left: RunId,
        right: RunId,
    ) -> Result<paraclete_types::RunDiff, AppError> {
        let a = self.store.load_report(left).await?;
        let b = self.store.load_report(right).await?;
        Ok(paraclete_store::diff_reports(left, right, &a, &b))
    }

    pub async fn list_assets_page(
        &self,
        run_id: RunId,
        q: &PageQuery,
    ) -> Result<(Vec<StoredAssetRow>, u64), AppError> {
        let (limit, offset) = q.clamped();
        let st = q.inspection_status.as_deref();
        let total = self.store.count_assets(run_id, st).await?;
        let items = self.store.list_assets_page(run_id, st, offset, limit).await?;
        Ok((items, total))
    }

    pub async fn list_findings_page(
        &self,
        run_id: RunId,
        q: &PageQuery,
    ) -> Result<(Vec<StoredFindingRow>, u64), AppError> {
        let (limit, offset) = q.clamped();
        let sev = q.severity.as_deref();
        let code = q.code.as_deref();
        let total = self.store.count_findings(run_id, sev, code).await?;
        let items = self.store.list_findings_page(run_id, sev, code, offset, limit).await?;
        Ok((items, total))
    }
}

fn normalize_job_status_filter(s: &str) -> Result<&'static str, AppError> {
    match s.trim().to_lowercase().as_str() {
        "queued" => Ok("queued"),
        "running" => Ok("running"),
        "succeeded" => Ok("succeeded"),
        "failed" => Ok("failed"),
        "canceled" => Ok("canceled"),
        _ => Err(AppError::InvalidRequest(format!("unknown job status filter `{s}`"))),
    }
}

fn scan_job_row_to_view(row: ScanJobRow) -> Result<ScanJobView, AppError> {
    let job_id = uuid::Uuid::parse_str(&row.job_id)
        .map_err(|_| AppError::Internal("invalid job_id in store".into()))?;
    let status = parse_job_status_label(&row.status)?;
    let submitted_at = parse_rfc3339(&row.submitted_at)?;
    let started_at = row.started_at.as_deref().map(parse_rfc3339).transpose()?;
    let completed_at = row.completed_at.as_deref().map(parse_rfc3339).transpose()?;
    let run_id = row
        .run_id
        .as_deref()
        .map(uuid::Uuid::parse_str)
        .transpose()
        .map_err(|_| AppError::Internal("invalid run_id on job".into()))?;

    let failure = if status == JobStatus::Failed {
        let code = row
            .failure_code
            .as_deref()
            .map(parse_failure_code)
            .unwrap_or(JobErrorCode::InternalError);
        let message = row.failure_message.unwrap_or_default();
        Some(JobFailureBody { code, message })
    } else {
        None
    };

    let heartbeat_at = row.heartbeat_at.as_deref().map(parse_rfc3339).transpose()?;
    let leased_until = row.leased_until.as_deref().map(parse_rfc3339).transpose()?;

    Ok(ScanJobView {
        job_id,
        status,
        submitted_at,
        started_at,
        completed_at,
        target_kind: row.target_kind,
        normalized_target_key: row.normalized_target_key,
        attempt_count: row.attempt_count,
        worker_id: row.worker_id,
        heartbeat_at,
        leased_until,
        recovery_note: row.recovery_note,
        run_id,
        failure,
        job_url: format!("/api/v1/jobs/{job_id}"),
    })
}

fn parse_rfc3339(s: &str) -> Result<DateTime<Utc>, AppError> {
    DateTime::parse_from_rfc3339(s)
        .map(|d| d.with_timezone(&Utc))
        .map_err(|_| AppError::Internal("invalid timestamp in job row".into()))
}

fn parse_job_status_label(s: &str) -> Result<JobStatus, AppError> {
    match s {
        "queued" => Ok(JobStatus::Queued),
        "running" => Ok(JobStatus::Running),
        "succeeded" => Ok(JobStatus::Succeeded),
        "failed" => Ok(JobStatus::Failed),
        "canceled" => Ok(JobStatus::Canceled),
        _ => Err(AppError::Internal(format!("unknown job status `{s}`"))),
    }
}

fn parse_failure_code(s: &str) -> JobErrorCode {
    match s {
        "invalid_request" => JobErrorCode::InvalidRequest,
        "target_not_found" => JobErrorCode::TargetNotFound,
        "scan_failed" => JobErrorCode::ScanFailed,
        "store_error" => JobErrorCode::StoreError,
        "internal_error" => JobErrorCode::InternalError,
        "worker_lost" => JobErrorCode::WorkerLost,
        "job_retries_exhausted" => JobErrorCode::JobRetriesExhausted,
        _ => JobErrorCode::InternalError,
    }
}
