//! Minimal HTTP client for Paraclete `/api/v1`.

use std::time::Duration;

use paraclete_types::{RunDiff, ScanRunListItem};
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde::Serialize;
use uuid::Uuid;

use crate::api::{
    ErrorBody, HealthResponse, PagedAssets, PagedFindings, RunSummaryView, ScanJobListResponse,
    ScanJobSubmissionResponse, ScanJobView,
};
use crate::error::CliError;
use crate::wire::StartScanRequest;

/// HTTP client (always talks to the service; no local engine or store).
pub struct ApiClient {
    base: String,
    http: reqwest::Client,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, CliError> {
        let base = base_url.into().trim_end_matches('/').to_string();
        Url::parse(&base).map_err(|e| CliError::Msg(format!("invalid base URL: {e}")))?;
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()
            .map_err(|e| CliError::Msg(e.to_string()))?;
        Ok(Self { base, http })
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base, path)
    }

    async fn get_bytes(&self, path: &str) -> Result<Vec<u8>, CliError> {
        let url = self.url(path);
        let resp = self.http.get(url).send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?.to_vec();
        if !status.is_success() {
            return Err(decode_api_error(status.as_u16(), &bytes));
        }
        Ok(bytes)
    }

    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, CliError> {
        let bytes = self.get_bytes(path).await?;
        serde_json::from_slice(&bytes).map_err(|e| CliError::Decode(format!("{path}: {e}")))
    }

    async fn post_json<B: Serialize + ?Sized, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, CliError> {
        let url = self.url(path);
        let resp = self.http.post(url).json(body).send().await?;
        let status = resp.status();
        let bytes = resp.bytes().await?.to_vec();
        if !status.is_success() {
            return Err(decode_api_error(status.as_u16(), &bytes));
        }
        serde_json::from_slice(&bytes).map_err(|e| CliError::Decode(e.to_string()))
    }

    pub async fn health(&self) -> Result<HealthResponse, CliError> {
        self.get_json("/api/v1/health").await
    }

    pub async fn submit_scan_job(
        &self,
        body: &StartScanRequest,
    ) -> Result<ScanJobSubmissionResponse, CliError> {
        self.post_json("/api/v1/jobs/scans", body).await
    }

    pub async fn get_job(&self, job_id: Uuid) -> Result<ScanJobView, CliError> {
        self.get_json(&format!("/api/v1/jobs/{job_id}")).await
    }

    pub async fn list_jobs(
        &self,
        status: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<ScanJobListResponse, CliError> {
        let mut path = format!("/api/v1/jobs?limit={limit}&offset={offset}");
        if let Some(s) = status {
            path.push_str(&format!("&status={}", urlencoding::encode(s)));
        }
        self.get_json(&path).await
    }

    pub async fn get_run_summary(&self, run_id: Uuid) -> Result<RunSummaryView, CliError> {
        self.get_json(&format!("/api/v1/runs/{run_id}")).await
    }

    pub async fn get_run_report_json(&self, run_id: Uuid) -> Result<serde_json::Value, CliError> {
        let bytes = self.get_bytes(&format!("/api/v1/runs/{run_id}/report")).await?;
        serde_json::from_slice(&bytes).map_err(|e| CliError::Decode(e.to_string()))
    }

    pub async fn list_assets(
        &self,
        run_id: Uuid,
        limit: u32,
        offset: u32,
        inspection_status: Option<&str>,
    ) -> Result<PagedAssets, CliError> {
        let mut path = format!("/api/v1/runs/{run_id}/assets?limit={limit}&offset={offset}");
        if let Some(s) = inspection_status {
            path.push_str(&format!("&inspection_status={}", urlencoding::encode(s)));
        }
        self.get_json(&path).await
    }

    pub async fn list_findings(
        &self,
        run_id: Uuid,
        limit: u32,
        offset: u32,
        severity: Option<&str>,
        code: Option<&str>,
    ) -> Result<PagedFindings, CliError> {
        let mut path = format!("/api/v1/runs/{run_id}/findings?limit={limit}&offset={offset}");
        if let Some(s) = severity {
            path.push_str(&format!("&severity={}", urlencoding::encode(s)));
        }
        if let Some(c) = code {
            path.push_str(&format!("&code={}", urlencoding::encode(c)));
        }
        self.get_json(&path).await
    }

    pub async fn list_runs_for_target(
        &self,
        target_kind: &str,
        normalized_key: &str,
        limit: i64,
    ) -> Result<Vec<ScanRunListItem>, CliError> {
        let key = urlencoding::encode(normalized_key);
        let path = format!("/api/v1/targets/{target_kind}/runs?normalized_key={key}&limit={limit}");
        self.get_json(&path).await
    }

    pub async fn diff_runs(&self, left: Uuid, right: Uuid) -> Result<RunDiff, CliError> {
        let path = format!("/api/v1/diff?left_run_id={left}&right_run_id={right}");
        self.get_json(&path).await
    }
}

fn decode_api_error(status: u16, bytes: &[u8]) -> CliError {
    if let Ok(body) = serde_json::from_slice::<ErrorBody>(bytes) {
        CliError::Api { code: body.error.code, message: body.error.message }
    } else {
        CliError::Http { status, body: String::from_utf8_lossy(bytes).to_string() }
    }
}
