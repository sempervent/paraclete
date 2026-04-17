//! OpenAPI document for the Phase 5 HTTP surface (built with utoipa builders).

use utoipa::openapi::path::{HttpMethod, Operation, Paths};
use utoipa::openapi::{Info, OpenApiBuilder};

fn op_get(summary: &str) -> Operation {
    let mut o = Operation::new();
    o.summary = Some(summary.to_string());
    o
}

fn op_post(summary: &str) -> Operation {
    let mut o = Operation::new();
    o.summary = Some(summary.to_string());
    o
}

/// OpenAPI 3.1 root document listing exposed routes (summaries only; schemas evolve later).
pub fn openapi_spec() -> utoipa::openapi::OpenApi {
    let mut paths = Paths::new();
    paths.add_path_operation("/api/v1/health", vec![HttpMethod::Get], op_get("Health check"));
    paths.add_path_operation(
        "/api/v1/openapi.json",
        vec![HttpMethod::Get],
        op_get("OpenAPI document"),
    );
    paths.add_path_operation(
        "/api/v1/scans",
        vec![HttpMethod::Post],
        op_post("Queue async scan job (202 Accepted)"),
    );
    paths.add_path_operation(
        "/api/v1/scans/sync",
        vec![HttpMethod::Post],
        op_post("Synchronous local scan and persist run (dev/tests)"),
    );
    paths.add_path_operation(
        "/api/v1/jobs/scans",
        vec![HttpMethod::Post],
        op_post("Queue async scan job"),
    );
    paths.add_path_operation(
        "/api/v1/jobs/{job_id}",
        vec![HttpMethod::Get],
        op_get("Get scan job status and result"),
    );
    paths.add_path_operation(
        "/api/v1/jobs",
        vec![HttpMethod::Get],
        op_get("List recent scan jobs"),
    );
    paths.add_path_operation(
        "/api/v1/runs/{run_id}",
        vec![HttpMethod::Get],
        op_get("Run summary (no full report blob)"),
    );
    paths.add_path_operation(
        "/api/v1/runs/{run_id}/report",
        vec![HttpMethod::Get],
        op_get("Full canonical ScanReport"),
    );
    paths.add_path_operation(
        "/api/v1/runs/{run_id}/assets",
        vec![HttpMethod::Get],
        op_get("Paginated asset projections"),
    );
    paths.add_path_operation(
        "/api/v1/runs/{run_id}/findings",
        vec![HttpMethod::Get],
        op_get("Paginated finding projections"),
    );
    paths.add_path_operation(
        "/api/v1/targets/{target_kind}/runs",
        vec![HttpMethod::Get],
        op_get("List runs for a target identity"),
    );
    paths.add_path_operation("/api/v1/diff", vec![HttpMethod::Get], op_get("Diff two runs"));

    OpenApiBuilder::new().info(Info::new("Paraclete API", "0.8.0")).paths(paths).build()
}
