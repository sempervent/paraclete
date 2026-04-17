//! Human-readable tables and JSON printing.

use comfy_table::{presets::UTF8_FULL, Attribute, Cell, ContentArrangement, Table};
use serde::Serialize;

use crate::api::{
    AssetRow, FindingRow, RunSummaryView, ScanJobListResponse, ScanJobSubmissionResponse,
    ScanJobView,
};
use crate::error::CliError;

pub fn print_json<T: Serialize>(value: &T) -> Result<(), CliError> {
    let s = serde_json::to_string_pretty(value).map_err(|e| CliError::Decode(e.to_string()))?;
    println!("{s}");
    Ok(())
}

pub fn print_health(json: bool, status: &str) -> Result<(), CliError> {
    if json {
        print_json(&serde_json::json!({ "status": status }))?;
    } else {
        println!("{status}");
    }
    Ok(())
}

pub fn print_job_submission(json: bool, r: &ScanJobSubmissionResponse) -> Result<(), CliError> {
    if json {
        print_json(r)?;
    } else {
        println!("job_id:      {}", r.job_id);
        println!("status:      {:?}", r.status);
        println!("submitted:   {}", r.submitted_at);
        println!("target_kind: {}", r.target_kind);
        println!("key:         {}", r.normalized_target_key);
        println!("job_url:     {}", r.job_url);
    }
    Ok(())
}

pub fn print_job_view(json: bool, r: &ScanJobView) -> Result<(), CliError> {
    if json {
        print_json(r)?;
    } else {
        println!("job_id:      {}", r.job_id);
        println!("status:      {:?}", r.status);
        println!("submitted:   {}", r.submitted_at);
        if let Some(s) = &r.started_at {
            println!("started:     {s}");
        }
        if let Some(s) = &r.completed_at {
            println!("completed:   {s}");
        }
        println!("target_kind: {}", r.target_kind);
        println!("key:         {}", r.normalized_target_key);
        if r.attempt_count > 0 {
            println!("attempts:    {}", r.attempt_count);
        }
        if let Some(w) = &r.worker_id {
            println!("worker_id:   {w}");
        }
        if let Some(s) = &r.recovery_note {
            println!("recovery:    {s}");
        }
        if let Some(id) = r.run_id {
            println!("run_id:      {id}");
        }
        if let Some(f) = &r.failure {
            println!("failure:     {:?} — {}", f.code, f.message);
        }
        println!("job_url:     {}", r.job_url);
    }
    Ok(())
}

pub fn print_job_list(json: bool, r: &ScanJobListResponse) -> Result<(), CliError> {
    if json {
        print_json(r)?;
        return Ok(());
    }
    let mut table = Table::new();
    table.load_preset(UTF8_FULL).set_content_arrangement(ContentArrangement::Dynamic).set_header(
        vec!["job_id", "status", "submitted_at", "target_kind", "normalized_key", "run_id"],
    );
    for j in &r.items {
        table.add_row(vec![
            Cell::new(j.job_id.to_string()),
            Cell::new(format!("{:?}", j.status)),
            Cell::new(j.submitted_at.to_rfc3339()),
            Cell::new(&j.target_kind),
            Cell::new(&j.normalized_target_key),
            Cell::new(j.run_id.map(|u| u.to_string()).unwrap_or_default()),
        ]);
    }
    println!("total: {}  (limit {} offset {})", r.total, r.limit, r.offset);
    println!("{table}");
    Ok(())
}

pub fn print_run_summary(json: bool, r: &RunSummaryView) -> Result<(), CliError> {
    if json {
        print_json(r)?;
    } else {
        println!("run_id:       {}", r.run_id);
        println!("scan_id:      {}", r.request_scan_id);
        println!("target_kind:  {}", r.target_kind);
        println!("key:          {}", r.normalized_target_key);
        println!("outcome:      {:?}", r.run_outcome);
        println!("started:      {}", r.started_at);
        println!("completed:    {}", r.completed_at);
        println!("report_sha256: {}", r.report_sha256);
    }
    Ok(())
}

pub fn print_assets(json: bool, page: &crate::api::PagedAssets) -> Result<(), CliError> {
    if json {
        print_json(page)?;
        return Ok(());
    }
    println!("total: {}  (limit {} offset {})", page.total, page.limit, page.offset);
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["path", "format", "status", "dataset_id"]);
    for AssetRow { path, format, inspection_status, dataset_id, .. } in &page.items {
        table.add_row(vec![
            Cell::new(path).add_attribute(Attribute::Dim),
            Cell::new(format),
            Cell::new(inspection_status),
            Cell::new(dataset_id.as_deref().unwrap_or("")),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub fn print_findings(json: bool, page: &crate::api::PagedFindings) -> Result<(), CliError> {
    if json {
        print_json(page)?;
        return Ok(());
    }
    println!("total: {}  (limit {} offset {})", page.total, page.limit, page.offset);
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["code", "severity", "category", "asset", "fingerprint"]);
    for FindingRow { fingerprint, code, severity, category, asset_path, .. } in &page.items {
        table.add_row(vec![
            Cell::new(code),
            Cell::new(severity),
            Cell::new(category),
            Cell::new(asset_path.as_deref().unwrap_or("")),
            Cell::new(&fingerprint[..fingerprint.len().min(16)]),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub fn print_run_list(
    json: bool,
    items: &[paraclete_types::ScanRunListItem],
) -> Result<(), CliError> {
    if json {
        print_json(&items)?;
        return Ok(());
    }
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["run_id", "completed_at", "outcome", "target_kind", "key"]);
    for r in items {
        table.add_row(vec![
            Cell::new(r.run_id.0.to_string()),
            Cell::new(r.completed_at.to_rfc3339()),
            Cell::new(format!("{:?}", r.run_outcome)),
            Cell::new(&r.target_identity.target_kind),
            Cell::new(&r.target_identity.normalized_key),
        ]);
    }
    println!("{table}");
    Ok(())
}
