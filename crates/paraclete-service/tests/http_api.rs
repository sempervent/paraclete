//! HTTP integration tests (hermetic temp SQLite + Axum router).

use std::path::PathBuf;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use camino::Utf8PathBuf;
use http_body_util::BodyExt;
use paraclete_service::{build_router, openapi_spec, ParacleteService};
use paraclete_store::SqliteScanStore;
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

fn sqlite_url(dir: &tempfile::TempDir) -> String {
    let p = dir.path().join("t.sqlite");
    std::fs::File::create(&p).unwrap();
    let abs = p.canonicalize().unwrap();
    format!("sqlite://{}", abs.display())
}

fn fixture(rel: &str) -> Utf8PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures").join(rel);
    Utf8PathBuf::from_path_buf(path).unwrap()
}

#[tokio::test]
async fn health_ok() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteScanStore::connect(&sqlite_url(&dir)).await.unwrap();
    let app = build_router(ParacleteService::new(store));
    let res =
        app.oneshot(Request::get("/api/v1/health").body(Body::empty()).unwrap()).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["status"], "ok");
}

#[tokio::test]
async fn openapi_lists_core_paths() {
    let doc = openapi_spec();
    assert!(doc.paths.paths.contains_key("/api/v1/health"));
    assert!(doc.paths.paths.contains_key("/api/v1/scans"));
    assert!(doc.paths.paths.contains_key("/api/v1/jobs"));
    assert!(doc.paths.paths.contains_key("/api/v1/diff"));
}

