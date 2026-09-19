//! End-to-end CLI against a hermetic Axum server + temp SQLite.
//!
//! Subprocess calls run in `spawn_blocking` so the Tokio runtime can keep serving HTTP while
//! the CLI blocks on `job wait`.

use std::path::PathBuf;
use std::process::Output;
use std::time::Duration;

use assert_cmd::Command;
use camino::Utf8PathBuf;
use paraclete_service::{build_router, ParacleteService};
use paraclete_store::SqliteScanStore;
use paraclete_types::AuthRole;

const E2E_TOKEN: &str = "paraclete-e2e-test-token";

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

/// Returns a [`tempfile::TempDir`] handle — keep it in scope for the whole test or the DB file is removed.
async fn spawn_test_server() -> (tokio::task::JoinHandle<()>, String, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let store = SqliteScanStore::connect(&sqlite_url(&dir)).await.unwrap();
    store.insert_auth_token("e2e", E2E_TOKEN, AuthRole::Admin).await.unwrap();
    let app = build_router(ParacleteService::new(store));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    tokio::time::sleep(Duration::from_millis(80)).await;
    let base = format!("http://{}", addr);
    (server, base, dir)
}

/// Run `paraclete` in a blocking pool (avoids starving the in-process Axum server).
async fn paraclete_output(base: &str, args: Vec<String>) -> Output {
    paraclete_output_with_token(base, E2E_TOKEN, args).await
}

async fn paraclete_output_with_token(base: &str, token: &str, args: Vec<String>) -> Output {
    let base = base.to_string();
    let token = token.to_string();
    tokio::task::spawn_blocking(move || {
        Command::cargo_bin("paraclete")
            .unwrap()
            .env("PARACLETE_BASE_URL", base)
            .env("PARACLETE_TOKEN", token)
            .args(args)
            .output()
            .expect("paraclete subprocess")
    })
    .await
    .expect("join")
}