#[tokio::test]
async fn scan_persist_summary_report_assets_findings_diff() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteScanStore::connect(&sqlite_url(&dir)).await.unwrap();
    let app = build_router(ParacleteService::new(store));

    let body = json!({
        "target": { "type": "local_file", "path": fixture("phase1/single_parquet/data.parquet").as_str() },
        "profile": "standard",
        "options": { "mode": "full", "max_files": 100000, "format_hints": [] }
    });
    let res = app
        .clone()
        .oneshot(
            Request::post("/api/v1/scans/sync")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let created: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let run_id = created["run_id"].as_str().unwrap().parse::<Uuid>().unwrap();

    let res = app
        .clone()
        .oneshot(Request::get(format!("/api/v1/runs/{run_id}")).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = app
        .clone()
        .oneshot(Request::get(format!("/api/v1/runs/{run_id}/report")).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let res = app
        .clone()
        .oneshot(
            Request::get(format!("/api/v1/runs/{run_id}/assets?limit=10&offset=0"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let page: serde_json::Value =
        serde_json::from_slice(&res.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(page["total"], 1);
    assert_eq!(page["items"].as_array().unwrap().len(), 1);

    let res = app
        .clone()
        .oneshot(
            Request::get(format!("/api/v1/runs/{run_id}/findings?limit=10&offset=0"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // Second run for diff
    let body2 = json!({
        "target": { "type": "local_file", "path": fixture("phase1/tiny_parquet/micro.parquet").as_str() },
        "profile": "standard",
        "options": { "mode": "full", "max_files": 100000, "format_hints": [] }
    });
    let res = app
        .clone()
        .oneshot(
            Request::post("/api/v1/scans/sync")
                .header("content-type", "application/json")
                .body(Body::from(body2.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let run2: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let run_b = run2["run_id"].as_str().unwrap();

    let res = app
        .oneshot(
            Request::get(format!("/api/v1/diff?left_run_id={run_id}&right_run_id={run_b}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn run_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteScanStore::connect(&sqlite_url(&dir)).await.unwrap();
    let app = build_router(ParacleteService::new(store));
    let id = Uuid::new_v4();
    let res = app
        .oneshot(Request::get(format!("/api/v1/runs/{id}")).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["error"]["code"], "run_not_found");
}

#[tokio::test]
async fn invalid_json_request() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteScanStore::connect(&sqlite_url(&dir)).await.unwrap();
    let app = build_router(ParacleteService::new(store));
    let res = app
        .oneshot(
            Request::post("/api/v1/scans")
                .header("content-type", "application/json")
                .body(Body::from("not-json"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        matches!(res.status(), StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY),
        "unexpected status {}",
        res.status()
    );
}

#[tokio::test]
async fn unknown_target_kind_404() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteScanStore::connect(&sqlite_url(&dir)).await.unwrap();
    let app = build_router(ParacleteService::new(store));
    let res = app
        .oneshot(
            Request::get("/api/v1/targets/not_a_kind/runs?normalized_key=x")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_runs_for_target() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteScanStore::connect(&sqlite_url(&dir)).await.unwrap();
    let app = build_router(ParacleteService::new(store));
    let path = fixture("phase1/single_parquet/data.parquet");
    let body = json!({
        "target": { "type": "local_file", "path": path.as_str() },
        "profile": "standard",
        "options": { "mode": "full", "max_files": 100000, "format_hints": [] }
    });
    let _ = app
        .clone()
        .oneshot(
            Request::post("/api/v1/scans/sync")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let key = path.as_str();
    let res = app
        .oneshot(
            Request::get(format!(
                "/api/v1/targets/local_file/runs?normalized_key={}",
                urlencoding::encode(key)
            ))
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn async_scan_job_poll_then_fetch_run() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteScanStore::connect(&sqlite_url(&dir)).await.unwrap();
    let app = build_router(ParacleteService::new(store));

    let body = json!({
        "target": { "type": "local_file", "path": fixture("phase1/single_parquet/data.parquet").as_str() },
        "profile": "standard",
        "options": { "mode": "full", "max_files": 100000, "format_hints": [] }
    });
    let res = app
        .clone()
        .oneshot(
            Request::post("/api/v1/jobs/scans")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let submitted: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let job_id = submitted["job_id"].as_str().unwrap().parse::<Uuid>().unwrap();

    let mut run_id = None;
    for _ in 0..400u32 {
        let res = app
            .clone()
            .oneshot(Request::get(format!("/api/v1/jobs/{job_id}")).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let j: serde_json::Value =
            serde_json::from_slice(&res.into_body().collect().await.unwrap().to_bytes()).unwrap();
        match j["status"].as_str() {
            Some("succeeded") => {
                assert_eq!(j["attempt_count"], 1);
                assert!(j["worker_id"].is_null());
                run_id = j["run_id"].as_str().and_then(|s| s.parse::<Uuid>().ok());
                break;
            }
            Some("failed") => panic!("job failed: {j}"),
            _ => tokio::time::sleep(Duration::from_millis(15)).await,
        }
    }
    let run_id = run_id.expect("job did not succeed in time");

    let res = app
        .oneshot(Request::get(format!("/api/v1/runs/{run_id}")).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn job_not_found_returns_envelope() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteScanStore::connect(&sqlite_url(&dir)).await.unwrap();
    let app = build_router(ParacleteService::new(store));
    let id = Uuid::new_v4();
    let res = app
        .oneshot(Request::get(format!("/api/v1/jobs/{id}")).body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["error"]["code"], "job_not_found");
}

#[tokio::test]
async fn async_scan_job_records_failure() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteScanStore::connect(&sqlite_url(&dir)).await.unwrap();
    let app = build_router(ParacleteService::new(store));

    let body = json!({
        "target": { "type": "local_file", "path": "/no/such/paraclete_missing_file.parquet" },
        "profile": "standard",
        "options": { "mode": "full", "max_files": 100000, "format_hints": [] }
    });
    let res = app
        .clone()
        .oneshot(
            Request::post("/api/v1/scans")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::ACCEPTED);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let submitted: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let job_id = submitted["job_id"].as_str().unwrap().parse::<Uuid>().unwrap();

    for _ in 0..400u32 {
        let res = app
            .clone()
            .oneshot(Request::get(format!("/api/v1/jobs/{job_id}")).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let j: serde_json::Value =
            serde_json::from_slice(&res.into_body().collect().await.unwrap().to_bytes()).unwrap();
        if j["status"] == "failed" {
            assert!(j["failure"].is_object());
            assert_eq!(j["run_id"], serde_json::Value::Null);
            assert!(j.get("attempt_count").is_some());
            return;
        }
        if j["status"] == "succeeded" {
            panic!("expected failure, got {j}");
        }
        tokio::time::sleep(Duration::from_millis(15)).await;
    }
    panic!("job did not fail in time");
}

#[tokio::test]
async fn list_jobs_returns_total() {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteScanStore::connect(&sqlite_url(&dir)).await.unwrap();
    let app = build_router(ParacleteService::new(store));

    let body = json!({
        "target": { "type": "local_file", "path": fixture("phase1/single_parquet/data.parquet").as_str() },
        "profile": "standard",
        "options": { "mode": "full", "max_files": 100000, "format_hints": [] }
    });
    let _ = app
        .clone()
        .oneshot(
            Request::post("/api/v1/jobs/scans")
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let res = app
        .oneshot(Request::get("/api/v1/jobs?limit=10&offset=0").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let page: serde_json::Value =
        serde_json::from_slice(&res.into_body().collect().await.unwrap().to_bytes()).unwrap();
    assert_eq!(page["total"], 1);
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
}