async fn submit_wait_run_id(base: &str, file: &Utf8PathBuf) -> String {
    let out = paraclete_output(
        base,
        vec![
            "--json".into(),
            "scan".into(),
            "submit".into(),
            "--file".into(),
            file.as_str().to_string(),
            "--profile".into(),
            "standard".into(),
        ],
    )
    .await;
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let job_id = v["job_id"].as_str().unwrap().to_string();

    let w = paraclete_output(
        base,
        vec![
            "--json".into(),
            "job".into(),
            "wait".into(),
            job_id,
            "--timeout-secs".into(),
            "120".into(),
        ],
    )
    .await;
    assert!(w.status.success(), "{}", String::from_utf8_lossy(&w.stderr));
    let jw: serde_json::Value = serde_json::from_slice(&w.stdout).unwrap();
    jw["run_id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn submit_wait_and_get_run() {
    let (_s, base, _dir) = spawn_test_server().await;
    let fp = fixture("phase1/single_parquet/data.parquet");
    let run_id = submit_wait_run_id(&base, &fp).await;

    let r =
        paraclete_output(&base, vec!["--json".into(), "run".into(), "get".into(), run_id.clone()])
            .await;
    assert!(r.status.success());
    let run: serde_json::Value = serde_json::from_slice(&r.stdout).unwrap();
    assert_eq!(run["run_id"].as_str().unwrap(), run_id);
}

#[tokio::test]
async fn diff_two_runs() {
    let (_s, base, _dir) = spawn_test_server().await;
    let run_a = submit_wait_run_id(&base, &fixture("phase1/single_parquet/data.parquet")).await;
    let run_b = submit_wait_run_id(&base, &fixture("phase1/tiny_parquet/micro.parquet")).await;

    let d =
        paraclete_output(&base, vec!["--json".into(), "diff".into(), run_a.clone(), run_b.clone()])
            .await;
    assert!(d.status.success());
    let diff: serde_json::Value = serde_json::from_slice(&d.stdout).unwrap();
    assert!(diff.get("run_a").is_some());
}

#[tokio::test]
async fn run_not_found_exits_nonzero() {
    let (_s, base, _dir) = spawn_test_server().await;
    let out = paraclete_output(
        &base,
        vec!["run".into(), "get".into(), "00000000-0000-4000-8000-000000000099".into()],
    )
    .await;
    assert!(!out.status.success());
}

#[tokio::test]
async fn job_wait_failed_exits_nonzero() {
    let (_s, base, _dir) = spawn_test_server().await;
    let tj = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(
        tj.path(),
        r#"{"type":"local_file","path":"/no/such/paraclete_missing_cli_test.parquet"}"#,
    )
    .unwrap();
    let out = paraclete_output(
        &base,
        vec![
            "--json".into(),
            "scan".into(),
            "submit".into(),
            "--target-json".into(),
            tj.path().to_string_lossy().into_owned(),
            "--profile".into(),
            "standard".into(),
        ],
    )
    .await;
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let job_id = v["job_id"].as_str().unwrap().to_string();

    let w = paraclete_output(
        &base,
        vec![
            "--json".into(),
            "job".into(),
            "wait".into(),
            job_id,
            "--timeout-secs".into(),
            "120".into(),
        ],
    )
    .await;
    assert!(!w.status.success());
}

#[tokio::test]
async fn token_create_list_disable_and_reject_disabled_secret() {
    let (_s, base, _dir) = spawn_test_server().await;
    let create = paraclete_output(
        &base,
        vec![
            "--json".into(),
            "token".into(),
            "create".into(),
            "--label".into(),
            "cli-reader".into(),
            "--role".into(),
            "reader".into(),
        ],
    )
    .await;
    assert!(create.status.success(), "{}", String::from_utf8_lossy(&create.stderr));
    let v: serde_json::Value = serde_json::from_slice(&create.stdout).unwrap();
    let secret = v["token_secret"].as_str().unwrap().to_string();
    let tid = v["token_id"].as_str().unwrap().to_string();

    let who =
        paraclete_output_with_token(&base, &secret, vec!["--json".into(), "who-am-i".into()]).await;
    assert!(who.status.success(), "who-am-i stderr: {}", String::from_utf8_lossy(&who.stderr));
    let w: serde_json::Value = serde_json::from_slice(&who.stdout).unwrap();
    assert_eq!(w["role"], "reader");

    let list = paraclete_output(&base, vec!["--json".into(), "token".into(), "list".into()]).await;
    assert!(list.status.success());
    let items: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    assert!(items["items"].as_array().unwrap().len() >= 2);

    let dis = paraclete_output(
        &base,
        vec!["--json".into(), "token".into(), "disable".into(), tid.clone()],
    )
    .await;
    assert!(dis.status.success());
    let d: serde_json::Value = serde_json::from_slice(&dis.stdout).unwrap();
    assert_eq!(d["status"], "disabled");

    let bad = paraclete_output_with_token(&base, &secret, vec!["who-am-i".into()]).await;
    assert!(!bad.status.success());
}

#[tokio::test]
async fn token_rotate_cli_new_secret_works_old_fails() {
    let (_s, base, _dir) = spawn_test_server().await;
    let create = paraclete_output(
        &base,
        vec![
            "--json".into(),
            "token".into(),
            "create".into(),
            "--label".into(),
            "rot-cli".into(),
            "--role".into(),
            "reader".into(),
        ],
    )
    .await;
    assert!(create.status.success(), "{}", String::from_utf8_lossy(&create.stderr));
    let v: serde_json::Value = serde_json::from_slice(&create.stdout).unwrap();
    let secret = v["token_secret"].as_str().unwrap().to_string();
    let tid = v["token_id"].as_str().unwrap().to_string();

    let who =
        paraclete_output_with_token(&base, &secret, vec!["--json".into(), "who-am-i".into()]).await;
    assert!(who.status.success());

    let rot = paraclete_output(
        &base,
        vec!["--json".into(), "token".into(), "rotate".into(), tid.clone()],
    )
    .await;
    assert!(rot.status.success(), "{}", String::from_utf8_lossy(&rot.stderr));
    let r: serde_json::Value = serde_json::from_slice(&rot.stdout).unwrap();
    let new_secret = r["token_secret"].as_str().unwrap().to_string();
    assert_eq!(r["previous_token_id"].as_str().unwrap(), tid);
    assert_ne!(new_secret, secret);

    let list = paraclete_output(&base, vec!["--json".into(), "token".into(), "list".into()]).await;
    assert!(list.status.success());
    let list_v: serde_json::Value = serde_json::from_slice(&list.stdout).unwrap();
    let items = list_v["items"].as_array().unwrap();
    assert!(items.iter().any(|row| row["token_id"].as_str() == Some(tid.as_str())));

    let bad = paraclete_output_with_token(&base, &secret, vec!["who-am-i".into()]).await;
    assert!(!bad.status.success());

    let ok =
        paraclete_output_with_token(&base, &new_secret, vec!["--json".into(), "who-am-i".into()])
            .await;
    assert!(ok.status.success());
}
